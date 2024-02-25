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

//! Wasm test extractor
//!
//! This library transforms binary of this kind:
//!
//! pub unsafe extern "C" handle {
//!     ... some code ...
//! }
//!
//! pub unsafe extern "C" fn test_some_test() {
//!     ... some test code ...
//! }
//!
//! to this binary:
//!
//! pub unsafe extern "C" handle {
//!     test_some_test();
//! }
//!
//! Note that original "... some code ..." is removed

#[cfg(test)]
mod tests;

use anyhow::Result;
use parity_wasm::elements::Module;

pub fn extract_from_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    unimplemented!()
}

pub fn extract(module: Module) -> Result<Module> {
    unimplemented!()
}
