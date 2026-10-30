//! Diagnostic runtime microbenchmark, not a world/renderer or sandbox budget.
use horizon_native_api::{GameplayEvent, WorldSnapshot};
use horizon_native_runtime::{HostSide, Runtime};
use std::{hint::black_box, time::Instant};

fn measure(label: &str, mut runtime: Runtime) {
    let startup = Instant::now();
    runtime.initialize().unwrap();
    let startup_us = startup.elapsed().as_micros();
    let mut event = GameplayEvent::WorldTick {
        world: WorldSnapshot {
            dimension: "minecraft:overworld".parse().unwrap(),
            tick: 2,
        },
        players: vec![],
    };
    let iterations = 100_000;
    let begin = Instant::now();
    for tick in 2..iterations + 2 {
        if let GameplayEvent::WorldTick { world, .. } = &mut event {
            world.tick = tick;
        }
        black_box(runtime.dispatch(black_box(&event)).unwrap());
    }
    println!(
        "{label}: startup_us={startup_us} dispatch_ns={} gameplay={}",
        begin.elapsed().as_nanos() / u128::from(iterations),
        runtime.has_gameplay_subscribers()
    );
    assert!(runtime.shutdown().errors.is_empty());
}

fn main() {
    measure(
        "empty-server",
        Runtime::for_side(vec![], HostSide::Server).unwrap(),
    );
    measure(
        "pack-client",
        horizon_native_pack::create_for_side(HostSide::Client).unwrap(),
    );
    measure(
        "pack-server",
        horizon_native_pack::create_for_side(HostSide::Server).unwrap(),
    );
}
