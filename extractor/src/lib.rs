//! Wasm test extractor
//!
//! This library transforms binary of this kind:
//!
//! pub unsafe extern "C" handle {
//!     ... some code ...
//! }
//!
//! pub unsafe extern "C" fn test_some_test() {
//!     ... some test code ...
//! }
//!
//! to this binary:
//!
//! pub unsafe extern "C" handle {
//!     test_some_test();
//! }
//!
//! Note that original "... some code ..." is removed

#[cfg(test)]
mod tests;

mod graph;

use anyhow::Result;
use parity_wasm::elements::{Deserialize, Module, Serialize};

pub fn extract_from_bytes(bytes: &[u8]) -> Result<Vec<u8>> {
    let module = parity_wasm::elements::Module::deserialize(&mut &bytes[..])?;
    let mut data = Vec::new();
    parity_wasm::elements::Module::serialize(extract(module)?, &mut data)?;
    Ok(data)
}

pub fn extract(module: Module) -> Result<Module> {
    graph::extract(module)
}
