use horizon_native_api::*;
use horizon_native_runtime::{Runtime, State};
static META: ModMetadata = ModMetadata {
    id: "recipes",
    name: "Recipes",
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
        let mut recipe = RecipeDescriptor {
            id: "recipes:tile".parse()?,
            ingredients: vec!["minecraft:stone".parse()?],
            result: "minecraft:stick".parse()?,
            count: 2,
        };
        match self.0 {
            1 => recipe.ingredients.clear(),
            2 => recipe.count = 0,
            3 => recipe.result = "missing:tile".parse()?,
            4 => recipe.id = "recipes:nested/tile".parse()?,
            _ => {}
        }
        c.register_recipe(recipe.clone());
        if self.0 == 5 {
            c.register_recipe(recipe);
        }
        Ok(())
    }
}
#[test]
fn recipe_freeze_rejects_invalid_counts_references_and_duplicates() {
    let mut r = Runtime::new(vec![Box::new(Producer(0))]).unwrap();
    r.initialize().unwrap();
    assert_eq!(r.content().recipes.len(), 1);
    for case in 1..=5 {
        let mut r = Runtime::new(vec![Box::new(Producer(case))]).unwrap();
        assert!(r.freeze().is_err(), "case {case}");
        assert_eq!(r.state(), State::Failed);
        assert!(r.content().recipes.is_empty());
    }
}
