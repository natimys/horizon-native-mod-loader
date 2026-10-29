use horizon_native_api::*;
use horizon_native_runtime::{HostSide, Runtime};
static META: ModMetadata = ModMetadata {
    id: "commands",
    name: "Commands",
    version: "1",
    description: "",
    authors: &[],
    license: "MIT",
    api_version: API_VERSION,
    side: ModSide::Common,
    dependencies: &[],
};
struct Example(bool);
impl NativeMod for Example {
    fn metadata(&self) -> &'static ModMetadata {
        &META
    }
    fn register(&mut self, c: &mut LoadContext) -> Result<(), String> {
        let command = CommandDescriptor {
            id: "commands:ping".parse()?,
        };
        c.register_command(command.clone());
        if self.0 {
            c.register_command(command);
        }
        Ok(())
    }
    fn command(
        &mut self,
        _: &ResourceId,
        p: &PlayerSnapshot,
        c: &mut GameplayContext,
    ) -> Result<(), String> {
        c.tell(p.id, "pong");
        Ok(())
    }
}
#[test]
fn commands_resolve_owners_without_client_execution_or_duplicates() {
    let player = PlayerSnapshot {
        id: 42,
        name: "test".into(),
        dimension: "minecraft:overworld".parse().unwrap(),
        position: [0.; 3],
        health: 20.,
    };
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut r = Runtime::for_side(vec![Box::new(Example(false))], side).unwrap();
        r.initialize().unwrap();
        assert!(r
            .command(&"minecraft:stop".parse().unwrap(), &player)
            .is_none());
        let result = r.command(&"commands:ping".parse().unwrap(), &player);
        if side == HostSide::Client {
            assert!(result.is_none());
        } else {
            let result = result.unwrap().unwrap();
            assert_eq!(result[0].mod_id, "commands");
            assert_eq!(
                result[0].action,
                GameplayAction::Tell {
                    player: 42,
                    text: "pong".into()
                }
            );
        }
    }
    let mut r = Runtime::new(vec![Box::new(Example(true))]).unwrap();
    assert!(r.freeze().is_err());
}
