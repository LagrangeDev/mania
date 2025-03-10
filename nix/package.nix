{
  lib,
  env,
  rust-bin,
  makeRustPlatform,
  libclang,
  openssl,
  protobuf,
  pkg-config,
}:
let
  fs = lib.fileset;
  source = fs.difference (fs.gitTracked ./..) (
    fs.unions [
      ./../.envrc
      ./../.rustfmt.toml
      ./../flake.lock
      ./../.ignore
      ./../.justfile
      ./../LICENCE
      ./../typos.toml
      ./../scripts
      ./../.github
      (fs.fileFilter (file: lib.strings.hasInfix ".git" file.name) ./..)
      (fs.fileFilter (file: file.hasExt "md") ./..)
      (fs.fileFilter (file: file.hasExt "nix") ./..)
    ]
  );
  src = fs.toSource {
    root = ./..;
    fileset = source;
  };
  rustPlatform = makeRustPlatform {
    rustc = rust-bin;
    cargo = rust-bin;
  };
  version = with builtins; (fromTOML (readFile ./../Cargo.toml)).workspace.package.version;
  commonArgs = {
    inherit
      src
      version
      env
      ;
    pname = "mania";
    strictDeps = true;
    doCheck = false;
    buildInputs = [
      libclang.lib
      openssl.dev
    ];
    nativeBuildInputs = [
      protobuf
      pkg-config
    ];
    cargoLock = rec {
      lockFile = ./../Cargo.lock;
      outputHashes = lib.narHashesFromCargoLock lockFile;
    };
    cargoBuildFlags = ''
      --example mania_multi_login
    '';
    preInstall = ''
      cp $tmpDir/examples/mania_multi_login $tmpDir/mania
      bins+="''${bins:+\n}$tmpDir/mania"
    '';
    meta = {
      mainProgram = "mania";
      homepage = "https://github.com/LagrangeDev/mania";
      license = lib.licenses.gpl3Only;
    };
  };
in
(rustPlatform.buildRustPackage commonArgs)
// {
  passthru = {
    override = (lib.makeOverridable rustPlatform.buildRustPackage commonArgs).override;
  };
}
