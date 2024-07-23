use prost_build::Config;
use std::path::{self, PathBuf};

fn main() {
    let mut config = Config::new();
    config.out_dir(PathBuf::from("src/protos"));
    let path = PathBuf::from("/Users/matkat/Software/protos");
    if PathBuf::from(&path).exists() {
        config
            .compile_protos(
                &["/Users/matkat/Software/protos/vehicle_model.proto"],
                &["/Users/matkat/Software/protos/"],
            )
            .unwrap();
    } else {
        config
            .compile_protos(&["./protos/vehicle_model.proto"], &["./protos/"])
            .unwrap();
    }
}
