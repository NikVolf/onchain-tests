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

    fn name(&self) -> &'static str {
        "test"
    }

    fn pre_process(&self, path: PathBuf) -> Result<Vec<(String, Vec<u8>)>> {
        let contents = std::fs::read(&path).context("Failed to read file by optimizer")?;

        let module = pwasm_utils::parity_wasm::deserialize_buffer(&contents).map_err(|e| {
            anyhow::anyhow!("Deserialization error for wasm file {0}: {e}!", path.display())
        })?;

        let mut module = wasm_test_extractor::extract(module)?;

        pwasm_utils::optimize(
            &mut module,
            ["handle", "handle_reply", "handle_signal"].to_vec(),
        )
        .map_err(|_| anyhow::anyhow!("Optimization error for wasm file {0}!", path.display()))?;

        let mut code = vec![];
        module.serialize(&mut code)?;

        Ok(vec![("test.wasm".into(), code)])
    }
}

fn main() {
    gear_wasm_builder::WasmBuilder::new()
        .with_pre_processor(Box::new(TestBinaryPreProcessor))
        .exclude_features(vec!["std"])
        .build();
}
