// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public Licensec
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Context, Result};
use gear_core::code::ALLOWED_EXPORTS;
use pwasm_utils::parity_wasm::elements::{Module, Serialize as _};
use std::path::PathBuf;

use gear_wasm_builder::PreProcessorTarget;
pub use gear_wasm_builder::WasmBuilder;

struct TestBinaryPreProcessor;

fn clone_and_opt(original_module: &Module) -> Result<Vec<u8>> {
    let mut new_module = original_module.clone();

    pwasm_utils::optimize(&mut new_module, ALLOWED_EXPORTS.to_vec())
        .map_err(|e| anyhow::anyhow!("Optimization error: {:?}!", e))?;

    let mut code = vec![];
    new_module.serialize(&mut code)?;

    Ok(code)
}

impl gear_wasm_builder::PreProcessor for TestBinaryPreProcessor {
    fn name(&self) -> &'static str {
        "test"
    }

    fn pre_process(&self, path: PathBuf) -> Result<Vec<(PreProcessorTarget, Vec<u8>)>> {
        let contents = std::fs::read(&path).context("Failed to read file by optimizer")?;

        let original_module =
            pwasm_utils::parity_wasm::deserialize_buffer(&contents).map_err(|e| {
                anyhow::anyhow!(
                    "Deserialization error for wasm file {0}: {e}!",
                    path.display()
                )
            })?;

        let original_code = clone_and_opt(&original_module)?;

        let module_with_test_runner = wasm_test_extractor::extract(original_module)?;

        let code_with_test_runner = clone_and_opt(&module_with_test_runner)?;

        Ok(vec![
            (PreProcessorTarget::Default, original_code),
            (
                PreProcessorTarget::Named("dummy_wasm.wasm".into()),
                code_with_test_runner,
            ),
        ])
    }
}

pub fn new() -> WasmBuilder {
    WasmBuilder::new()
        .with_pre_processor(Box::new(TestBinaryPreProcessor))
        .exclude_features(vec!["std"])
}
