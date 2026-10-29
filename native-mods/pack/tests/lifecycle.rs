use horizon_native_pack::{create_for_side, create_runtime};
use horizon_native_runtime::{HostSide, Runtime, State};

#[test]
fn compiled_hello_has_metadata_init_and_shutdown_on_both_hosts() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut runtime = create_for_side(side).unwrap();
        let metadata = runtime.metadata().find(|m| m.id == "hello_mod").unwrap();
        assert_eq!(metadata.name, "Hello from native Rust");
        assert_eq!(metadata.api_version, 1);
        let actions = runtime.initialize().unwrap();
        assert_eq!(actions.len(), 1);
        assert!(format!("{:?}", actions[0]).contains("HelloMod initialized"));
        assert!(!runtime.has_tick_subscribers());
        assert_eq!(runtime.shutdown().actions.len(), 1);
        assert_eq!(runtime.state(), State::Stopped);
    }
}

#[test]
fn zero_mod_lifecycle_has_no_actions_or_callbacks() {
    let mut runtime = Runtime::new(vec![]).unwrap();
    assert_eq!(runtime.metadata().len(), 0);
    assert!(runtime.initialize().unwrap().is_empty());
    assert!(runtime.tick(1).unwrap().is_empty());
    assert!(runtime.shutdown().actions.is_empty());
    assert_eq!(
        create_runtime().unwrap().metadata().len(),
        1 + usize::from(cfg!(feature = "test-pack"))
    );
}

#[test]
fn hello_declares_real_item_and_original_resources_before_setup() {
    let mut runtime = horizon_native_pack::create_runtime().unwrap();
    runtime.freeze().unwrap();
    let item = "hello_mod:token".parse().unwrap();
    let tab = "hello_mod:hello".parse().unwrap();
    assert_eq!(runtime.content().tabs[&tab].items, [item]);
    assert_eq!(
        runtime.content().items.len(),
        1 + usize::from(cfg!(feature = "test-pack"))
    );
    assert_eq!(
        runtime.content().resources.len(),
        5 * (1 + usize::from(cfg!(feature = "test-pack")))
    );
    assert!(
        runtime.content().resources["assets/hello_mod/textures/item/token.png"]
            .starts_with(b"\x89PNG")
    );
    let snapshot = runtime.content().clone();
    runtime.initialize().unwrap();
    assert_eq!(runtime.content(), &snapshot);
    assert!(runtime.shutdown().errors.is_empty());
}

#[test]
#[cfg(feature = "test-pack")]
fn acceptance_pack_has_two_independent_mods_and_stable_names_on_all_hosts() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut runtime = create_for_side(side).unwrap();
        runtime.freeze().unwrap();
        assert_eq!(
            runtime.metadata().map(|m| m.id).collect::<Vec<_>>(),
            ["companion", "hello_mod"]
        );
        assert_eq!(runtime.content().items.len(), 2);
        assert_eq!(runtime.content().tabs.len(), 2);
        runtime.initialize().unwrap();
        assert!(runtime.shutdown().errors.is_empty());
    }
}
