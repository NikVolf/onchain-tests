use crate::{
    io::{Error, Event, FailType, FailedFixture, FailedFixtures},
    service,
};

use futures::stream::{FuturesUnordered, StreamExt};
use gstd::{msg, prelude::*, sync::RwLock, ActorId};
use sails_macros::command_handlers;
use sails_service::{BoxedFuture, SimpleService};

fn get_svc() -> &'static RwLock<service::Service> {
    &crate::wasm::SERVICE
}

fn static_get_owner() -> &'static mut ActorId {
    unsafe { crate::wasm::OWNER.as_mut().expect("set during initialization") }
}

pub struct CommandProcessorMeta;

impl sails_service::CommandProcessorMeta for CommandProcessorMeta {
    type Request = commands::Commands;
    type Response = commands::CommandResponses;
    type ProcessFn = fn(Self::Request) -> BoxedFuture<(Self::Response, bool)>;
}

pub type Service = SimpleService<CommandProcessorMeta, ()>;

#[command_handlers]
pub mod commands {
    use super::*;

    fn get_owner() -> Result<ActorId, ()> {
        Ok(super::static_get_owner().clone())
    }

    async fn get_fixtures() -> Result<Vec<service::Fixture>, ()> {
        Ok(get_svc().read().await.fixtures().to_vec())
    }

    async fn update_owner(new_owner: ActorId) -> Result<(), Error> {
        if static_get_owner() != &msg::source() {
            return Err(Error::Unauthorized);
        }

        *static_get_owner() = new_owner;

        Ok(())
    }

    async fn remove_fixture(index: u32) -> Result<(), Error> {
        if static_get_owner() != &msg::source() {
            return Err(Error::Unauthorized);
        }

        let mut service = get_svc().write().await;
        if (index as usize) < service.fixtures().len() {
            service.drop_fixture(index as usize);
            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn update_fixture(index: u32, fixture: service::Fixture) -> Result<(), Error> {
        if static_get_owner() != &msg::source() {
            return Err(Error::Unauthorized);
        }

        let mut service = get_svc().write().await;
        if (index as usize) < service.fixtures().len() {
            service.fixtures_mut()[index as usize] = fixture;

            Ok(())
        } else {
            Err(Error::NotFound)
        }
    }

    async fn add_fixture(fixture: service::Fixture) -> Result<(), Error> {
        if static_get_owner() != &msg::source() {
            return Err(Error::Unauthorized);
        }

        get_svc().write().await.add_fixture(fixture);

        Ok(())
    }

    async fn clear_fixtures() -> Result<(), Error> {
        if static_get_owner() != &msg::source() {
            return Err(Error::Unauthorized);
        }

        get_svc().write().await.clear_fixtures();

        Ok(())
    }

    async fn terminate() -> Result<(), Error> {
        if static_get_owner() == &msg::source() {
            gstd::exec::exit(static_get_owner().clone())
        } else {
            Err(Error::Unauthorized)
        }
    }

    async fn run_fixtures() -> Result<(), Error> {
        enum RuntimeError {
            PreparationSendFail(u32),
            ExpectationSendFail(u32),
            ExpectationExecutionFail(u32, gstd::errors::Error),
            PayloadMismatch(u32, Vec<u8>, service::StringIndex),
        }

        let source = msg::source();
        let service = get_svc().read().await;

        let gas_required = service.gas_required();
        let gas_available = gstd::exec::gas_available();
        if gas_available < gas_required {
            return Err(Error::NotEnoughGas {
                actual: gas_available,
                needed: gas_required,
            });
        }

        let mut fails_list: Vec<FailedFixture> = vec![];

        let mut fixtures_stream = FuturesUnordered::new();

        for fixture_no in 0..service.fixtures().len() {
            let ref_svc = &service; // to do only partial move below
            fixtures_stream.push(async move {
                let fixture = &ref_svc.fixtures()[fixture_no];

                // preparations
                for preparation in fixture.preparation.iter() {
                    let _ = match gstd::msg::send_bytes_for_reply(
                        ref_svc.address(),
                        preparation.payload.clone(),
                        0, // TODO: figure out preparation.value,
                        preparation.gas,
                    ) {
                        Ok(fut) => fut,
                        Err(_e) => {
                            return Err(RuntimeError::PreparationSendFail(fixture_no as u32))
                        }
                    }
                    .await; // we don't care about what preparation returns
                }

                // expectations
                for expectation in fixture.expectations.iter() {
                    let result = match gstd::msg::send_bytes_for_reply(
                        ref_svc.address(),
                        expectation.request.payload.clone(),
                        0, // TODO: figure out expectation.request.value,
                        expectation.request.gas,
                    ) {
                        Ok(fut) => fut,
                        Err(_) => return Err(RuntimeError::ExpectationSendFail(fixture_no as u32)),
                    }
                    .await;

                    match result {
                        Ok(payload) => {
                            if let Some(expected_payload) = expectation.response.payload.as_ref() {
                                if expected_payload != &payload[..] {
                                    return Err(RuntimeError::PayloadMismatch(
                                        fixture_no as u32,
                                        payload,
                                        expectation.fail_hint,
                                    ));
                                }
                                // TODO: check gas & value somehow
                            }
                        }
                        Err(e) => {
                            return Err(RuntimeError::ExpectationExecutionFail(
                                fixture_no as u32,
                                e,
                            ));
                        }
                    }
                }

                Ok(fixture_no as u32)
            });
        }

        while let Some(result) = fixtures_stream.next().await {
            match result {
                Ok(index) => {
                    let _ = gstd::msg::send(source, Event::FixtureSuccess { index }, 0);
                }
                Err(
                    RuntimeError::ExpectationSendFail(index)
                    | RuntimeError::ExpectationExecutionFail(index, _),
                ) => {
                    fails_list.push(FailedFixture {
                        index,
                        fail_type: FailType::Execution,
                        fail_hint: None,
                    });
                    let _ = gstd::msg::send(
                        source,
                        Event::FixtureFail {
                            index,
                            fail_hint: None,
                            fail_type: FailType::Execution,
                        },
                        0,
                    );
                }
                Err(RuntimeError::PreparationSendFail(index)) => {
                    fails_list.push(FailedFixture {
                        index,
                        fail_type: FailType::Preparation,
                        fail_hint: None,
                    });
                    let _ = gstd::msg::send(
                        source,
                        Event::FixtureFail {
                            index,
                            fail_hint: None,
                            fail_type: FailType::Preparation,
                        },
                        0,
                    );
                }
                Err(RuntimeError::PayloadMismatch(index, _, fail_hint)) => {
                    fails_list.push(FailedFixture {
                        index,
                        fail_type: FailType::PayloadMismatch,
                        fail_hint: Some(fail_hint),
                    });

                    let _ = gstd::msg::send(
                        source,
                        Event::FixtureFail {
                            index,
                            fail_hint: Some(fail_hint),
                            fail_type: FailType::PayloadMismatch,
                        },
                        0,
                    );
                }
            }
        }

        if fails_list.is_empty() {
            Ok(())
        } else {
            Err(Error::SomeFailed(fails_list.into()))
        }
    }
}
