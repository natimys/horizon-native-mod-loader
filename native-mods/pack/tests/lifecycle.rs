use horizon_native_pack::{create_for_side, create_runtime};
use horizon_native_runtime::{HostSide, Runtime, State};

#[test]
fn compiled_hello_has_metadata_init_and_shutdown_on_both_hosts() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut runtime = create_for_side(side).unwrap();
        let metadata = runtime.metadata().next().unwrap();
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
    assert_eq!(create_runtime().unwrap().metadata().len(), 1);
}
