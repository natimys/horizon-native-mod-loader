//! Independent API-only mods; no assets, renderer or private engine dependencies.
use horizon_native_api::{Dependency, LoadContext, ModMetadata, ModSide, NativeMod, API_VERSION};
use horizon_native_runtime::{HostSide, Runtime, State};
use horizon_native_ui::{
    DependencyStatus, ModListView, LOADING_MODEL, REBUILD_NOTICE, TRUST_NOTICE,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

static ALPHA: ModMetadata = ModMetadata {
    id: "alpha",
    name: "Альфа",
    version: "1.2.3",
    description: "A common mod with a localized display name",
    authors: &["Author A", "Author B"],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
static BETA: ModMetadata = ModMetadata {
    id: "beta",
    name: "Beta",
    dependencies: &[
        Dependency {
            id: "alpha",
            version: "1.2.3",
            optional: false,
        },
        Dependency {
            id: "absent",
            version: "1",
            optional: true,
        },
    ],
    ..ALPHA
};
static SERVER: ModMetadata = ModMetadata {
    id: "server",
    side: ModSide::Server,
    ..BETA
};

struct Example(&'static ModMetadata, Arc<AtomicUsize>);
impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        self.0
    }
    fn register(&mut self, _: &mut LoadContext) -> Result<(), String> {
        self.1.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
fn runtime(side: HostSide, reverse: bool, calls: &Arc<AtomicUsize>) -> Runtime {
    let mut mods: Vec<Box<dyn NativeMod>> = [&BETA, &SERVER, &ALPHA]
        .into_iter()
        .map(|m| Box::new(Example(m, calls.clone())) as Box<dyn NativeMod>)
        .collect();
    if reverse {
        mods.reverse();
    }
    Runtime::for_side(mods, side).unwrap()
}

#[test]
fn both_mods_and_inactive_server_mod_have_complete_deterministic_details() {
    let calls = Arc::default();
    for side in [HostSide::Client, HostSide::Combined] {
        let a = runtime(side, false, &calls);
        let b = runtime(side, true, &calls);
        let view = ModListView::for_runtime(&a).unwrap();
        assert_eq!(view, ModListView::for_runtime(&b).unwrap());
        assert_eq!(
            view.entries()
                .iter()
                .map(|m| m.metadata.id)
                .collect::<Vec<_>>(),
            ["alpha", "beta", "server"]
        );
        assert_eq!(view.runtime_state(), State::Resolved);
        assert_eq!(view.details("alpha").unwrap().metadata, &ALPHA);
        assert_eq!(view.details("alpha").unwrap().metadata.name, "Альфа");
        assert!(view.details("alpha").unwrap().dependencies.is_empty());
        assert!(view.details("unknown").is_none());
        assert!(view.details("beta").unwrap().selected_for_host);
        assert_eq!(
            view.details("server").unwrap().selected_for_host,
            side == HostSide::Combined
        );
        let deps = &view.details("beta").unwrap().dependencies;
        assert_eq!(deps[0].requirement, BETA.dependencies[0]);
        assert_eq!(
            deps[0].status,
            DependencyStatus::Present {
                selected_for_host: true
            }
        );
        assert_eq!(deps[1].status, DependencyStatus::OptionalAbsent);
    }
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "reading UI data never calls registration"
    );
    assert!(LOADING_MODEL.contains("statically linked"));
    assert!(REBUILD_NOTICE.contains("rebuilding"));
    assert!(TRUST_NOTICE.contains("not sandboxed"));
}

#[test]
fn server_has_no_ui_view_or_ui_triggered_callbacks() {
    let calls = Arc::default();
    let mut runtime = runtime(HostSide::Server, false, &calls);
    assert!(ModListView::for_runtime(&runtime).is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    runtime.initialize().unwrap();
    assert!(ModListView::for_runtime(&runtime).is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 3);
    runtime.shutdown();
}

#[test]
fn empty_pack_and_lifecycle_snapshots_do_not_claim_per_mod_running_status() {
    let mut runtime = Runtime::for_side(vec![], HostSide::Client).unwrap();
    let before = ModListView::for_runtime(&runtime).unwrap();
    assert!(before.entries().is_empty());
    runtime.initialize().unwrap();
    let running = ModListView::for_runtime(&runtime).unwrap();
    assert_eq!(running.runtime_state(), State::Running);
    assert_eq!(before.runtime_state(), State::Resolved);
    runtime.shutdown();
    assert_eq!(
        ModListView::for_runtime(&runtime).unwrap().runtime_state(),
        State::Stopped
    );
}
