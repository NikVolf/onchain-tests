extern crate gtest;
extern crate std;

use self::gtest::{Log, Program, System};
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
