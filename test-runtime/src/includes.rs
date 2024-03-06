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

use super::ControlSignal;
use core::{future::Future, pin::Pin};
use gstd::{msg, prelude::*};

use crate::sessions;

#[derive(Debug)]
pub enum TestResult {
    Ok,
    Fail(String),
}

unsafe fn read_tests(mut ptr: *const u8) -> Vec<unsafe extern "C" fn()> {
    let mut buf = [0u8; 4];
    buf.clone_from_slice(slice::from_raw_parts(ptr, 4));
    let len = u32::from_le_bytes(buf);

    let mut result: Vec<unsafe extern "C" fn()> = Vec::new();

    for _ in 0..len {
        ptr = ptr.offset(4);
        buf.clone_from_slice(slice::from_raw_parts(ptr, 4));

        let u32_ptr = u32::from_le_bytes(buf);

        result.push(core::mem::transmute(u32_ptr as usize));
    }

    result
}

pub struct ContextFuture {
    fut: Pin<Box<dyn Future<Output = ()> + 'static>>,
    name: &'static str,
}

impl ContextFuture {
    pub fn new(
        fut: impl future::Future<Output = ()> + 'static + gstd::Send,
        name: &'static str,
    ) -> Self {
        use futures::FutureExt;
        ContextFuture {
            fut: fut.boxed(),
            name,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn into_future(self) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
        self.fut
    }
}

fn extract_test_context(ptr: *const u8, index: u32) -> ContextFuture {
    unsafe {
        let tests = read_tests(ptr);
        let test_by_index = tests[index as usize];
        test_by_index();

        let test_future = core::mem::replace(&mut CONTEXT_FUTURES, Vec::new()).remove(0);

        test_future
    }
}

fn extract_test_names(ptr: *const u8) -> Vec<&'static str> {
    unsafe {
        let tests = read_tests(ptr);
        for test in tests {
            test()
        }
        core::mem::replace(&mut CONTEXT_FUTURES, Vec::new())
            .into_iter()
            .map(|con_fut| con_fut.name())
            .collect()
    }
}

// thread-local-like variable for run_tests workflow (synchronously populating one big future)
pub static mut CONTEXT_FUTURES: Vec<ContextFuture> = Vec::new();

pub fn run_tests(ptr: *const u8) {
    // at the moment, just runs all tests

    gstd::message_loop(async move {
        // invoke all declared tests..
        let signal = ControlSignal::current();
        match signal {
            ControlSignal::Test(actor_id) => {
                let me = gstd::exec::program_id();
                let (session_id, active_session) = sessions::new_session(actor_id).await;
                let mut success_count: u32 = 0;
                let mut fail_count: u32 = 0;
                let test_names = extract_test_names(ptr);
                let test_count = test_names.len() as u32;
                gstd::debug!("scheduled total {test_count} tests to run...");

                for test_index in 0..test_count {
                    // running tests synchronously

                    let test_name = test_names[test_index as usize];
                    active_session.test_start(test_index, test_name);

                    let test_result = msg::send_for_reply(
                        me,
                        ControlSignal::WrapExecute(session_id.clone(), test_index as u32),
                        0,
                        0,
                    )
                    .expect("Failed to send message")
                    .await;

                    match test_result {
                        Ok(_) => {
                            // TODO: report success
                            success_count += 1;
                            active_session.test_success(test_index, test_name);
                            gstd::debug!("Finished test #{test_index}: success");
                        }
                        Err(e) => {
                            // TODO: report failure
                            fail_count += 1;
                            active_session.test_fail(
                                test_index,
                                test_name,
                                gstd::string::ToString::to_string(&e),
                            );
                            gstd::debug!("Finished test #{test_index}: fail\nOutput: {e}");
                        }
                    }
                }

                gstd::debug!(
                    "Test session over: {} success, {} failed, {} total.",
                    success_count,
                    fail_count,
                    test_count,
                );
                sessions::drop_session(&session_id).await;
            }
            ControlSignal::WrapExecute(session_id, test_index) => {
                sessions::set_active_session(&session_id).await;

                // TODO: make sure it is obvious that only one is used?
                let test_future = extract_test_context(ptr, test_index);
                test_future.into_future().await;

                msg::reply((), 0).expect("Failed to reply");
            }
        };
    });
}
