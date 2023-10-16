//! Tested service

use gstd::{prelude::*, ActorId};

use codec::{Decode, Encode};

// TODO: use metadata-stored static strings once ready
#[derive(Debug, Encode, Decode)]
pub struct StringIndex;

/// Message to be sent or to be expected.
#[derive(Debug, Encode, Decode)]
pub struct Message {
    pub gas: u64,
    pub value: u128,
    pub payload: Vec<u8>,
}

/// Simple expectation of a particular request.
#[derive(Debug, Encode, Decode)]
pub struct Expectation {
    pub request: Message,
    pub response: Message,
    pub fail_hint: StringIndex,
}

/// Single set of tests with common setup procedure.
///
/// Setup and run bunch of request with expected responses.
#[derive(Debug, Encode, Decode)]
pub struct Fixture {
    pub description: StringIndex,
    pub preparation: Vec<Message>,
    pub expectations: Vec<Expectation>,
}

/// Service with assigned fixture.
#[derive(Debug, Encode, Decode)]
pub struct Service {
    address: ActorId,
    fixtures: Vec<Fixture>,
}

impl Service {
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
}
