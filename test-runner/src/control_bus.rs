// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public Licensec
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Local(gtest) test results collector for onchain tests

use codec::Decode;
use gtest::WasmProgram;

use gear_test_runtime::{ProgressSignal, TestUpdate};

#[derive(Debug, Default)]
struct ControlBus {
    total_tests: u32,
    total_success: u32,
    total_failed: u32,
}

impl WasmProgram for ControlBus {
    fn init(&mut self, _payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        // does nothing!
        Ok(None)
    }

    fn handle(&mut self, payload: Vec<u8>) -> Result<Option<Vec<u8>>, &'static str> {
        let progress_signal =
            ProgressSignal::decode(&mut &payload[..]).expect("Failed to decode progress signal!");

        let ProgressSignal { test_info, update } = progress_signal;

        match update {
            TestUpdate::Start => {
                self.total_tests += 1;
            }
            TestUpdate::Success => {
                self.total_success += 1;
                println!("Test {}: ok", test_info.name);
            }
            TestUpdate::Fail(hint) => {
                self.total_failed += 1;
                println!("Test {}: fail \nError report: {}", test_info.name, hint);
            }
        }

        Ok(None)
    }

    fn handle_reply(&mut self, _payload: Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }

    fn handle_signal(&mut self, _payload: Vec<u8>) -> Result<(), &'static str> {
        Ok(())
    }

    fn state(&mut self) -> Result<Vec<u8>, &'static str> {
        Ok(vec![])
    }
}
