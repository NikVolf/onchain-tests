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
use wasm_graph::{EntryRef, ExportLocal, Func, ImportedOrDeclared, Instruction, Module, Memory};

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
            None => { bail!("Default memory not found in the module"); }
            Some(mem) => Ok(mem),
        }
    }

    /// Returns pointer to the free space
    pub fn allocate(&self, size: usize) -> Result<u32> {
        if size == 0 { bail!("Cannot allocate zero bytes"); }

        // Extending the memory
        let mem = self.default_memory()?;
        let mut mem_mut = mem.write();

        let extra_pages = (size / 65536) + 1;
        let ptr = mem_mut.limits.initial() * 65536;

        let new_limits = parity_wasm::elements::ResizableLimits::new(
            mem_mut.limits.initial() + (extra_pages as u32),
            mem_mut.limits.maximum(),
        );

        mem.write().limits = new_limits;

        Ok(ptr)
    }

    pub fn handle_impl(&self) -> Result<EntryRef<Func>> {
        let handle_export = match self
            .module
            .exports
            .iter()
            .find(|export| &export.name == "handle")
        {
            Some(export) => export,
            None => {
                bail!("'handle' function is not exported, which is invalid");
            }
        };

        let handle_local = match handle_export.local {
            ExportLocal::Func(ref func_ref) => func_ref,
            _ => bail!("'handle' export is of invalid type, expected function"),
        };

        Ok(handle_local.clone())
    }

    pub fn to_module(self) -> Module {
        self.module
    }
}

pub fn extract(module: parity_wasm::elements::Module) -> Result<parity_wasm::elements::Module> {
    let module = Module::from_elements(&module).with_context(|| "Unable to parse module")?;
    let context = Context::new(module);
    let test_funcs = context.test_funcs();
    let handle_impl = context.handle_impl()?;

    {
        // Block to end borrowing at the end
        let mut handle_func = handle_impl.write();
        match handle_func.origin {
            ImportedOrDeclared::Imported(..) => {
                bail!("'handle' function declared as import, which is invalid");
            }
            ImportedOrDeclared::Declared(ref mut body) => {
                body.locals.clear();
                body.code.clear();

                for test_entry in test_funcs.iter() {
                    body.code.push(Instruction::Call(test_entry.clone()))
                }
            }
        }
    }

    let mut module = context.to_module();

    module
        .exports
        .retain(|export| !export.name.starts_with("test_"));

    let result = module.generate()?;

    Ok(result)
}
