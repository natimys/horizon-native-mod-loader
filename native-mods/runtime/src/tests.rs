use super::*;
use horizon_native_api::Dependency;
use std::sync::{Arc, Mutex};

const A: ModMetadata = ModMetadata {
    id: "alpha",
    name: "Alpha",
    version: "1.0.0",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
const DEP: Dependency = Dependency {
    id: "alpha",
    version: "1.0.0",
    optional: false,
};
const B: ModMetadata = ModMetadata {
    id: "beta",
    dependencies: &[DEP],
    ..A
};
struct Dummy {
    meta: &'static ModMetadata,
    trace: Arc<Mutex<Vec<String>>>,
    fail: &'static str,
    tick: bool,
}
impl Dummy {
    fn hook(&self, name: &str, c: &mut ModContext) -> Result<(), String> {
        self.trace
            .lock()
            .unwrap()
            .push(format!("{}:{name}", self.meta.id));
        c.log(name);
        if self.fail == name {
            Err("test failure".into())
        } else {
            Ok(())
        }
    }
}
impl NativeMod for Dummy {
    fn metadata(&self) -> &'static ModMetadata {
        self.meta
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        self.trace
            .lock()
            .unwrap()
            .push(format!("{}:register", self.meta.id));
        if self.tick {
            c.subscribe_tick();
        }
        if self.fail == "register" {
            Err("test failure".into())
        } else {
            Ok(())
        }
    }
    fn initialize(&mut self, c: &mut ModContext) -> Result<(), String> {
        self.hook("initialize", c)
    }
    fn client_setup(&mut self, c: &mut ModContext) -> Result<(), String> {
        self.hook("client_setup", c)
    }
    fn server_setup(&mut self, c: &mut ModContext) -> Result<(), String> {
        self.hook("server_setup", c)
    }
    fn tick(&mut self, _: u64, c: &mut ModContext) -> Result<(), String> {
        self.hook("tick", c)
    }
    fn shutdown(&mut self, c: &mut ModContext) -> Result<(), String> {
        self.hook("shutdown", c)
    }
}
fn boxed(meta: &'static ModMetadata) -> Box<dyn NativeMod> {
    Box::new(Dummy {
        meta,
        trace: Arc::default(),
        fail: "",
        tick: false,
    })
}
fn error(mods: Vec<Box<dyn NativeMod>>) -> LoaderError {
    Runtime::new(mods).err().unwrap()
}

