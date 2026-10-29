use horizon_native_api::*;
use horizon_native_runtime::{HostSide, Runtime, State};
static A: ModMetadata = ModMetadata {
    id: "a",
    name: "A",
    version: "1",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
static B: ModMetadata = ModMetadata { id: "b", ..A };
struct Example(&'static ModMetadata, u8);
impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        self.0
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        c.subscribe_gameplay(GameplaySubscriptions {
            world: true,
            ..Default::default()
        });
        Ok(())
    }
    fn gameplay(&mut self, _: &GameplayEvent, c: &mut GameplayContext) -> Result<(), String> {
        c.tell(42, self.0.id);
        match self.1 {
            1 => return Err("intentional failure".into()),
            2 => {
                for _ in 0..MAX_EVENT_ACTIONS {
                    c.tell(42, "overflow");
                }
            }
            _ => {}
        }
        Ok(())
    }
}
fn event() -> GameplayEvent {
    GameplayEvent::WorldTick {
        world: WorldSnapshot {
            dimension: "minecraft:overworld".parse().unwrap(),
            tick: 1,
        },
        players: vec![],
    }
}
#[test]
fn stable_opt_in_server_dispatch_and_atomic_failure() {
    for side in [HostSide::Server, HostSide::Combined] {
        let mut r = Runtime::for_side(
            vec![Box::new(Example(&B, 0)), Box::new(Example(&A, 0))],
            side,
        )
        .unwrap();
        r.initialize().unwrap();
        let batch = r.dispatch(&event()).unwrap();
        assert_eq!(
            batch.iter().map(|a| a.mod_id).collect::<Vec<_>>(),
            ["a", "b"]
        );
        for failure in [1, 2] {
            let mut r = Runtime::for_side(
                vec![Box::new(Example(&A, 0)), Box::new(Example(&B, failure))],
                side,
            )
            .unwrap();
            r.initialize().unwrap();
            assert!(r.dispatch(&event()).is_err());
            assert_eq!(r.state(), State::Failed);
            assert!(r.dispatch(&event()).is_err());
            assert!(r.shutdown().errors.is_empty());
        }
    }
    let mut r = Runtime::for_side(vec![Box::new(Example(&A, 1))], HostSide::Client).unwrap();
    r.initialize().unwrap();
    assert!(!r.has_gameplay_subscribers());
    assert!(r.dispatch(&event()).unwrap().is_empty());
}
#[test]
fn unsubscribed_events_do_not_invoke_callbacks() {
    let mut r = Runtime::new(vec![Box::new(Example(&A, 1))]).unwrap();
    r.initialize().unwrap();
    assert!(r
        .dispatch(&GameplayEvent::PlayerJoined(PlayerSnapshot {
            id: 42,
            name: "test".into(),
            dimension: "minecraft:overworld".parse().unwrap(),
            position: [0.; 3],
            health: 20.
        }))
        .unwrap()
        .is_empty());
}
