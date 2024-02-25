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

use anyhow::{anyhow, bail, Context, Result};
use wasm_graph::{EntryRef, ExportLocal, Func, ImportedOrDeclared, Instruction, Module};

pub fn extract(module: parity_wasm::elements::Module) -> Result<parity_wasm::elements::Module> {
    let mut module = Module::from_elements(&module).with_context(|| "Unable to parse module")?;
    let test_funcs: Vec<EntryRef<Func>> = module
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
        .collect();

    let handle_export = match module
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

    {
        /// Block to end borrowing at the end
        let mut handle_func = handle_local.write();
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

    module
        .exports
        .retain(|export| !export.name.starts_with("test_"));

    let result = module.generate()?;

    Ok(result)
}
