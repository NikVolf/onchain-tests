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
    let actual_wat = wasmprinter::print_bytes(bytes).expect("Failed to convert actual wasm to wat");
    let expected_wat =
        wasmprinter::print_bytes(expected).expect("Failed to convert result wasm to wat");

    if actual_wat != expected_wat {
        println!("Error: wasms don't match!");
        for diff in diff::lines(&expected_wat, &actual_wat) {
            match diff {
                diff::Result::Left(l) => println!("-{}", l),
                diff::Result::Both(l, _) => println!(" {}", l),
                diff::Result::Right(r) => println!("+{}", r),
            }
        }

        panic!()
    }
}

#[test]
fn simple() {
    let original_bytes = bytes(
        r#"
        (module
            (type (;0;) (func))
            (type (;1;) (func (param i32)))
            (import "env" "memory" (memory 1))
            (table 1 1 funcref)
            (export "handle" (func 0))
            (export "run_tests" (func 3))
            (export "test_some_test" (func 1))
            (export "test_another_test" (func 2))
            (elem (i32.const 0) func 0)
            (func (;0;))
            (func (;1;)
                i32.const 0
                drop
            )
            (func (;2;)
                i32.const 2
                drop
            )
            (func (type 1) (;3;)
                i32.const 4
                drop
            )
        )
    "#,
    );

    let expected_bytes = bytes(
        r#"
        (module
            (type (;0;) (func))
            (type (;1;) (func (param i32)))
            (import "env" "memory" (memory (;0;) 2))
            (func (;0;) (type 0)
              i32.const 65536
              call 3
            )
            (func (;1;) (type 0)
              i32.const 0
              drop
            )
            (func (;2;) (type 0)
              i32.const 2
              drop
            )
            (func (type 1) (;3;)
                i32.const 4
                drop
            )
            (table (;0;) 3 3 funcref)
            (export "handle" (func 0))
            (elem (;0;) (i32.const 0) func 0)
            (elem (;1;) (i32.const 1) func 1 2)
            (data (;0;) (i32.const 65536) "\02\00\00\00\00\00\00\00\01\00\00\00\02\00\00\00")
          )
    "#,
    );

    let actual_bytes = super::extract_from_bytes(&original_bytes[..]).expect("Failed to extract");

    assert_bytes(&actual_bytes[..], &expected_bytes[..]);
}
