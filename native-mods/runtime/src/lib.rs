//! Deterministic composition and fail-stop lifecycle, independent of the engine.
#![forbid(unsafe_code)]
use horizon_native_api::{
    valid_mod_id, ContentSnapshot, LoadContext, ModAction, ModContext, ModMetadata, ModSide,
    NativeMod, API_VERSION,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod content;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostSide {
    Client,
    Server,
    Combined,
}

impl HostSide {
    fn supports(self, side: ModSide) -> bool {
        side == ModSide::Common
            || self == Self::Combined
            || matches!(
                (self, side),
                (Self::Client, ModSide::Client) | (Self::Server, ModSide::Server)
            )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Resolved,
    Registering,
    Frozen,
    Running,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoaderError {
    InvalidId(String),
    DuplicateId(String),
    IncompatibleApi {
        mod_id: String,
        required: u32,
    },
    MissingDependency {
        mod_id: String,
        dependency: String,
    },
    IncompatibleDependency {
        mod_id: String,
        dependency: String,
        required: String,
        actual: String,
    },
    InactiveDependency {
        mod_id: String,
        dependency: String,
    },
    DependencyCycle(Vec<String>),
    InvalidState(State),
    HookFailed {
        mod_id: String,
        hook: &'static str,
        reason: String,
    },
}
impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "native loader: {self:?}")
    }
}
impl std::error::Error for LoaderError {}

#[derive(Debug, Default)]
pub struct ShutdownReport {
    pub actions: Vec<ModAction>,
    pub errors: Vec<LoaderError>,
}

struct Entry {
    module: Box<dyn NativeMod>,
    // Snapshot once; do not trust a mod to return unchanged metadata later.
    metadata: &'static ModMetadata,
}

/// Immutable compiled catalog entry, including mods excluded from this host's hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompiledMod {
    pub metadata: &'static ModMetadata,
    /// Side selection only; this does not assert successful setup or permit toggling.
    pub selected_for_host: bool,
}

pub struct Runtime {
    mods: Vec<Entry>,
    compiled: Vec<CompiledMod>,
    tick_subscribers: Vec<usize>,
    started: usize,
    side: HostSide,
    state: State,
    content: Arc<ContentSnapshot>,
}

impl Runtime {
    pub fn new(mods: Vec<Box<dyn NativeMod>>) -> Result<Self, LoaderError> {
        Self::for_side(mods, HostSide::Combined)
    }

    /// Validate the whole compiled graph before side selection; ID tie-breaks are stable.
    pub fn for_side(mods: Vec<Box<dyn NativeMod>>, side: HostSide) -> Result<Self, LoaderError> {
        let mut pending = BTreeMap::new();
        for module in mods {
            let metadata = module.metadata();
            let id = metadata.id;
            if !valid_mod_id(id) {
                return Err(LoaderError::InvalidId(id.into()));
            }
            if metadata.api_version != API_VERSION {
                return Err(LoaderError::IncompatibleApi {
                    mod_id: id.into(),
                    required: metadata.api_version,
                });
            }
            if pending.insert(id, Entry { module, metadata }).is_some() {
                return Err(LoaderError::DuplicateId(id.into()));
            }
        }
        for (&id, entry) in &pending {
            for dep in entry.metadata.dependencies {
                if !valid_mod_id(dep.id) {
                    return Err(LoaderError::InvalidId(dep.id.into()));
                }
                let Some(target) = pending.get(dep.id) else {
                    if dep.optional {
                        continue;
                    }
                    return Err(LoaderError::MissingDependency {
                        mod_id: id.into(),
                        dependency: dep.id.into(),
                    });
                };
                if dep.version != target.metadata.version {
                    return Err(LoaderError::IncompatibleDependency {
                        mod_id: id.into(),
                        dependency: dep.id.into(),
                        required: dep.version.into(),
                        actual: target.metadata.version.into(),
                    });
                }
                if side.supports(entry.metadata.side) && !side.supports(target.metadata.side) {
                    return Err(LoaderError::InactiveDependency {
                        mod_id: id.into(),
                        dependency: dep.id.into(),
                    });
                }
            }
        }
        let ids: BTreeSet<_> = pending.keys().copied().collect();
        let mut completed = BTreeSet::new();
        let mut ordered = Vec::new();
        let mut compiled = Vec::new();
        while !pending.is_empty() {
            let ready = pending
                .iter()
                .find(|(_, entry)| {
                    entry
                        .metadata
                        .dependencies
                        .iter()
                        .all(|dep| !ids.contains(dep.id) || completed.contains(dep.id))
                })
                .map(|(&id, _)| id);
            let Some(id) = ready else {
                return Err(LoaderError::DependencyCycle(
                    pending.keys().map(|s| (*s).into()).collect(),
                ));
            };
            let entry = pending.remove(id).expect("ready entry");
            completed.insert(id);
            compiled.push(CompiledMod {
                metadata: entry.metadata,
                selected_for_host: side.supports(entry.metadata.side),
            });
            if side.supports(entry.metadata.side) {
                ordered.push(entry);
            }
        }
        Ok(Self {
            mods: ordered,
            compiled,
            tick_subscribers: Vec::new(),
            started: 0,
            side,
            state: State::Resolved,
            content: Arc::default(),
        })
    }

