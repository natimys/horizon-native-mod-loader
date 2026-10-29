//! Versioned source API for trusted, statically linked mods. No engine dependencies.
#![forbid(unsafe_code)]

mod content;
mod gameplay;
pub use content::{
    BlockDescriptor, ContentSnapshot, CreativeTabDescriptor, ItemDescriptor, RecipeDescriptor,
    ResourceId,
};
pub use gameplay::*;
use std::sync::Arc;

pub const API_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModSide {
    Common,
    Client,
    Server,
}

/// An exact source-version requirement, deliberately not a partial semver parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dependency {
    pub id: &'static str,
    pub version: &'static str,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModMetadata {
    pub id: &'static str,
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
    pub authors: &'static [&'static str],
    pub license: &'static str,
    pub api_version: u32,
    pub side: ModSide,
    pub dependencies: &'static [Dependency],
}

pub fn valid_mod_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

/// Registration-only services. Consumed before setup; no world or renderer access.
#[derive(Default)]
pub struct LoadContext {
    tick: bool,
    gameplay: GameplaySubscriptions,
    items: Vec<ItemDescriptor>,
    blocks: Vec<BlockDescriptor>,
    recipes: Vec<RecipeDescriptor>,
    tabs: Vec<CreativeTabDescriptor>,
    resources: Vec<(String, Vec<u8>)>,
}

impl LoadContext {
    pub fn subscribe_gameplay(&mut self, subscriptions: GameplaySubscriptions) {
        self.gameplay = subscriptions;
    }
    pub fn gameplay_subscriptions(&self) -> GameplaySubscriptions {
        self.gameplay
    }

    pub fn subscribe_tick(&mut self) {
        self.tick = true;
    }

    pub fn has_tick_subscription(&self) -> bool {
        self.tick
    }

    pub fn register_item(&mut self, item: ItemDescriptor) {
        self.items.push(item);
    }

    pub fn register_block(&mut self, block: BlockDescriptor) {
        self.blocks.push(block);
    }

    pub fn register_recipe(&mut self, recipe: RecipeDescriptor) {
        self.recipes.push(recipe);
    }

    pub fn register_tab(&mut self, tab: CreativeTabDescriptor) {
        self.tabs.push(tab);
    }

    /// Embed original assets/data in the linked binary. Paths use the existing
    /// assets/<mod-id>/... or data/<mod-id>/... resource layout.
    pub fn register_resource(&mut self, path: impl Into<String>, bytes: impl Into<Vec<u8>>) {
        self.resources.push((path.into(), bytes.into()));
    }

    /// Original item/tab extraction contract, retained for existing consumers.
    /// Resource-aware hosts use `into_declarations` instead.
    pub fn into_descriptors(self) -> (Vec<ItemDescriptor>, Vec<CreativeTabDescriptor>) {
        (self.items, self.tabs)
    }

    /// Runtime consumes declarations; nothing is published until global freeze succeeds.
    pub fn into_declarations(self) -> ContentDeclarations {
        ContentDeclarations {
            items: self.items,
            blocks: self.blocks,
            recipes: self.recipes,
            tabs: self.tabs,
            resources: self.resources,
        }
    }
}

/// Unpublished registration batch, consumed only by the runtime.
pub struct ContentDeclarations {
    pub items: Vec<ItemDescriptor>,
    pub blocks: Vec<BlockDescriptor>,
    pub recipes: Vec<RecipeDescriptor>,
    pub tabs: Vec<CreativeTabDescriptor>,
    pub resources: Vec<(String, Vec<u8>)>,
}

/// Owned action queue; the host validates and executes actions after each dispatch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModAction {
    Log { mod_id: String, message: String },
}

pub struct ModContext {
    mod_id: &'static str,
    actions: Vec<ModAction>,
    content: Arc<ContentSnapshot>,
}

impl ModContext {
    pub fn new(mod_id: &'static str) -> Self {
        Self::with_content(mod_id, Arc::default())
    }

    /// Host constructor. Callbacks receive an immutable, frozen snapshot.
    pub fn with_content(mod_id: &'static str, content: Arc<ContentSnapshot>) -> Self {
        Self {
            mod_id,
            actions: Vec::new(),
            content,
        }
    }

    pub fn content(&self) -> &ContentSnapshot {
        &self.content
    }

    pub fn log(&mut self, message: impl Into<String>) {
        self.actions.push(ModAction::Log {
            mod_id: self.mod_id.to_owned(),
            message: message.into(),
        });
    }

    pub fn finish(self) -> Vec<ModAction> {
        self.actions
    }
}

/// Called only by the runtime, never with engine objects. A rebuild adds/removes mods.
pub trait NativeMod: Send {
    fn metadata(&self) -> &'static ModMetadata;
    fn register(&mut self, _context: &mut LoadContext) -> Result<(), String> {
        Ok(())
    }
    fn initialize(&mut self, _context: &mut ModContext) -> Result<(), String> {
        Ok(())
    }
    fn client_setup(&mut self, _context: &mut ModContext) -> Result<(), String> {
        Ok(())
    }
    fn server_setup(&mut self, _context: &mut ModContext) -> Result<(), String> {
        Ok(())
    }
    fn tick(&mut self, _tick: u64, _context: &mut ModContext) -> Result<(), String> {
        Ok(())
    }
    fn gameplay(
        &mut self,
        _event: &GameplayEvent,
        _context: &mut GameplayContext,
    ) -> Result<(), String> {
        Ok(())
    }
    fn shutdown(&mut self, _context: &mut ModContext) -> Result<(), String> {
        Ok(())
    }
}

/// Validated mod identifier. Display names and save content IDs are separate concepts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModId(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidModId(pub String);

impl std::fmt::Display for InvalidModId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid mod ID {:?}: expected [a-z0-9_-]+", self.0)
    }
}
impl std::error::Error for InvalidModId {}

impl std::str::FromStr for ModId {
    type Err = InvalidModId;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if valid_mod_id(value) {
            Ok(Self(value.into()))
        } else {
            Err(InvalidModId(value.into()))
        }
    }
}
impl ModId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for ModId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ids_reject_noncanonical_names_and_roundtrip() {
        for id in ["", "Hello", "x:y", "../x", "hello.mod", "мод", "x\n"] {
            assert!(id.parse::<ModId>().is_err(), "{id:?}");
        }
        let id: ModId = "example_mod-2".parse().unwrap();
        assert_eq!(id.as_str(), "example_mod-2");
        assert_eq!(id.to_string().parse::<ModId>().unwrap(), id);
    }
}
