extern crate gtest;
extern crate std;

use self::gtest::{Log, Program, System};
use crate::io::Error;
use crate::service::{Expectation, ExpectedMessage, Fixture, Message, StringIndex};
use gstd::prelude::*;

use crate::io;

const OWNER_1: [u8; 32] =
    hex_literal::hex!("abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a");

const SENDER: [u8; 32] =
    hex_literal::hex!("0a367b92cf0b037dfd89960ee832d56f7fc151681bb41e53690e776f5786998a");

#[test]
fn create() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let _res = program.send(
        OWNER_1,
        io::Init {
            owner: OWNER_1.into(),
            service_address: [0u8; 32].into(),
        },
    );

    let res = program.send(SENDER, io::Control::GetOwner);

    let log = Log::builder()
        .source(program.id())
        .dest(SENDER)
        .payload_bytes(&OWNER_1[..]);

    assert!(res.contains(&log));
}

#[test]
fn service_rest() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let _res = program.send(
        OWNER_1,
        io::Init {
            owner: OWNER_1.into(),
            service_address: [0u8; 32].into(),
        },
    );

    let fixture = Fixture {
        description: StringIndex,
        preparation: vec![],
        expectations: vec![Expectation {
            request: Message {
                gas: 1000000000,
                value: 0,
                payload: b"ping".to_vec(),
            },
            response: ExpectedMessage {
                at_least_gas: None,
                value: Some(0),
                payload: Some(b"pong".to_vec()),
            },
            fail_hint: StringIndex,
        }],
    };
    let _res = program.send(
        SENDER,
        io::Control::AddFixture {
            fixture: fixture.clone(),
        },
    );

    let res = program.send(SENDER, io::Control::GetFixtures);

    let log = Log::builder()
        .source(program.id())
        .dest(SENDER)
        .payload_bytes(vec![fixture].encode());

    assert!(res.contains(&log));

    let _res = program.send(SENDER, io::Control::RemoveFixture { index: 0 });

    let res = program.send(SENDER, io::Control::GetFixtures);

    let log = Log::builder()
        .source(program.id())
        .dest(SENDER)
        .payload_bytes(Vec::<Fixture>::new().encode());

    assert!(res.contains(&log));
}

#[test]
fn service_run_empty() {
    let system = System::new();
    system.init_logger();

    let program = Program::current(&system);
    let _res = program.send(
        OWNER_1,
        io::Init {
            owner: OWNER_1.into(),
            service_address: [0u8; 32].into(),
        },
    );

    // no fixtures should return ok()

    let res = program.send(SENDER, io::Control::RunFixtures);

    let log = Log::builder()
        .source(program.id())
        .dest(SENDER)
        .payload_bytes(Result::<(), Error>::Ok(()).encode());

    assert!(res.contains(&log));

}

#[test]
fn service_run_more() {

}
