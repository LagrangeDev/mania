use anyhow::Result;
use prost_build::Config;
use std::path::Path;

fn main() -> Result<()> {
    if Path::new("../.git").is_dir() {
        if !Path::new("../.git/hooks").exists() {
            std::fs::create_dir_all(".././.git/hooks")?;
        }
        std::fs::copy("../scripts/pre-commit", "../.git/hooks/pre-commit")?;
        std::fs::copy("../scripts/pre-push", "../.git/hooks/pre-push")?;
    }
    let protos = glob::glob("src/core/protos/**/*.proto")?
        .map(|x| x.unwrap())
        .collect::<Vec<_>>();
    let mut config = Config::new();
    Ok(config
        .include_file("_includes.rs")
        .compile_protos(&protos, &["src/core/protos"])?)
}
