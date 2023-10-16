use crate::service;

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId};

#[derive(Debug, Decode, Encode)]
pub enum Control {
    GetOwner,
    ReplaceOwner { new_owner: ActorId },
    GetFixtures,
    RemoveFixture { index: u32 },
    UpdateFixture { index: u32, fixture: service::Fixture },
    AddFixture { fixture: service::Fixture },
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
    pub payload: Vec<u8>,
}

impl<T: Encode> From<T> for Reply {
    fn from(t: T) -> Self {
        Self {
            payload: t.encode(),
        }
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

    pub fn dispatch(&self, control: Control) -> Reply {
        match control {
            Control::GetOwner => self.get_owner().into(),
            _ => unimplemented!(),
        }
    }

    fn get_owner(&self) -> ActorId {
        self.owner.clone()
    }
}
