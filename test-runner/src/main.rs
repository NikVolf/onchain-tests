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

use std::path::Path;
use anyhow::Result;

mod control_bus;

pub fn run_from_bin_path(bin_path_file: impl AsRef<Path>) -> anyhow::Result<()> {
    unimplemented!()

}

pub fn run_from_dir(directory: impl AsRef<Path>) -> anyhow::Result<()> {
    unimplemented!()

}

pub fn run_tests(program_wasm_path: impl AsRef<Path>, progrm_test_path: impl AsRef<Path>) -> anyhow::Result<()> {
    unimplemented!()
}

pub fn main() -> anyhow::Result<()> {
    unimplemented!()
}
