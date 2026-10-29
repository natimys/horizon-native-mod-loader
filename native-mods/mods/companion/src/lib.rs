//! Optional second independent producer for the two-mod acceptance pack.
use horizon_native_api::{
    CreativeTabDescriptor, ItemDescriptor, LoadContext, ModMetadata, ModSide, NativeMod,
    ResourceId, API_VERSION,
};
static META: ModMetadata = ModMetadata {
    id: "companion",
    name: "Native Companion",
    version: "0.1.0",
    description: "Independent API-only registry acceptance mod",
    authors: &["Native API examples"],
    dependencies: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
};
struct Companion;
impl NativeMod for Companion {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn register(&mut self, context: &mut LoadContext) -> Result<(), String> {
        let item: ResourceId = "companion:marker".parse()?;
        context.register_item(ItemDescriptor {
            id: item.clone(),
            translation_key: "item.companion.marker".into(),
            template: "minecraft:stick".parse()?,
        });
        context.register_tab(CreativeTabDescriptor {
            id: "companion:tools".parse()?,
            translation_key: "tab.companion.tools".into(),
            icon: item.clone(),
            items: vec![item],
        });
        context.register_resource(
            "assets/companion/items/marker.json",
            br#"{"model":{"type":"minecraft:model","model":"companion:item/marker"}}"#.to_vec(),
        );
        context.register_resource("assets/companion/models/item/marker.json", br#"{"parent":"minecraft:builtin/generated","textures":{"layer0":"companion:item/marker"}}"#.to_vec());
        context.register_resource(
            "assets/companion/textures/item/marker.png",
            include_bytes!("../assets/marker.png").to_vec(),
        );
        context.register_resource("assets/companion/lang/en_us.json",br#"{"item.companion.marker":"Companion Marker","tab.companion.tools":"Native Companion"}"#.to_vec());
        context.register_resource(
            "assets/companion/lang/ru_ru.json",
            r#"{"item.companion.marker":"Метка спутника","tab.companion.tools":"Спутник Native"}"#
                .as_bytes()
                .to_vec(),
        );
        Ok(())
    }
}
pub fn create() -> Box<dyn NativeMod> {
    Box::new(Companion)
}
