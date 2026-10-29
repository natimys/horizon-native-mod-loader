use horizon_native_pack::create_runtime;

fn main() {
    let mut runtime = create_runtime().expect("compiled modpack is valid");
    for metadata in runtime.metadata() {
        println!("{} {} ({})", metadata.name, metadata.version, metadata.id);
    }
    for action in runtime.initialize().expect("mods initialize") {
        println!("{action:?}");
    }
    for action in runtime.shutdown().actions {
        println!("{action:?}");
    }
}
