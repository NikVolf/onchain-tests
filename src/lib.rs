#![no_std]

use gstd::{prelude::*, ActorId};

mod service;
mod io;

#[cfg(test)]
mod tests;

static mut SERVICE: Option<service::Service> = None;
static mut OWNER: Option<ActorId> = None;

#[gstd::async_init]
async fn init() {
    // Configurable owner (in case it might be some control program, this is not taken from msg::source)

    let init: io::Init = gstd::msg::load().expect("failed to read init payload");

    unsafe {
        OWNER = Some(init.owner);
        SERVICE = Some(service::Service::new(init.service_address));
    }
}

#[gstd::async_main]
async fn main() {
    let service = unsafe { SERVICE.as_ref().expect("Service not created somehow!") };
    let handler = io::Handler::new(service, unsafe { OWNER.as_ref().expect("Owner not initialized somehow").clone() });
    let request: io::Control = gstd::msg::load().expect("Unable to parse control message");

    let reply = handler.dispatch(request);

    gcore::msg::reply(&reply.payload[..], 0);
}