    /// Active mods for this host, in lifecycle order.
    pub fn metadata(&self) -> impl ExactSizeIterator<Item = &'static ModMetadata> + '_ {
        self.mods.iter().map(|entry| entry.metadata)
    }

    /// Complete validated pack in dependency/ID order, independent of host selection.
    /// Uses the metadata captured during resolution; querying never calls mod code.
    pub fn compiled_mods(&self) -> impl ExactSizeIterator<Item = &CompiledMod> {
        self.compiled.iter()
    }
    pub fn state(&self) -> State {
        self.state
    }
    pub fn has_tick_subscribers(&self) -> bool {
        !self.tick_subscribers.is_empty()
    }

    /// No partial registry is visible before successful global freeze.
    pub fn content(&self) -> &ContentSnapshot {
        &self.content
    }

    /// Register all, freeze, then setup in dependency order. A failure cannot be retried.
    /// Failed-batch actions are discarded; shutdown still visits every started mod.
    pub fn initialize(&mut self) -> Result<Vec<ModAction>, LoaderError> {
        if self.state != State::Resolved {
            return Err(LoaderError::InvalidState(self.state));
        }
        self.state = State::Registering;
        let mut content = ContentSnapshot::default();
        for i in 0..self.mods.len() {
            self.started = i + 1;
            let mut context = LoadContext::default();
            if let Err(reason) = self.mods[i].module.register(&mut context) {
                return Err(self.fail(i, "register", reason));
            }
            if context.has_tick_subscription() {
                self.tick_subscribers.push(i);
            }
            if let Err(reason) = content::collect(&mut content, self.mods[i].metadata.id, context) {
                return Err(self.fail(i, "register", reason));
            }
        }
        if let Err((owner, reason)) = content::validate(&content, self.metadata()) {
            let i = self
                .mods
                .iter()
                .position(|entry| entry.metadata.id == owner)
                .expect("registered owner");
            return Err(self.fail(i, "freeze", reason));
        }
        self.content = Arc::new(content);
        self.state = State::Frozen;
        let mut actions = Vec::new();
        for i in 0..self.mods.len() {
            self.call(i, "initialize", |m, c| m.initialize(c), &mut actions)?;
            let mod_side = self.mods[i].metadata.side;
            if self.side != HostSide::Server && mod_side != ModSide::Server {
                self.call(i, "client_setup", |m, c| m.client_setup(c), &mut actions)?;
            }
            if self.side != HostSide::Client && mod_side != ModSide::Client {
                self.call(i, "server_setup", |m, c| m.server_setup(c), &mut actions)?;
            }
        }
        self.state = State::Running;
        Ok(actions)
    }

    fn fail(&mut self, i: usize, hook: &'static str, reason: String) -> LoaderError {
        self.state = State::Failed;
        LoaderError::HookFailed {
            mod_id: self.mods[i].metadata.id.into(),
            hook,
            reason,
        }
    }

    fn call(
        &mut self,
        i: usize,
        hook: &'static str,
        f: impl FnOnce(&mut dyn NativeMod, &mut ModContext) -> Result<(), String>,
        actions: &mut Vec<ModAction>,
    ) -> Result<(), LoaderError> {
        let mut context = ModContext::with_content(self.mods[i].metadata.id, self.content.clone());
        if let Err(reason) = f(self.mods[i].module.as_mut(), &mut context) {
            return Err(self.fail(i, hook, reason));
        }
        actions.extend(context.finish());
        Ok(())
    }

    /// Host must supply a monotonic tick from its authoritative loop; no world references.
    pub fn tick(&mut self, tick: u64) -> Result<Vec<ModAction>, LoaderError> {
        if self.state != State::Running {
            return Err(LoaderError::InvalidState(self.state));
        }
        let mut actions = Vec::new();
        for j in 0..self.tick_subscribers.len() {
            let i = self.tick_subscribers[j];
            self.call(i, "tick", |m, c| m.tick(tick, c), &mut actions)?;
        }
        Ok(actions)
    }

    /// Reverse dependency order, all cleanup attempted, once, including partial startup.
    pub fn shutdown(&mut self) -> ShutdownReport {
        let mut report = ShutdownReport::default();
        for i in (0..self.started).rev() {
            let mut context =
                ModContext::with_content(self.mods[i].metadata.id, self.content.clone());
            if let Err(reason) = self.mods[i].module.shutdown(&mut context) {
                report.errors.push(LoaderError::HookFailed {
                    mod_id: self.mods[i].metadata.id.into(),
                    hook: "shutdown",
                    reason,
                });
            }
            report.actions.extend(context.finish());
        }
        self.started = 0;
        self.state = State::Stopped;
        report
    }
}

#[cfg(test)]
mod tests;
