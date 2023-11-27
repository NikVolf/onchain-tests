//! Tested service

use gstd::{prelude::*, ActorId};

pub use onchain_test_types::{Expectation, ExpectedMessage, Fixture, Message, StringIndex};

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
