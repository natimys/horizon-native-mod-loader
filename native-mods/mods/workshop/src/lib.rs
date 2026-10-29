//! Tiny phase 3 acceptance mod. Depends exclusively on the public Mod API.
use horizon_native_api::*;
use std::collections::BTreeMap;
static META: ModMetadata = ModMetadata {
    id: "workshop",
    name: "Native Workshop",
    version: "0.1.0",
    description: "Phase 3 counter, recipe, command and gameplay events",
    authors: &["Native modding example"],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
struct Workshop;
impl NativeMod for Workshop {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        c.register_block(BlockDescriptor {
            id: "workshop:counter".parse()?,
            template: "minecraft:gold_block".parse()?,
            translation_key: "block.workshop.counter".into(),
        });
        c.register_block_entity(BlockEntityDescriptor {
            id: "workshop:counter_data".parse()?,
            block: "workshop:counter".parse()?,
            version: 1,
            fields: BTreeMap::from([("visits".into(), 0)]),
        });
        c.register_recipe(RecipeDescriptor {
            id: "workshop:counter".parse()?,
            ingredients: vec!["minecraft:stick".parse()?, "minecraft:cobblestone".parse()?],
            result: "workshop:counter".parse()?,
            count: 1,
        });
        c.register_command(CommandDescriptor {
            id: "workshop:place".parse()?,
        });
        c.register_tab(CreativeTabDescriptor {
            id: "workshop:tools".parse()?,
            translation_key: "tab.workshop.tools".into(),
            icon: "workshop:counter".parse()?,
            items: vec!["workshop:counter".parse()?],
        });
        c.register_resource(
            "assets/workshop/items/counter.json",
            br#"{"model":{"type":"minecraft:model","model":"minecraft:block/gold_block"}}"#
                .to_vec(),
        );
        c.register_resource("assets/workshop/lang/en_us.json",br#"{"block.workshop.counter":"Native Counter","tab.workshop.tools":"Native Workshop"}"#.to_vec());
        c.register_resource("assets/workshop/lang/ru_ru.json",r#"{"block.workshop.counter":"Нативный счётчик","tab.workshop.tools":"Мастерская Native"}"#.as_bytes().to_vec());
        c.subscribe_gameplay(GameplaySubscriptions {
            players: true,
            world: true,
            blocks: true,
            block_entities: true,
        });
        Ok(())
    }
    fn command(
        &mut self,
        _: &ResourceId,
        p: &PlayerSnapshot,
        c: &mut GameplayContext,
    ) -> Result<(), String> {
        // A visible counter two blocks east of the caller, on the same level.
        let at = [
            p.position[0].floor() as i32 + 2,
            p.position[1].floor() as i32,
            p.position[2].floor() as i32,
        ];
        c.set_block(p.dimension.clone(), at, "workshop:counter");
        c.tell(
            p.id,
            format!("counter queued at {} {} {}", at[0], at[1], at[2]),
        );
        Ok(())
    }
    fn gameplay(&mut self, event: &GameplayEvent, c: &mut GameplayContext) -> Result<(), String> {
        match event {
            GameplayEvent::PlayerJoined(p) => c.tell(p.id, "Workshop joined through public API"),
            GameplayEvent::WorldTick { world, .. } if world.tick == 1 => {}
            GameplayEvent::BlockChanged {
                player,
                state,
                placed,
                ..
            } => c.tell(
                player.id,
                format!(
                    "Workshop {} {state}",
                    if *placed { "placed" } else { "broke" }
                ),
            ),
            GameplayEvent::BlockEntityTick { world, entity }
                if entity.id.as_str() == "workshop:counter_data"
                    && world.tick.is_multiple_of(20) =>
            {
                let mut next = entity.clone();
                next.fields
                    .insert("visits".into(), entity.fields["visits"].saturating_add(1));
                c.set_block_entity(world.dimension.clone(), next);
            }
            _ => {}
        }
        Ok(())
    }
}
pub fn create() -> Box<dyn NativeMod> {
    Box::new(Workshop)
}
