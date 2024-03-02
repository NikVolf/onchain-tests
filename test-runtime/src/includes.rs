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

use core::{future::Future, pin::Pin};
use futures::{
    stream::{FuturesUnordered, StreamExt},
    FutureExt,
};
use gstd::{msg, prelude::*, ActorId};

#[derive(Debug, codec::Encode)]
pub enum ProgressSignal {
    TestStart(String),
    TestSuccess(String),
}

#[derive(Debug, codec::Decode, codec::Encode)]
pub struct ControlSignal {
    pub deployed_actor: gstd::ActorId,
}

#[derive(Debug)]
pub struct TestContext {
    deployed_actor: gstd::ActorId,
    control_bus: gstd::ActorId,
}

#[derive(Debug)]
pub enum TestResult {
    Ok,
    Fail(String),
}

impl TestContext {
    pub fn current() -> Self {
        let req = msg::load::<ControlSignal>().expect("Failed to decode control signal");

        TestContext {
            deployed_actor: req.deployed_actor,
            control_bus: msg::source(),
        }
    }

    fn send_progress(&self, msg: ProgressSignal) {
        let _ = msg::send(self.control_bus, msg, 0);
    }

    pub fn test_start(&self, name: &str) {
        gstd::debug!("test starts: {}", name);
        self.send_progress(ProgressSignal::TestStart(name.to_string()));
    }

    pub fn test_success(&self, name: &str) {
        gstd::debug!("test success: {}", name);
        self.send_progress(ProgressSignal::TestSuccess(name.to_string()))
    }

    pub fn testee(&self) -> &ActorId {
        &self.deployed_actor
    }

    pub fn assert(&self, cond: bool, fail_hint: String) -> TestResult {
        match cond {
            true => TestResult::Ok,
            false => TestResult::Fail(fail_hint),
        }
    }
}

unsafe fn read_tests(mut ptr: *const u8) -> Vec<unsafe extern "C" fn()> {
    let mut buf = [0u8; 4];
    buf.clone_from_slice(slice::from_raw_parts(ptr, 4));
    let len = u32::from_le_bytes(buf);

    let mut result: Vec<unsafe extern "C" fn()> = Vec::new();

    for i in 0..len {
        ptr = ptr.offset(4);
        buf.clone_from_slice(slice::from_raw_parts(ptr, 4));

        let u32_ptr = u32::from_le_bytes(buf);

        result.push(core::mem::transmute(u32_ptr as usize));
    }

    result
}

// thread-local-like variable for run_tests workflow (synchronously populating one big future)
pub static mut CONTEXT_FUTURES: Vec<Pin<Box<dyn Future<Output = TestResult> + 'static>>> =
    Vec::new();

pub fn run_tests(ptr: *const u8) {
    // at the moment, just runs all tests

    gstd::message_loop(async move {
        // invoke all declared tests..

        let mut stream = unsafe {
            gstd::debug!("reading tests...");
            let tests = read_tests(ptr);
            gstd::debug!("total tests read: {}", tests.len());

            for test in tests {
                test();
            }

            // drain message to local var and create FuturesUnordered
            let mut stream = FuturesUnordered::new();

            gstd::debug!("scheduled total {} tests to run...", CONTEXT_FUTURES.len());
            stream.extend(core::mem::replace(&mut CONTEXT_FUTURES, Vec::new()));

            stream
        };

        while let Some(res) = stream.next().await {
            match res {
                TestResult::Ok => {
                    gstd::debug!("test ok.");
                }
                TestResult::Fail(hint) => {
                    gstd::debug!("test not ok: {}", hint);
                }
            };
        }
    });
}
