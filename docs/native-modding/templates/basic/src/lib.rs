//! Rename the package, metadata ID and namespaced content before use.
use horizon_native_api::{ModContext, ModMetadata, ModSide, NativeMod, API_VERSION};

static INFO: ModMetadata = ModMetadata {
    id: "example",
    name: "Example native mod",
    version: "0.1.0",
    description: "An API-only author template",
    authors: &["Your name"],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};

struct Example;
impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        &INFO
    }

    fn initialize(&mut self, context: &mut ModContext) -> Result<(), String> {
        context.log("Example initialized");
        Ok(())
    }
}

pub fn create() -> Box<dyn NativeMod> {
    Box::new(Example)
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_native_api::LoadContext;

    #[test]
    fn registration_is_idle_and_setup_only_queues_owned_actions() {
        let mut module = create();
        let mut registration = LoadContext::default();
        module.register(&mut registration).unwrap();
        assert!(!registration.gameplay_subscriptions().any());
        assert!(!registration.has_tick_subscription());
        let mut context = ModContext::new("example");
        module.initialize(&mut context).unwrap();
        assert_eq!(context.finish().len(), 1);
        // Partial startup cleanup must also work before initialize.
        let mut fresh = create();
        let mut context = ModContext::new("example");
        fresh.shutdown(&mut context).unwrap();
        assert!(context.finish().is_empty());
    }
}
