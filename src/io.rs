use crate::service;

use codec::{Decode, Encode};
use gstd::{prelude::*, ActorId, sync::RwLock};

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
    service: &'a RwLock<service::Service>,
    owner: ActorId,
}

#[derive(Debug, Decode, Encode)]
pub struct FailedFixtures {
    pub indices: Vec<(u32, service::StringIndex)>,
}

impl<'a> Handler<'a> {
    pub fn new(service: &'a RwLock<service::Service>, owner: ActorId) -> Self {
        Self { service, owner }
    }

    pub async fn dispatch(&mut self, control: Control) -> Reply {
        use Control::*;
        match control {
            GetOwner => self.get_owner().into(),
            ReplaceOwner { new_owner } => {
                self.owner = new_owner;
                Reply::none()
            }
            GetFixtures => self.get_fixtures().await.into(),
            RemoveFixture { index } => self.remove_fixture(index).await.into(),
            UpdateFixture { index, fixture } => self.update_fixture(index, fixture).await.into(),
            AddFixture { fixture } => {
                self.add_fixture(fixture).await;
                Reply::none()
            }
            ClearFixtures => {
                self.clear_fixtures().await;
                Reply::none()
            }
            RunFixtures => self.run_fixtures().await.into(),
        }
    }

    fn get_owner(&self) -> ActorId {
        self.owner.clone()
    }

    async fn get_fixtures(&self) -> Vec<service::Fixture> {
        self.service.read().await.fixtures().to_vec()
    }

    async fn remove_fixture(&mut self, index: u32) -> Result<(), Error> {
        let mut service = self.service.write().await;
        if (index as usize) < service.fixtures().len() {
            service.drop_fixture(index as usize);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn update_fixture(&mut self, index: u32, fixture: service::Fixture) -> Result<(), Error> {
        let mut service = self.service.write().await;

        if (index as usize) < service.fixtures().len() {
            service.fixtures_mut()[index as usize] = fixture;

            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn add_fixture(&mut self, fixture: service::Fixture) {
        self.service.write().await.add_fixture(fixture);
    }

    async fn clear_fixtures(&mut self) {
        self.service.write().await.clear_fixtures();
    }

    async fn run_fixtures(&self) -> Result<FailedFixtures, Error> {
        unimplemented!()
    }
}
