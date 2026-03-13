use anyhow::Result;
use prost_build::Config;

fn main() -> Result<()> {
    let protos = glob::glob("src/core/protos/**/*.proto")?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();
    let mut config = Config::new();
    Ok(config
        .include_file("_includes.rs")
        .compile_protos(&protos, &["src/core/protos"])?)
}
