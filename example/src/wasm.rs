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

use gstd::{msg, prelude::*, ActorId, CodeId};

#[gstd::async_main]
async fn main() {
    let payload = msg::load_bytes().expect("Failed to load payload");

    if payload == b"PING" {
        msg::reply_bytes("PONG", 0).expect("Failed to send reply");
    }
}

async fn create_this(code_hash: &CodeId) -> ActorId {
    let (actor_id, _reply) =
        gstd::prog::ProgramGenerator::create_program_bytes_for_reply(code_hash.clone(), b"PING", 0, 0)
            .expect("Failed to create this/self")
            .await
            .expect("Failed to initialize this/self");

    gstd::debug!("Created! (never enters this line)");

    actor_id
}

#[gear_test_codegen::test]
async fn good(context: &gear_test_runtime::SessionData) {
    let this = create_this(&context.testee()).await;

    let result: Vec<u8> = msg::send_bytes_for_reply(this, b"PING", 0, 0)
        .expect("failed to send")
        .await
        .expect("Program to handle simple PING!!1");

    assert_eq!(result, b"PONG")
}

#[gear_test_codegen::test]
async fn bad(context: &gear_test_runtime::SessionData) {
    let this = create_this(&context.testee()).await;

    let result: Vec<u8> = msg::send_bytes_for_reply(this, b"PING", 0, 0)
        .expect("failed to send")
        .await
        .expect("Program to handle simple PING!!1");

    assert_eq!(result, b"NOTPOING")
}
