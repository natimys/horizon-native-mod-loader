#![cfg(feature = "gameplay-example")]
use horizon_native_api::*;
use horizon_native_runtime::HostSide;

#[test]
fn workshop_composes_through_public_api_on_both_sides() {
    for side in [HostSide::Client, HostSide::Server, HostSide::Combined] {
        let mut runtime = horizon_native_pack::create_for_side(side).unwrap();
        runtime.initialize().unwrap();
        let player = PlayerSnapshot {
            id: 42,
            name: "Test".into(),
            dimension: "minecraft:overworld".parse().unwrap(),
            position: [0.5, 65., 0.5],
            health: 20.,
        };
        let command = runtime.command(&"workshop:place".parse().unwrap(), &player);
        if side == HostSide::Client {
            assert!(command.is_none());
            continue;
        }
        let actions = command.unwrap().unwrap();
        assert!(actions.iter().any(|a|matches!(&a.action,GameplayAction::SetBlock{position,state,..} if position==&[2,65,0] && state=="workshop:counter")));
        let descriptor =
            &runtime.content().block_entities[&"workshop:counter_data".parse().unwrap()];
        let entity = BlockEntitySnapshot {
            id: descriptor.id.clone(),
            position: [2, 65, 0],
            version: descriptor.version,
            fields: descriptor.fields.clone(),
        };
        let event = GameplayEvent::BlockEntityTick {
            world: WorldSnapshot {
                dimension: player.dimension.clone(),
                tick: 20,
            },
            entity,
        };
        let actions = runtime.dispatch(&event).unwrap();
        assert!(
            matches!(&actions[0].action,GameplayAction::SetBlockEntity{entity,..} if entity.fields["visits"]==1)
        );
        assert!(runtime.shutdown().errors.is_empty());
    }
}
