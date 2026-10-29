use horizon_native_api::*;
use horizon_native_runtime::Runtime;
static META: ModMetadata = ModMetadata {
    id: "entities",
    name: "Entities",
    version: "1",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
struct Example(u8);
impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        if self.0 != 1 {
            c.register_block(BlockDescriptor {
                id: "entities:block".parse()?,
                template: "minecraft:stone".parse()?,
                translation_key: "block.name".into(),
            });
        }
        let mut descriptor = BlockEntityDescriptor {
            id: "entities:counter".parse()?,
            block: "entities:block".parse()?,
            version: 1,
            fields: std::collections::BTreeMap::from([("visits".into(), 0)]),
        };
        if self.0 == 2 {
            descriptor.version = 0;
        }
        if self.0 == 3 {
            descriptor.fields.insert("bad field".into(), 0);
        }
        c.register_block_entity(descriptor.clone());
        if self.0 == 4 {
            c.register_block_entity(descriptor);
        }
        Ok(())
    }
}
#[test]
fn versioned_entities_freeze_only_with_valid_owned_blocks_and_unique_schemas() {
    let mut r = Runtime::new(vec![Box::new(Example(0))]).unwrap();
    r.freeze().unwrap();
    assert_eq!(r.content().block_entities.len(), 1);
    for case in 1..=4 {
        let mut r = Runtime::new(vec![Box::new(Example(case))]).unwrap();
        assert!(r.freeze().is_err(), "{case}");
        assert!(r.content().block_entities.is_empty());
    }
}
