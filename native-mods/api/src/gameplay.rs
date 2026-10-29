//! Owned authoritative snapshots and deferred actions. No borrowed engine data.
use crate::ResourceId;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerSnapshot {
    pub id: u128,
    pub name: String,
    pub dimension: ResourceId,
    pub position: [f64; 3],
    pub health: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldSnapshot {
    pub dimension: ResourceId,
    pub tick: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameplayEvent {
    PlayerJoined(PlayerSnapshot),
    PlayerLeft(PlayerSnapshot),
    /// Once per ticking dimension; players are sorted by UUID.
    WorldTick {
        world: WorldSnapshot,
        players: Vec<PlayerSnapshot>,
    },
    /// Sorted by position within each authoritative dimension tick.
    BlockEntityTick {
        world: WorldSnapshot,
        entity: BlockEntitySnapshot,
    },
    /// Committed player breaking/placing only; queued writes do not recurse.
    BlockChanged {
        player: PlayerSnapshot,
        position: [i32; 3],
        state: String,
        placed: bool,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameplaySubscriptions {
    pub players: bool,
    pub world: bool,
    pub blocks: bool,
    pub block_entities: bool,
}
impl GameplaySubscriptions {
    pub fn accepts(self, event: &GameplayEvent) -> bool {
        match event {
            GameplayEvent::PlayerJoined(_) | GameplayEvent::PlayerLeft(_) => self.players,
            GameplayEvent::WorldTick { .. } => self.world,
            GameplayEvent::BlockChanged { .. } => self.blocks,
            GameplayEvent::BlockEntityTick { .. } => self.block_entities,
        }
    }
    pub fn any(self) -> bool {
        self.players || self.world || self.blocks || self.block_entities
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockEntitySnapshot {
    pub id: ResourceId,
    pub position: [i32; 3],
    pub version: u32,
    pub fields: std::collections::BTreeMap<String, i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameplayAction {
    SetBlockEntity {
        dimension: ResourceId,
        entity: BlockEntitySnapshot,
    },
    Tell {
        player: u128,
        text: String,
    },
    SetBlock {
        dimension: ResourceId,
        position: [i32; 3],
        state: String,
    },
}

/// Owner is supplied by the runtime, never by a callback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedAction {
    pub mod_id: &'static str,
    pub action: GameplayAction,
}

pub const MAX_EVENT_ACTIONS: usize = 256;
pub struct GameplayContext {
    actions: Vec<GameplayAction>,
}
impl Default for GameplayContext {
    fn default() -> Self {
        Self::new()
    }
}
impl GameplayContext {
    pub fn new() -> Self {
        Self { actions: vec![] }
    }
    pub fn tell(&mut self, player: u128, text: impl Into<String>) {
        self.actions.push(GameplayAction::Tell {
            player,
            text: text.into(),
        });
    }
    /// State is a canonical namespaced block name, optionally with properties.
    /// Host resolves it and rejects unloaded positions; it never loads chunks.
    pub fn set_block(
        &mut self,
        dimension: ResourceId,
        position: [i32; 3],
        state: impl Into<String>,
    ) {
        self.actions.push(GameplayAction::SetBlock {
            dimension,
            position,
            state: state.into(),
        });
    }
    pub fn set_block_entity(&mut self, dimension: ResourceId, entity: BlockEntitySnapshot) {
        self.actions
            .push(GameplayAction::SetBlockEntity { dimension, entity });
    }
    pub fn finish(self) -> Result<Vec<GameplayAction>, String> {
        if self.actions.len() > MAX_EVENT_ACTIONS {
            return Err("gameplay action batch exceeds 256 actions".into());
        }
        for action in &self.actions {
            match action {
                GameplayAction::SetBlockEntity { entity, .. }
                    if entity.fields.len() > 64 || entity.version == 0 =>
                {
                    return Err("invalid block entity action".into());
                }
                GameplayAction::Tell { text, .. } if text.len() > 4096 => {
                    return Err("gameplay message exceeds 4096 bytes".into())
                }
                GameplayAction::SetBlock { state, .. } => {
                    if state.len() > 1024 {
                        return Err("block state exceeds 1024 bytes".into());
                    }
                    let name = state.split('[').next().unwrap_or("");
                    name.parse::<ResourceId>()?;
                }
                _ => {}
            }
        }
        Ok(self.actions)
    }
}
