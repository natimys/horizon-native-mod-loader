use horizon_native_api::{ModContext, ModMetadata, ModSide, NativeMod, API_VERSION};

static META: ModMetadata = ModMetadata {
    id: "hello_mod",
    name: "Hello from native Rust",
    version: "0.1.0",
    description: "First statically linked API-only example mod",
    authors: &["Horizon native modding example"],
    dependencies: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
};

pub struct HelloMod;

impl NativeMod for HelloMod {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn initialize(&mut self, context: &mut ModContext) -> Result<(), String> {
        context.log("HelloMod initialized through the public API");
        Ok(())
    }
    fn shutdown(&mut self, context: &mut ModContext) -> Result<(), String> {
        context.log("HelloMod shutdown through the public API");
        Ok(())
    }
}

pub fn create() -> Box<dyn NativeMod> {
    Box::new(HelloMod)
}
