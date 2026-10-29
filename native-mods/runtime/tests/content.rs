//! Two independent mods communicate only through API descriptors and frozen reads.
use horizon_native_api::{
    CreativeTabDescriptor, Dependency, ItemDescriptor, LoadContext, ModContext, ModMetadata,
    ModSide, NativeMod, ResourceId, API_VERSION,
};
use horizon_native_runtime::{HostSide, LoaderError, Runtime, State};

static ALPHA: ModMetadata = ModMetadata {
    id: "alpha",
    name: "Alpha",
    version: "1",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
static BETA: ModMetadata = ModMetadata {
    id: "beta",
    dependencies: &[Dependency {
        id: "alpha",
        version: "1",
        optional: false,
    }],
    ..ALPHA
};
static UNDECLARED: ModMetadata = ModMetadata {
    dependencies: &[],
    ..BETA
};
static RESERVED: ModMetadata = ModMetadata {
    id: "minecraft",
    ..ALPHA
};

fn id(text: &str) -> ResourceId {
    text.parse().unwrap()
}

struct Example {
    metadata: &'static ModMetadata,
    items: Vec<ItemDescriptor>,
    tabs: Vec<CreativeTabDescriptor>,
}

impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        self.metadata
    }

    fn register(&mut self, context: &mut LoadContext) -> Result<(), String> {
        for item in self.items.drain(..) {
            context.register_item(item);
        }
        for tab in self.tabs.drain(..) {
            context.register_tab(tab);
        }
        Ok(())
    }

    fn initialize(&mut self, context: &mut ModContext) -> Result<(), String> {
        // Both mods see the COMPLETE snapshot, including later-registered beta.
        if context.content().items.len() != 2 || context.content().tabs.len() != 2 {
            return Err("setup saw an incomplete registry".into());
        }
        let tab = &context.content().tabs[&id("beta:tools")];
        if tab.items != [id("alpha:gear"), id("beta:handle")] {
            return Err("cross-mod reference or display order was lost".into());
        }
        context.log("read both mods through the frozen public API");
        Ok(())
    }
}

fn pair() -> (Example, Example) {
    let item = |name| ItemDescriptor {
        id: id(name),
        translation_key: format!("item.{name}"),
        template: id("minecraft:stick"),
    };
    let tab = |name, icon, items| CreativeTabDescriptor {
        id: id(name),
        translation_key: format!("tab.{name}"),
        icon: id(icon),
        items,
    };
    (
        Example {
            metadata: &ALPHA,
            items: vec![item("alpha:gear")],
            tabs: vec![tab("alpha:tools", "alpha:gear", vec![])],
        },
        Example {
            metadata: &BETA,
            items: vec![item("beta:handle")],
            tabs: vec![tab(
                "beta:tools",
                "alpha:gear",
                vec![id("alpha:gear"), id("beta:handle")],
            )],
        },
    )
}

#[test]
fn two_mods_freeze_before_setup_on_every_host_in_stable_order() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        for reverse in [false, true] {
            let (a, b) = pair();
            let mut mods: Vec<Box<dyn NativeMod>> = vec![Box::new(a), Box::new(b)];
            if reverse {
                mods.reverse();
            }
            let mut runtime = Runtime::for_side(mods, side).unwrap();
            assert!(runtime.content().items.is_empty());
            assert_eq!(runtime.initialize().unwrap().len(), 2);
            assert_eq!(runtime.state(), State::Running);
            assert_eq!(
                runtime.metadata().map(|m| m.id).collect::<Vec<_>>(),
                ["alpha", "beta"]
            );
            assert_eq!(
                runtime
                    .content()
                    .items
                    .keys()
                    .map(ResourceId::as_str)
                    .collect::<Vec<_>>(),
                ["alpha:gear", "beta:handle"]
            );
            let snapshot = runtime.content().clone();
            assert!(runtime.initialize().is_err());
            assert_eq!(runtime.content(), &snapshot);
            assert!(runtime.shutdown().errors.is_empty());
        }
    }
}

#[test]
fn invalid_registration_never_publishes_partial_content_or_runs_setup() {
    for case in [
        "duplicate item",
        "duplicate tab",
        "foreign ID",
        "missing icon",
        "missing member",
        "undeclared dependency",
        "blank title",
        "reserved namespace",
    ] {
        let (mut a, mut b) = pair();
        let expected = match case {
            "duplicate item" => {
                a.items.push(a.items[0].clone());
                "duplicate item ID alpha:gear"
            }
            "duplicate tab" => {
                b.tabs.push(b.tabs[0].clone());
                "duplicate tab ID beta:tools"
            }
            "foreign ID" => {
                b.items[0].id = id("alpha:intruder");
                "cannot register foreign ID"
            }
            "missing icon" => {
                b.tabs[0].icon = id("alpha:missing");
                "missing item alpha:missing"
            }
            "missing member" => {
                b.tabs[0].items.push(id("beta:missing"));
                "missing item beta:missing"
            }
            "undeclared dependency" => {
                b.metadata = &UNDECLARED;
                "without a declared dependency"
            }
            "blank title" => {
                a.items[0].translation_key.clear();
                "needs a translation key"
            }
            "reserved namespace" => {
                a.metadata = &RESERVED;
                a.items[0].id = id("minecraft:gear");
                b.metadata = &UNDECLARED;
                "reserved content namespace"
            }
            _ => unreachable!(),
        };
        let mut runtime = Runtime::new(vec![Box::new(b), Box::new(a)]).unwrap();
        let error = runtime.initialize().unwrap_err();
        assert!(
            matches!(&error, LoaderError::HookFailed { hook: "register" | "freeze", reason, .. } if reason.contains(expected)),
            "{case}: {error}"
        );
        assert_eq!(runtime.state(), State::Failed);
        assert!(runtime.content().items.is_empty());
        assert!(runtime.content().tabs.is_empty());
        assert!(runtime.initialize().is_err());
        assert!(runtime.shutdown().errors.is_empty());
        assert!(runtime.shutdown().actions.is_empty());
    }
}
