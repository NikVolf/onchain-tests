//! Tested service

use gstd::{prelude::*, ActorId};

use codec::{Decode, Encode};

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

/// Service with assigned fixture.
#[derive(Debug, Encode, Decode)]
pub struct Service {
    address: ActorId,
    fixtures: Vec<Fixture>,
}

impl Service {
    pub const fn empty() -> Self {
        Self {
            address: ActorId::new([0u8; 32]),
            fixtures: vec![],
        }
    }

    pub fn new(address: ActorId) -> Self {
        Service {
            address,
            fixtures: vec![],
        }
    }

    pub fn add_fixture(&mut self, fixture: Fixture) {
        self.fixtures.push(fixture);
    }

    pub fn drop_fixture(&mut self, index: usize) -> Fixture {
        self.fixtures.remove(index)
    }

    pub fn fixtures(&self) -> &[Fixture] {
        &self.fixtures
    }

    pub fn fixtures_mut(&mut self) -> &mut [Fixture] {
        &mut self.fixtures
    }

    pub fn clear_fixtures(&mut self) {
        self.fixtures.clear();
    }

    pub fn gas_required(&self) -> u64 {
        self.fixtures().iter().map(|f| f.gas_required()).sum()
    }

    pub fn address(&self) -> ActorId {
        self.address
    }
}
