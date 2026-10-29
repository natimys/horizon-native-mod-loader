//! Compile-time composition root. Only this crate names concrete mods.
use horizon_native_runtime::{HostSide, LoaderError, Runtime};

pub fn create_runtime() -> Result<Runtime, LoaderError> {
    create_for_side(HostSide::Combined)
}

/// Composition stays in this crate; the engine only selects its execution role.
pub fn create_for_side(side: HostSide) -> Result<Runtime, LoaderError> {
    let mods = vec![horizon_native_hello_mod::create()];
    #[cfg(feature = "test-pack")]
    let mods = {
        let mut mods = mods;
        mods.push(horizon_native_companion_mod::create());
        mods
    };
    #[cfg(feature = "gameplay-example")]
    let mods = {
        let mut mods = mods;
        mods.push(horizon_native_workshop_mod::create());
        mods
    };
    Runtime::for_side(mods, side)
}
