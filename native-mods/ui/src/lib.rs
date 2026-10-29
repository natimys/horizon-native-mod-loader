//! Read-only native mod list/details adapter. No engine, widgets or mod callbacks.
//! The host supplies these snapshots to its own client presentation; server hosts
//! receive no view. Capturing a view never starts mods or changes runtime settings.
#![forbid(unsafe_code)]

use horizon_native_api::{Dependency, ModMetadata};
use horizon_native_runtime::{HostSide, Runtime, State};

/// Distinguishes this path from sandboxed WASM and source-mixin native mods.
pub const LOADING_MODEL: &str = "Native Rust (statically linked)";
pub const REBUILD_NOTICE: &str =
    "Adding, removing or disabling a linked mod requires rebuilding the executable.";
pub const TRUST_NOTICE: &str =
    "Native mods run with full process privileges and are not sandboxed.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyStatus {
    /// The validated compiled graph contains the exact required source version.
    Present {
        selected_for_host: bool,
    },
    OptionalAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyDetails {
    pub requirement: Dependency,
    pub status: DependencyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModDetails {
    /// Includes name, ID, version, description, authors, license, API and side.
    pub metadata: &'static ModMetadata,
    /// Host-side selection, not a claim that setup completed or the mod is running.
    pub selected_for_host: bool,
    pub dependencies: Vec<DependencyDetails>,
}

/// One client snapshot in deterministic dependency/ID order. No toggle/settings
/// callbacks: mutable configuration needs its own future public API contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModListView {
    state: State,
    entries: Vec<ModDetails>,
}

impl ModListView {
    /// No view is allocated on dedicated/integrated server runtimes. Combined
    /// hosts simulate in-frame and still have a client presentation.
    pub fn for_runtime(runtime: &Runtime) -> Option<Self> {
        if runtime.host_side() == HostSide::Server {
            return None;
        }
        let compiled: std::collections::BTreeMap<_, _> = runtime
            .compiled_mods()
            .map(|entry| (entry.metadata.id, entry))
            .collect();
        let entries = runtime
            .compiled_mods()
            .map(|entry| ModDetails {
                metadata: entry.metadata,
                selected_for_host: entry.selected_for_host,
                dependencies: entry
                    .metadata
                    .dependencies
                    .iter()
                    .map(|requirement| DependencyDetails {
                        requirement: *requirement,
                        // Missing required dependencies are rejected by Runtime
                        // before a catalog can exist; only optional ones can be absent.
                        status: compiled.get(requirement.id).map_or(
                            DependencyStatus::OptionalAbsent,
                            |target| DependencyStatus::Present {
                                selected_for_host: target.selected_for_host,
                            },
                        ),
                    })
                    .collect(),
            })
            .collect();
        Some(Self {
            state: runtime.state(),
            entries,
        })
    }

    /// State of the whole runtime at capture, not a per-mod diagnostic.
    pub fn runtime_state(&self) -> State {
        self.state
    }

    pub fn entries(&self) -> &[ModDetails] {
        &self.entries
    }

    /// Selection uses the stable mod ID, never a transient screen row index.
    pub fn details(&self, id: &str) -> Option<&ModDetails> {
        self.entries.iter().find(|entry| entry.metadata.id == id)
    }
}
