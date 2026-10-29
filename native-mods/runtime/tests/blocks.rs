use horizon_native_api::*;
use horizon_native_runtime::{HostSide, Runtime, State};
static META: ModMetadata = ModMetadata {
    id: "blocks",
    name: "Blocks",
    version: "1",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
struct Producer(u8);
impl NativeMod for Producer {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        let block = BlockDescriptor {
            id: if self.0 == 3 {
                "foreign:tile"
            } else {
                "blocks:tile"
            }
            .parse()?,
            translation_key: "tile.name".into(),
            template: "minecraft:stone".parse()?,
        };
        c.register_block(block.clone());
        if self.0 == 1 {
            c.register_block(block.clone());
        }
        if self.0 == 2 {
            c.register_item(ItemDescriptor {
                id: block.id,
                translation_key: "tile.name".into(),
                template: "minecraft:stick".parse()?,
            });
        }
        c.register_tab(CreativeTabDescriptor {
            id: "blocks:tab".parse()?,
            translation_key: "tab.name".into(),
            icon: "blocks:tile".parse()?,
            items: vec!["blocks:tile".parse()?],
        });
        Ok(())
    }
}
#[test]
fn blocks_freeze_with_takeable_items_and_fail_atomically_on_collisions() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut r = Runtime::for_side(vec![Box::new(Producer(0))], side).unwrap();
        r.freeze().unwrap();
        assert_eq!(r.content().blocks.len(), 1);
        r.initialize().unwrap();
        for case in 1..=3 {
            let mut r = Runtime::for_side(vec![Box::new(Producer(case))], side).unwrap();
            assert!(r.freeze().is_err());
            assert_eq!(r.state(), State::Failed);
            assert!(r.content().blocks.is_empty());
        }
    }
}
