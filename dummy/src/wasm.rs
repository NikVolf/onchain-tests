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

use codec::Encode;
use gstd::{msg, prelude::*};
use onchain_test_types::{Expectation, ExpectedMessage, Fixture, Message, StringIndex};

#[no_mangle]
extern "C" fn handle() {
    let payload = msg::load_bytes().expect("Failed to load payload");

    if payload == b"PING" {
        msg::reply_bytes("PONG", 0).expect("Failed to send reply");
    }
}

#[no_mangle]
extern "C" fn test() -> u64 {
    let fixtures = vec![Fixture {
        description: StringIndex,
        preparation: vec![],
        expectations: vec![Expectation {
            request: Message {
                gas: 1_000_000_000,
                value: 0,
                payload: b"PING".to_vec(),
            },
            response: ExpectedMessage {
                at_least_gas: None,
                value: Some(0),
                payload: Some(b"PONG".to_vec()),
            },
            fail_hint: StringIndex,
        }],
    }];

    let fixtures_encoded = fixtures.encode().into_boxed_slice();

    let len = fixtures_encoded.len();
    let ptr = fixtures_encoded.as_ptr();

    let ret = ((ptr as u64) << 32) + (len as u64);

    core::mem::forget(fixtures_encoded);

    ret
}
