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

fn bytes(wat: &str) -> Vec<u8> {
    wabt::Wat2Wasm::new()
        .convert(wat)
        .expect("failed to parse module")
        .as_ref()
        .to_vec()
}

fn assert_bytes(bytes: &[u8], expected: &[u8]) {
    if bytes != expected {
        // TODO: show wat & diff
        panic!("Bytes not equal!")
    }
}

#[test]
fn simple() {
    let original_bytes = bytes(
        r#"
        (module
            (import "env" "memory" (memory 1))
            (export "handle" (func $handle))
            (export "test_some_test" (func $test_some_test))
            (func $handle)
            (func $test_some_test
                i32.const 0
                drop
            )
        )
    "#,
    );

    let expected_bytes = bytes(
        r#"
        (module
            (import "env" "memory" (memory 1))
            (export "handle" (func $handle))
            (func $handle
                call $test_some_test
            )
            (func $test_some_test
                i32.const 0
                drop
            )
        )
    "#,
    );

    let actual_bytes = super::extract_from_bytes(&original_bytes[..]).expect("Failed to extract");

    assert_bytes(&actual_bytes[..], &expected_bytes[..]);
}
