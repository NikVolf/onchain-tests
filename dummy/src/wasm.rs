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

#[gstd::async_main]
async fn main() {
    let payload = msg::load_bytes().expect("Failed to load payload");

    if payload == b"PING" {
        msg::reply_bytes("PONG", 0).expect("Failed to send reply");
    }
}

#[gear_test_codegen::test]
async fn smoky(context: &gear_test_runtime::TestContext) -> gear_test_runtime::TestResult {
    context.assert(
        msg::send_bytes_for_reply(context.testee().clone(), b"PING", 0, 0)
            .expect("failed to send")
            .await
            .expect("Failed to handle simple PING!")
            == b"PONG",
        "Reply to PING is not PONG!!1".to_string(),
    )
}

#[no_mangle]
pub unsafe extern "C" fn run_tests(ptr: *const u8) {
    gear_test_runtime::run_tests(ptr)
}
