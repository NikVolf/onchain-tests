#![no_std]
use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

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

/// Type of fixture execution.
#[derive(Clone, Debug, Encode, Decode)]
pub enum FixtureExecution {
    MessagePassing {
        preparation: Vec<Message>,
        expectations: Vec<Expectation>,
    },
    Call {
        code_hash: ActorId,
        func_id: u32,
        gas: u64,
    },
}

impl FixtureExecution {
    pub fn gas_required(&self) -> u64 {
        use FixtureExecution::*;
        match self {
            MessagePassing {
                preparation,
                expectations,
            } => {
                preparation.iter().map(|p| p.gas).sum::<u64>()
                    + expectations.iter().map(|e| e.request.gas).sum::<u64>()
            }
            Call { gas, .. } => *gas,
        }
    }
}

/// Single set of tests with common setup procedure.
///
/// Setup and run bunch of request with expected responses.
#[derive(Clone, Debug, Encode, Decode)]
pub struct Fixture {
    pub description: StringIndex,
    pub execution: FixtureExecution,
}

impl Fixture {
    pub fn gas_required(&self) -> u64 {
        self.execution.gas_required()
    }
}
