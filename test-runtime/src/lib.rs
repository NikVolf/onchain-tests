// This file is part of Gear.

// Copyright (C) 2023 Gear Technologies Inc.
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

//! Support library to introduce test runtime to any gear program.
//!
//! Use #![gstd_test::runtime] for your crate root always. Decorate any function under
//! test with #![gstd_test::test] to include it in the test list.
//!
//! In your build, include wasm-test-extractor::PreProcessor in build.rs
//!
//! Compatible only with gstd::async_main entry point, no custom `unsafe handle`-s please!

#![no_std]

use gstd::prelude::*;

mod includes;
pub use includes::{TestContext, TestResult, CONTEXT_FUTURES};

#[derive(Debug, codec::Encode)]
pub enum ProgressSignal {
    TestStart(String),
    TestSuccess(String),
    TestFail(String, String),
}

#[derive(Debug, codec::Decode, codec::Encode)]
pub struct ControlSignal {
    pub deployed_actor: gstd::ActorId,
}

#[no_mangle]
pub unsafe extern "C" fn run_tests(ptr: *const u8) {
    includes::run_tests(ptr)
}

/// Help with dependencies in client code (instead of `async { }.boxed()`)
pub fn box_test_future(
    async_block: impl future::Future<Output = ()> + 'static + gstd::Send,
) -> core::pin::Pin<Box<dyn future::Future<Output = ()> + 'static>> {
    use futures::FutureExt;

    async_block.boxed()
}
