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

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Context, Result};
use pwasm_utils::parity_wasm::elements::Serialize as _;
use std::path::PathBuf;

struct TestBinaryPreProcessor;

impl gear_wasm_builder::PreProcessor for TestBinaryPreProcessor {
    fn pre_process(&self, path: PathBuf) -> Result<gear_wasm_builder::PreProcessOutput> {
        let contents = std::fs::read(&path).context("Failed to read file by optimizer")?;
        let mut module = pwasm_utils::parity_wasm::deserialize_buffer(&contents).map_err(|_| {
            anyhow::anyhow!("Deserialization error for wasm file {0}!", path.display())
        })?;

        pwasm_utils::optimize(&mut module, ["test"].to_vec()).map_err(|_| {
            anyhow::anyhow!("Optimization error for wasm file {0}!", path.display())
        })?;

        let mut code = vec![];
        module.serialize(&mut code)?;

        Ok(gear_wasm_builder::PreProcessOutput {
            content: code,
            path: "dummy-wasm.test.wasm".into(),
        })
    }
}

fn main() {
    gear_wasm_builder::WasmBuilder::new()
        .exclude_features(vec!["std"])
        .with_pre_processor(Box::new(TestBinaryPreProcessor))
        .build();
}
