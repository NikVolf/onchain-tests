#![no_std]

#[cfg(feature = "std")]
mod code {
    include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));
}

#[cfg(feature = "std")]
pub use code::WASM_BINARY_OPT as WASM_BINARY;

use crate::sails::{commands::handlers as c_handlers, Service};
use sails_service::{CompositeService, Service as ServiceTrait};

pub mod io;
mod sails;
mod service;

#[cfg(test)]
mod tests;

#[cfg(not(feature = "std"))]
pub mod wasm {
    use super::{io, service};
    use gstd::{prelude::*, sync::RwLock, ActorId};

    pub static SERVICE: RwLock<service::Service> = RwLock::new(service::Service::empty());
    pub static mut OWNER: Option<ActorId> = None;

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
        let input = gstd::msg::load_bytes().expect("This needs to be handled in some way: read error");
        let (output, is_error) = Service::new(
            |command| Box::pin(c_handlers::process_commands(command)),
            |()| ((), true),
        )
        .process_command(&input)
        .await;

        if is_error {
            unsafe {
                gsys::gr_panic(output.as_ptr(), output.len() as u32);
            }
        }
        gstd::msg::reply(output, 0).expect("This needs to be handled in a consistent way: reply error");
    }

}
