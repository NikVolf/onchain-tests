extern crate gtest;

use self::gtest::{Log, Program, System};
use gstd::{prelude::*, ActorId};

use crate::io;

const OWNER_1: [u8; 32] =
    hex_literal::hex!("abf3746e72a6e8740bd9e12b879fbdd59e052cb390f116454e9116c22021ae4a");

#[test]
fn smoky() {
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
}
