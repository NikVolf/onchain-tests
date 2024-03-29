//! Support library to introduce test runtime to any gear program.
//!
//! Use #![gstd_test::runtime] for your crate root always. Decorate any function under
//! test with #![gstd_test::test] to include it in the test list.
//!
//! In your build, include wasm-test-extractor::PreProcessor in build.rs
//!
//! Compatible only with gstd::async_main entry point, no custom `unsafe handle`-s please!

#![no_std]

use gstd::{prelude::*, ActorId, CodeId, MessageId};

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
    Test {
        code_hash: CodeId,
        control_bus: ActorId,
    },

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
