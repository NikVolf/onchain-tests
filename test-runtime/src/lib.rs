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

use gstd::{prelude::*, ActorId, MessageId};

mod includes;
mod sessions;

pub use includes::{ContextFuture, TestResult, CONTEXT_FUTURES};
pub use sessions::{active_session, SessionData};

#[derive(Debug, codec::Encode, codec::Decode)]
pub enum TestUpdate {
    Start,
    Success,
    /// contains information about panic / error happened
    Fail(String),
}

#[derive(Debug, codec::Encode, codec::Decode)]
pub struct TestInfo {
    pub index: u32,
    pub name: String,
}

#[derive(Debug, codec::Encode, codec::Decode)]
pub struct ProgressSignal {
    pub test_info: TestInfo,
    pub update: TestUpdate,
}

impl ProgressSignal {
    pub fn new(index: u32, name: String) -> Self {
        ProgressSignal {
            test_info: TestInfo { index, name },
            update: TestUpdate::Start,
        }
    }

    pub fn success(self) -> Self {
        let test_info = self.test_info;

        ProgressSignal {
            test_info,
            update: TestUpdate::Success,
        }
    }

    pub fn fail(self, hint: String) -> Self {
        let test_info = self.test_info;

        ProgressSignal {
            test_info,
            update: TestUpdate::Fail(hint),
        }
    }
}

#[derive(Debug, codec::Decode, codec::Encode)]
pub enum ControlSignal {
    /// Run all tests.
    ///
    /// The only action can be called externally.
    ///
    /// TODO: add test filter
    Test(ActorId),

    /// Execute single test to try catch panic if any.
    ///
    /// Can only be called internally by this actor.
    WrapExecute(MessageId, u32),
}

impl ControlSignal {
    pub fn current() -> Self {
        gstd::msg::load::<ControlSignal>().expect("Failed to decode control signal")
    }
}

#[no_mangle]
pub unsafe extern "C" fn run_tests(ptr: *const u8) {
    includes::run_tests(ptr)
}
