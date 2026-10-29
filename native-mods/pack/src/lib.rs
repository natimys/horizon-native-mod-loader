//! Compile-time composition root. Only this crate names concrete mods.
use horizon_native_runtime::{HostSide, LoaderError, Runtime};

pub fn create_runtime() -> Result<Runtime, LoaderError> {
    Runtime::new(vec![horizon_native_hello_mod::create()])
}

/// Composition stays in this crate; the engine only selects its execution role.
pub fn create_for_side(side: HostSide) -> Result<Runtime, LoaderError> {
    Runtime::for_side(vec![horizon_native_hello_mod::create()], side)
}
