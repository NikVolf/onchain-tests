// This file is part of Gear.
//
// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{bail, Context as _, Result};
use wasm_graph::{
    DataSegment, ElementSegment, EntryRef, ExportLocal, Func, ImportedOrDeclared, Instruction,
    Memory, Module, SegmentLocation, Table,
};

struct Context {
    module: Module,
}

impl Context {
    pub fn new(module: Module) -> Self {
        Self { module }
    }

    pub fn test_funcs(&self) -> Vec<EntryRef<Func>> {
        self.module
            .exports
            .iter()
            .filter_map(|export| {
                if export.name.starts_with("test_") {
                    match export.local {
                        ExportLocal::Func(ref func_ref) => Some(func_ref.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn default_memory(&self) -> Result<EntryRef<Memory>> {
        match self.module.memory.get(0) {
            None => {
                bail!("Default memory not found in the module");
            }
            Some(mem) => Ok(mem),
        }
    }

    pub fn default_table(&self) -> Result<EntryRef<Table>> {
        match self.module.tables.get(0) {
            None => {
                bail!("Default table not found in the module");
            }
            Some(mem) => Ok(mem),
        }
    }

    pub fn extend_default_table(&self, amount: u32) -> Result<u32> {
        let table = self.default_table()?;
        let mut table_mut = table.write();

        let original = table_mut.limits.initial();

        let new_limits = parity_wasm::elements::ResizableLimits::new(
            table_mut.limits.initial() + amount,
            table_mut
                .limits
                .maximum()
                .and_then(|max| Some(max + amount)),
        );

        table_mut.limits = new_limits;

        Ok(original)
    }

    pub fn store_funcs(&mut self, at: u32, funcs: &[EntryRef<Func>]) {
        let new_element_segment = ElementSegment {
            location: SegmentLocation::Default(vec![
                Instruction::Plain(parity_wasm::elements::Instruction::I32Const(at as i32)),
                Instruction::Plain(parity_wasm::elements::Instruction::End),
            ]),
            value: funcs.iter().cloned().collect(),
        };
        self.module.elements.push(new_element_segment);
    }

    /// Returns pointer to the free space
    pub fn allocate(&self, size: usize) -> Result<u32> {
        if size == 0 {
            bail!("Cannot allocate zero bytes");
        }

        // Extending the memory
        let mem = self.default_memory()?;
        let mut mem_mut = mem.write();

        let extra_pages = (size / 65536) + 1;
        let ptr = mem_mut.limits.initial() * 65536;

        let new_limits = parity_wasm::elements::ResizableLimits::new(
            mem_mut.limits.initial() + (extra_pages as u32),
            mem_mut.limits.maximum(),
        );

        mem_mut.limits = new_limits;

        Ok(ptr)
    }

    /// Append function pointers, allocating new data segment
    ///
    /// Inside the program, this should accessible as (len, &[*const unsafe extern "C" fn()])
    /// Returns pointer where the data is located
    pub fn store_func_ptrs(&mut self, funcs: &[EntryRef<Func>]) -> Result<i32> {
        // Store funcs in the table extension
        let fn_ptr_start = self.extend_default_table(funcs.len() as u32)?;
        self.store_funcs(fn_ptr_start, funcs);

        let data_len = (funcs.len() + 1) * 4; // extra 4 bytes to strore number of functions
        let ptr = self.allocate(data_len)?;
        let mut data = Vec::with_capacity(data_len);
        data.extend((funcs.len() as u32).to_le_bytes());
        for idx in 0..(funcs.len() as u32) {
            data.extend((idx + fn_ptr_start).to_le_bytes());
        }

        let new_data_segment = DataSegment {
            location: SegmentLocation::Default(vec![
                Instruction::Plain(parity_wasm::elements::Instruction::I32Const(ptr as i32)),
                Instruction::Plain(parity_wasm::elements::Instruction::End),
            ]),
            value: data,
        };
        self.module.data.push(new_data_segment);

        Ok(ptr as i32)
    }

    fn get_exported_impl(&self, export_name: &str) -> Result<EntryRef<Func>> {
        let export = match self
            .module
            .exports
            .iter()
            .find(|export| &export.name == export_name)
        {
            Some(export) => export,
            None => {
                bail!(
                    "'{0}' function is not exported, which is invalid",
                    export_name
                );
            }
        };

        let local_impl = match export.local {
            ExportLocal::Func(ref func_ref) => func_ref,
            _ => bail!(
                "'{0}' export is of invalid type, expected function",
                export_name
            ),
        };

        Ok(local_impl.clone())
    }

    pub fn run_tests_impl(&self) -> Result<EntryRef<Func>> {
        self.get_exported_impl("run_tests")
    }

    pub fn handle_impl(&self) -> Result<EntryRef<Func>> {
        self.get_exported_impl("handle")
    }

    pub fn to_module(self) -> Module {
        self.module
    }
}

pub fn extract(module: parity_wasm::elements::Module) -> Result<parity_wasm::elements::Module> {
    let module = Module::from_elements(&module).with_context(|| "Unable to parse module")?;
    let mut context = Context::new(module);

    let test_funcs = context.test_funcs();
    let handle_impl = context.handle_impl()?;
    let run_tests_impl = context.run_tests_impl()?;

    let i32_ptr = context.store_func_ptrs(&test_funcs[..])?;

    {
        // Block to end borrowing at the end
        let mut handle_func = handle_impl.write();
        match handle_func.origin {
            ImportedOrDeclared::Imported(..) => {
                bail!("'handle' function declared as import, which is invalid");
            }
            ImportedOrDeclared::Declared(ref mut body) => {
                body.locals.clear();
                body.code = vec![
                    Instruction::Plain(parity_wasm::elements::Instruction::I32Const(i32_ptr)),
                    Instruction::Call(run_tests_impl.clone()),
                    Instruction::Plain(parity_wasm::elements::Instruction::End),
                ];
            }
        }
    }

    let mut module = context.to_module();

    module
        .exports
        .retain(|export| !(export.name.starts_with("test_") || export.name == "run_tests"));

    let result = module.generate()?;

    Ok(result)
}
