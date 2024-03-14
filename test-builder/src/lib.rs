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
use gear_wasm_builder::{PreProcessorTarget, WasmBuilder};
use pwasm_utils::parity_wasm::elements::{Module, Serialize as _};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

struct TestBinaryPreProcessor;

fn clone_and_opt(original_module: &Module) -> Result<Vec<u8>> {
    let mut new_module = original_module.clone();

    pwasm_utils::optimize(&mut new_module, ALLOWED_EXPORTS.to_vec())
        .map_err(|e| anyhow::anyhow!("Optimization error: {:?}!", e))?;

    let mut code = vec![];
    new_module.serialize(&mut code)?;

    Ok(code)
}

fn create_and_write_to_file(path: PathBuf, data: &[u8]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

fn create_artifact_path_and_write(
    absolute_path: &PathBuf,
    stem: &str,
    suffix: &str,
    data: &[u8],
) -> std::io::Result<PathBuf> {
    let mut artifact_path = PathBuf::from(absolute_path.clone());
    artifact_path.pop();
    artifact_path.push(format!("{}.{}.wasm", stem, suffix));
    create_and_write_to_file(artifact_path.clone(), data)?;
    Ok(artifact_path)
}

impl gear_wasm_builder::PreProcessor for TestBinaryPreProcessor {
    fn name(&self) -> &'static str {
        "test"
    }

    fn pre_process(&self, path: PathBuf) -> Result<Vec<(PreProcessorTarget, Vec<u8>)>> {
        let contents = std::fs::read(&path).with_context(|| "Failed to read file by optimizer")?;

        let absolute_path = path.canonicalize()?;

        let file_name = path
            .file_name()
            .expect("Path expected to be an actual file")
            .to_str()
            .expect("Filename expected to be convertable to utf-8")
            .to_string();

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

        if let Ok(value) = std::env::var("GEAR_BUILDER_ARTIFACTS") {
            // path without .binpath and stuff

            let stem = absolute_path
                .file_stem()
                .expect("should be a valid str")
                .to_string_lossy();

            let prog_artifact_path = create_artifact_path_and_write(
                &absolute_path,
                stem.as_ref(),
                "opt",
                &original_code[..],
            )?;
            let test_artifact_path = create_artifact_path_and_write(
                &absolute_path,
                stem.as_ref(),
                "test.opt",
                &code_with_test_runner[..],
            )?;

            let record = format!(
                "{}|{}",
                prog_artifact_path.display(),
                test_artifact_path.display()
            );
            let mut file = std::fs::File::options().append(true).open(value.clone())?;
            writeln!(file, "{}", record)?;

            println!("artifacts are dumped at {}:", value);
            println!("{record}");
        } else {
            println!("GEAR_TEST_BUILDER_ARTIFACTS is not set... Probably custom build!");
        }

        Ok(vec![
            (PreProcessorTarget::Default, original_code),
            (PreProcessorTarget::Named(file_name), code_with_test_runner),
        ])
    }
}

pub fn new() -> WasmBuilder {
    WasmBuilder::new()
        .with_pre_processor(Box::new(TestBinaryPreProcessor))
        .exclude_features(vec!["std"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use gear_wasm_builder::PreProcessor;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use wabt::Wat2Wasm;

    #[test]
    fn test_binary_pre_processor() {
        let pre_processor = TestBinaryPreProcessor;
        assert_eq!(pre_processor.name(), "test");

        let wat = r#"
            (module
                (func (export "test_func1") (param i32 i32) (result i32)
                    get_local 0
                    get_local 1
                    i32.add)
                (func (export "test_func2") (param i32 i32) (result i32)
                    get_local 0
                    get_local 1
                    i32.sub)
                (func (export "handle") (param i32) (result i32)
                    get_local 0
                    i32.const 1
                    i32.add)
                (func (export "run_tests") (param i32) (result i32)
                    get_local 0
                    i32.const 1
                    i32.add)
                (memory (export "memory") 1)
                (table (export "table") 1 anyfunc)
            )
        "#;

        let wasm = Wat2Wasm::new().convert(wat).unwrap();

        let mut temp_file = NamedTempFile::new().unwrap();

        temp_file.write_all(wasm.as_ref()).unwrap();

        let path = temp_file.path().to_path_buf();

        let result = pre_processor.pre_process(path.clone());
        assert!(
            result.is_ok(),
            "pre_process failed with error: {:?}",
            result.err()
        );

    }
}
