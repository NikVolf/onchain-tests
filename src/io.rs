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
    service: &'a service::Service,
    owner: ActorId,
}

impl<'a> Handler<'a> {
    pub fn new(service: &'a service::Service, owner: ActorId) -> Self {
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
            _ => unimplemented!(),
        }
    }

    fn get_owner(&self) -> ActorId {
        self.owner.clone()
    }

    fn get_fixtures(&self) -> Vec<service::Fixture> {
        vec![]
    }
}
