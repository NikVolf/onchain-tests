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

// thread-local-like variable for run_tests workflow (synchronously populating one big future)
pub static mut CONTEXT_FUTURES: Vec<Pin<Box<dyn Future<Output = ()> + 'static>>> = Vec::new();

pub fn run_tests(ptr: *const u8) {
    // at the moment, just runs all tests

    gstd::message_loop(async move {
        // invoke all declared tests..
        let tests = unsafe { read_tests(ptr) };
        let signal = ControlSignal::current();
        match signal {
            ControlSignal::Test(actor_id) => {
                gstd::debug!("scheduled total {} tests to run...", tests.len());
                let me = gstd::exec::program_id();
                let session_id = sessions::new_session(actor_id).await;
                let mut success_count: u32 = 0;
                let mut fail_count: u32 = 0;

                for test_index in 0..tests.len() {
                    // running tests synchronously
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
                            gstd::debug!("Finished test #{test_index}: success");
                        }
                        Err(e) => {
                            // TODO: report failure
                            fail_count += 1;
                            gstd::debug!("Finished test #{test_index}: fail\nOutput: {e}");
                        }
                    }
                }

                gstd::debug!(
                    "Test session over: {} success, {} failed.",
                    success_count,
                    fail_count
                );
                sessions::drop_session(&session_id).await;
            }
            ControlSignal::WrapExecute(session_id, test_index) => {
                sessions::set_active_session(&session_id).await;

                for (index, test) in tests.into_iter().enumerate() {
                    if index as u32 == test_index {
                        unsafe {
                            test();
                        }
                    }
                }

                let test_future =
                    unsafe { core::mem::replace(&mut CONTEXT_FUTURES, Vec::new()).remove(0) };

                test_future.await;

                msg::reply((), 0).expect("Failed to reply");
            }
        };
    });
}
