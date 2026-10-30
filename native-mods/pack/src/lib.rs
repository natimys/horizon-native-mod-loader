//! Generated static composition. Edit native-modpack.toml, then regenerate.
use horizon_native_runtime::{HostSide, LoaderError, Runtime};

pub fn create_runtime() -> Result<Runtime, LoaderError> {
    create_for_side(HostSide::Combined)
}

/// The engine selects a host role; only this crate names concrete mods.
pub fn create_for_side(side: HostSide) -> Result<Runtime, LoaderError> {
    Runtime::for_side(
        vec![
            #[cfg(feature = "test-pack")]
            horizon_native_companion_mod::create(),
            horizon_native_hello_mod::create(),
            #[cfg(feature = "gameplay-example")]
            horizon_native_workshop_mod::create(),
        ],
        side,
    )
}
