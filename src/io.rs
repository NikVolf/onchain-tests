use crate::service;

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Decode, Encode)]
pub enum Control {
    GetOwner,
    ReplaceOwner {
        new_owner: ActorId,
    },
    GetFixtures,
    RemoveFixture {
        index: u32,
    },
    UpdateFixture {
        index: u32,
        fixture: service::Fixture,
    },
    AddFixture {
        fixture: service::Fixture,
    },
    ClearFixtures,
    RunFixtures,
}

#[derive(Debug, Decode, Encode)]
pub enum Error {
    NotFound,
    NotEnoughGas,
}

#[derive(Debug, Decode, Encode)]
pub struct Init {
    pub owner: ActorId,
    pub service_address: ActorId,
}

#[derive(Debug)]
pub struct Reply {
    pub payload: Option<Vec<u8>>,
}

impl<T: Encode> From<T> for Reply {
    fn from(t: T) -> Self {
        Reply {
            payload: Some(t.encode()),
        }
    }
}

impl Reply {
    pub fn none() -> Self {
        Reply { payload: None }
    }
}

pub struct Handler<'a> {
    service: &'a mut service::Service,
    owner: ActorId,
}

#[derive(Debug, Decode, Encode)]
pub struct FailedFixtures {
    pub indices: Vec<(u32, service::StringIndex)>,
}

impl<'a> Handler<'a> {
    pub fn new(service: &'a mut service::Service, owner: ActorId) -> Self {
        Self { service, owner }
    }

    pub fn dispatch(&mut self, control: Control) -> Reply {
        use Control::*;
        match control {
            GetOwner => self.get_owner().into(),
            ReplaceOwner { new_owner } => {
                self.owner = new_owner;
                Reply::none()
            }
            GetFixtures => self.get_fixtures().into(),
            RemoveFixture { index } => self.remove_fixture(index).into(),
            UpdateFixture { index, fixture } => self.update_fixture(index, fixture).into(),
            AddFixture { fixture } => {
                self.add_fixture(fixture);
                Reply::none()
            }
            ClearFixtures => {
                self.clear_fixtures();
                Reply::none()
            }
            RunFixtures => self.run_fixtures().into(),
        }
    }

    fn get_owner(&self) -> ActorId {
        self.owner.clone()
    }

    fn get_fixtures(&self) -> Vec<service::Fixture> {
        self.service.fixtures().to_vec()
    }

    fn remove_fixture(&mut self, index: u32) -> Result<(), Error> {
        if (index as usize) < self.service.fixtures().len() {
            self.service.drop_fixture(index as usize);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    fn update_fixture(&mut self, index: u32, fixture: service::Fixture) -> Result<(), Error> {
        if (index as usize) < self.service.fixtures().len() {
            self.service.fixtures_mut()[index as usize] = fixture;

            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    fn add_fixture(&mut self, fixture: service::Fixture) {
        self.service.add_fixture(fixture);
    }

    fn clear_fixtures(&mut self) {
        self.service.clear_fixtures();
    }

    fn run_fixtures(&self) -> Result<FailedFixtures, Error> {
        unimplemented!()
    }
}
