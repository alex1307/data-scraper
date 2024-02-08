use prost_build::Config;
use std::path::PathBuf;

fn main() {
    let mut config = Config::new();
    config.out_dir(PathBuf::from("src/protos"));

    config
        .compile_protos(&["protos/vehicle_model.proto"], &["protos/"])
        .unwrap();
}