#[test]
fn deterministic_dependency_and_id_order() {
    const C: ModMetadata = ModMetadata {
        id: "aardvark",
        ..A
    };
    for mods in [
        vec![boxed(&B), boxed(&A), boxed(&C)],
        vec![boxed(&C), boxed(&B), boxed(&A)],
    ] {
        let r = Runtime::new(mods).unwrap();
        assert_eq!(
            r.metadata().map(|m| m.id).collect::<Vec<_>>(),
            ["aardvark", "alpha", "beta"]
        );
    }
}
#[test]
fn graph_errors_are_actionable() {
    assert_eq!(
        error(vec![boxed(&A), boxed(&A)]),
        LoaderError::DuplicateId("alpha".into())
    );
    assert_eq!(
        error(vec![boxed(&B)]),
        LoaderError::MissingDependency {
            mod_id: "beta".into(),
            dependency: "alpha".into()
        }
    );
    const BAD: ModMetadata = ModMetadata { id: "Bad ID", ..A };
    assert_eq!(
        error(vec![boxed(&BAD)]),
        LoaderError::InvalidId("Bad ID".into())
    );
    const FUTURE: ModMetadata = ModMetadata {
        api_version: API_VERSION + 1,
        ..A
    };
    assert!(matches!(
        error(vec![boxed(&FUTURE)]),
        LoaderError::IncompatibleApi { .. }
    ));
    const WRONG: ModMetadata = ModMetadata {
        version: "2.0.0",
        ..A
    };
    assert!(matches!(
        error(vec![boxed(&WRONG), boxed(&B)]),
        LoaderError::IncompatibleDependency { .. }
    ));
}
#[test]
fn cycles_include_optional_present_dependencies() {
    const X: ModMetadata = ModMetadata {
        id: "x",
        dependencies: &[Dependency {
            id: "y",
            version: "1.0.0",
            optional: true,
        }],
        ..A
    };
    const Y: ModMetadata = ModMetadata {
        id: "y",
        dependencies: &[Dependency {
            id: "x",
            version: "1.0.0",
            optional: false,
        }],
        ..A
    };
    assert_eq!(
        error(vec![boxed(&Y), boxed(&X)]),
        LoaderError::DependencyCycle(vec!["x".into(), "y".into()])
    );
    assert!(Runtime::new(vec![boxed(&X)]).is_ok());
}
#[test]
fn side_filtering_and_active_dependencies() {
    const CLIENT: ModMetadata = ModMetadata {
        side: ModSide::Client,
        ..A
    };
    let mut server = Runtime::for_side(vec![boxed(&CLIENT)], HostSide::Server).unwrap();
    assert_eq!(server.metadata().len(), 0);
    assert!(server.initialize().unwrap().is_empty());
    assert!(matches!(
        Runtime::for_side(vec![boxed(&CLIENT), boxed(&B)], HostSide::Server),
        Err(LoaderError::InactiveDependency { .. })
    ));
}
#[test]
fn lifecycle_registers_everything_before_setup_and_reverses_shutdown() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let make = |meta, tick| {
        Box::new(Dummy {
            meta,
            trace: trace.clone(),
            fail: "",
            tick,
        }) as Box<dyn NativeMod>
    };
    let mut r = Runtime::for_side(vec![make(&B, true), make(&A, false)], HostSide::Server).unwrap();
    assert_eq!(
        r.tick(1).unwrap_err(),
        LoaderError::InvalidState(State::Resolved)
    );
    r.initialize().unwrap();
    assert_eq!(r.state(), State::Running);
    assert!(r.initialize().is_err());
    r.tick(1).unwrap();
    assert!(r.shutdown().errors.is_empty());
    assert!(r.shutdown().actions.is_empty());
    assert!(r.tick(2).is_err());
    assert_eq!(
        *trace.lock().unwrap(),
        [
            "alpha:register",
            "beta:register",
            "alpha:initialize",
            "alpha:server_setup",
            "beta:initialize",
            "beta:server_setup",
            "beta:tick",
            "beta:shutdown",
            "alpha:shutdown"
        ]
    );
}
#[test]
fn failures_are_terminal_and_cleanup_continues() {
    for hook in [
        "register",
        "initialize",
        "client_setup",
        "server_setup",
        "tick",
        "shutdown",
    ] {
        let trace = Arc::new(Mutex::new(Vec::new()));
        let make = |meta, fail| {
            Box::new(Dummy {
                meta,
                trace: trace.clone(),
                fail,
                tick: true,
            }) as Box<dyn NativeMod>
        };
        let mut r = Runtime::new(vec![make(&A, ""), make(&B, hook)]).unwrap();
        let result = r.initialize();
        if ["register", "initialize", "client_setup", "server_setup"].contains(&hook) {
            assert!(result.is_err());
            assert_eq!(r.state(), State::Failed);
        } else {
            result.unwrap();
            assert_eq!(r.tick(0).is_err(), hook == "tick");
        }
        assert!(r.initialize().is_err());
        let report = r.shutdown();
        assert_eq!(report.errors.len(), usize::from(hook == "shutdown"));
        let t = trace.lock().unwrap();
        assert_eq!(&t[t.len() - 2..], ["beta:shutdown", "alpha:shutdown"]);
    }
}
#[test]
fn empty_runtime_is_a_noop_and_never_subscribes() {
    let mut r = Runtime::new(vec![]).unwrap();
    assert!(!r.has_tick_subscribers());
    assert!(r.initialize().unwrap().is_empty());
    assert!(r.tick(0).unwrap().is_empty());
    assert!(r.shutdown().actions.is_empty());
}
#[test]
fn client_and_server_setup_do_not_cross_sides() {
    for (side, present, absent) in [
        (HostSide::Server, "server_setup", "client_setup"),
        (HostSide::Client, "client_setup", "server_setup"),
    ] {
        let mut r = Runtime::for_side(vec![boxed(&A)], side).unwrap();
        let actions = r.initialize().unwrap();
        assert!(actions
            .iter()
            .any(|a| matches!(a, ModAction::Log { message, .. } if message == present)));
        assert!(!actions
            .iter()
            .any(|a| matches!(a, ModAction::Log { message, .. } if message == absent)));
    }
}
