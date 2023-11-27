#![no_std]

use gstd::{prelude::*, sync::RwLock, ActorId};

pub mod io;
mod service;

pub use service::{Fixture, StringIndex, Expectation, ExpectedMessage, Message};

#[cfg(test)]
mod tests;

static SERVICE: RwLock<service::Service> = RwLock::new(service::Service::empty());
static mut OWNER: Option<ActorId> = None;

#[gstd::async_init]
async fn init() {
    // Configurable owner (in case it might be some control program, this is not taken from msg::source)

    let init: io::Init = gstd::msg::load().expect("failed to read init payload");

    unsafe {
        OWNER = Some(init.owner);

        *SERVICE.write().await = service::Service::new(init.service_address);
    }
}

#[gstd::async_main]
async fn main() {
    let mut handler = io::Handler::new(&SERVICE, unsafe {
        OWNER
            .as_ref()
            .expect("Owner not initialized somehow")
            .clone()
    });
    let request: io::Control = gstd::msg::load().expect("Unable to parse control message");

    let reply = handler.dispatch(request).await;

    if let Some(payload) = reply.payload {
        gcore::msg::reply(&payload[..], 0).expect("Failed to reply");
    }
}
