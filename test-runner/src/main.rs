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

use gear_test_runtime::ControlSignal;
use gtest::{Program, System};
use std::path::{Path, PathBuf};
use anyhow::Context;

mod control_bus;

pub fn run_from_bin_path(bin_path_file: impl AsRef<Path>) -> anyhow::Result<()> {
    let wasm_base = std::fs::read_to_string(bin_path_file.as_ref().to_path_buf())
        .with_context(|| format!("Reading {:?}", bin_path_file.as_ref().to_path_buf()))?;

    let mut bin_base = bin_path_file.as_ref().to_path_buf();
    bin_base.pop();

    let wasm_bin_path = bin_base.join(PathBuf::from(format!("{wasm_base}.opt.wasm")));

    let test_bin_path = bin_base.join(PathBuf::from(format!("{wasm_base}_test.opt.wasm")));

    run_tests(wasm_bin_path, test_bin_path)
}

pub fn run_from_dir(directory: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut path = directory.as_ref().to_path_buf();
    path.push(".binpath");

    run_from_bin_path(path)
}

pub fn run_tests(
    program_wasm_path: impl AsRef<Path>,
    progrm_test_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let system = System::new();
    system.init_logger();

    // test_program
    let test_program = Program::from_file(&system, progrm_test_path);
    let res = test_program.send_bytes(0, vec![]); // empty initialization for test program
    assert!(!res.main_failed());

    // code under test (code_hash)
    let code_hash = system.submit_code(program_wasm_path);

    // control bus program (for results telemetry)
    let control_bus = control_bus::ControlBus::default();
    let running_state = control_bus.running_state();
    let control = Program::mock(&system, control_bus);
    // apparently it also should be initialized
    let res = control.send_bytes(0, vec![]);
    assert!(!res.main_failed());

    // actual test run
    let res = test_program.send(
        0,
        ControlSignal::Test {
            code_hash: code_hash.into_bytes().into(),
            control_bus: control.id().into_bytes().into(),
        },
    );
    assert!(!res.main_failed());

    let report = running_state.read().unwrap().report();
    println!("\n{}", report);

    if !report.success() {
        anyhow::bail!("Some test failed or unfinished!");
    }

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    let actual = if std::env::args().len() <= 1 {
        std::env::current_dir()?
    } else {
        PathBuf::from(
            std::env::args()
                .nth(1)
                .expect("Should exist since args.len() > 1"),
        )
    };

    run_from_dir(actual)
}
