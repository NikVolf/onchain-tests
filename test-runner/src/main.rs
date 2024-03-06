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

use anyhow::Result;
use gear_test_runtime::{ControlSignal, ProgressSignal, TestUpdate};
use gtest::{Program, System};
use std::path::Path;

mod control_bus;

pub fn run_from_bin_path(bin_path_file: impl AsRef<Path>) -> anyhow::Result<()> {
    unimplemented!()
}

pub fn run_from_dir(directory: impl AsRef<Path>) -> anyhow::Result<()> {
    unimplemented!()
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

    // actual program
    let prog = Program::from_file(&system, program_wasm_path);
    // TODO: specify init message for test program!
    let res = prog.send_bytes(0, vec![]);
    assert!(!res.main_failed());

    // actual test run
    let res = test_program.send(0, ControlSignal::Test(prog.id().into_bytes().into()));
    assert!(!res.main_failed());

    Ok(())
}

pub fn main() -> anyhow::Result<()> {
    unimplemented!()
}
