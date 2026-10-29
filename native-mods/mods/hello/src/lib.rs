use horizon_native_api::{
    CreativeTabDescriptor, ItemDescriptor, LoadContext, ModContext, ModMetadata, ModSide,
    NativeMod, API_VERSION,
};

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
    fn register(&mut self, context: &mut LoadContext) -> Result<(), String> {
        let item: horizon_native_api::ResourceId = "hello_mod:token".parse()?;
        context.register_item(ItemDescriptor {
            id: item.clone(),
            translation_key: "item.hello_mod.token".into(),
            template: "minecraft:stick".parse()?,
        });
        context.register_tab(CreativeTabDescriptor {
            id: "hello_mod:hello".parse()?,
            translation_key: "tab.hello_mod.hello".into(),
            icon: item.clone(),
            items: vec![item],
        });
        context.register_resource(
            "assets/hello_mod/items/token.json",
            br#"{"model":{"type":"minecraft:model","model":"hello_mod:item/token"}}"#.to_vec(),
        );
        context.register_resource("assets/hello_mod/models/item/token.json", br#"{"parent":"minecraft:builtin/generated","textures":{"layer0":"hello_mod:item/token"}}"#.to_vec());
        context.register_resource(
            "assets/hello_mod/textures/item/token.png",
            include_bytes!("../assets/token.png").to_vec(),
        );
        context.register_resource(
            "assets/hello_mod/lang/en_us.json",
            br#"{"item.hello_mod.token":"Hello Token","tab.hello_mod.hello":"Hello Native"}"#
                .to_vec(),
        );
        context.register_resource("assets/hello_mod/lang/ru_ru.json", r#"{"item.hello_mod.token":"Приветственный жетон","tab.hello_mod.hello":"Привет, Native"}"#.as_bytes().to_vec());
        Ok(())
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
