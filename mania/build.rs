use prost_build::Config;
fn main() {
    let protos = glob::glob("src/core/protos/**/*.proto")
        .unwrap()
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();
    let mut config = Config::new();
    config
        .include_file("_includes.rs")
        .compile_protos(&protos, &["src/core/protos"])
        .unwrap();
}
