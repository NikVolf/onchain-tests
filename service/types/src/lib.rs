#![no_std]
use codec::{Decode, Encode};
use gstd::prelude::*;

// TODO: use metadata-stored static strings once ready
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct StringIndex;

/// Message to be sent or to be expected.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Message {
    pub gas: u64,
    pub value: u128,
    pub payload: Vec<u8>,
}

/// Message to be expected.
#[derive(Clone, Debug, Encode, Decode)]
pub struct ExpectedMessage {
    pub at_least_gas: Option<u64>,
    pub value: Option<u128>,
    pub payload: Option<Vec<u8>>,
}

/// Simple expectation of a particular request.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Expectation {
    pub request: Message,
    pub response: ExpectedMessage,
    pub fail_hint: StringIndex,
}

/// Single set of tests with common setup procedure.
///
/// Setup and run bunch of request with expected responses.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Fixture {
    pub description: StringIndex,
    pub preparation: Vec<Message>,
    pub expectations: Vec<Expectation>,
}

impl Fixture {
    pub fn gas_required(&self) -> u64 {
        self.preparation.iter().map(|p| p.gas).sum::<u64>()
            + self.expectations.iter().map(|e| e.request.gas).sum::<u64>()
    }
}
