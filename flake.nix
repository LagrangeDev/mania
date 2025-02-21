{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
    };
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
      rust-overlay,
      crane,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        filteredSource =
          let
            pathsToIgnore = [
              ".envrc"
              ".ignore"
              ".github"
              ".gitignore"
              "rust-toolchain.toml"
              "README.MD"
              "flake.nix"
              "flake.lock"
              "target"
              "LICENCE"
              ".direnv"
            ];
            ignorePaths =
              path: type:
              let
                inherit (nixpkgs) lib;
                # split the nix store path into its components
                components = lib.splitString "/" path;
                # drop off the `/nix/hash-source` section from the path
                relPathComponents = lib.drop 4 components;
                # reassemble the path components
                relPath = lib.concatStringsSep "/" relPathComponents;
              in
              lib.all (p: !(lib.hasPrefix p relPath)) pathsToIgnore;
          in
          builtins.path {
            name = "mania-source";
            path = toString ./.;
            # filter out unnecessary paths
            filter = ignorePaths;
          };
        stdenv = if pkgs.stdenv.isLinux then pkgs.stdenv else pkgs.clangStdenv;
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        env =
          let
            inherit (pkgs) lib libclang;
            version = lib.getVersion libclang;
            majorVersion = lib.versions.major version;
          in
          {
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${libclang.lib}/lib/clang/${majorVersion}/include";
            LIBCLANG_PATH = lib.makeLibraryPath [ libclang.lib ];
          };
        commonArgs = {
          inherit stdenv env;
          inherit (craneLib.crateNameFromCargoToml { cargoToml = ./mania/Cargo.toml; }) pname;
          inherit (craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }) version;
          src = filteredSource;
          strictDeps = true;
          depsBuildBuild = with pkgs; [ protobuf ];
          nativeBuildInputs = with pkgs; [ libclang.lib ];
          doCheck = false;
          meta = {
            mainProgram = "mania";
            homepage = "https://github.com/LagrangeDev/mania";
            license = pkgs.lib.licenses.gpl3Only;
          };
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        typoCheck =
          pkgs.runCommandNoCCLocal "check-typo"
            {
              src = ./.;
              nativeBuildInputs = with pkgs; [ typos ];
            }
            ''
              mkdir -p $out

              cd $src
              typos --config ./typos.toml
            '';
        fmtCheck =
          let
            restFmtCheck =
              pkgs.runCommandNoCCLocal "check-fmt"
                {
                  src = ./.;
                  nativeBuildInputs = with pkgs; [
                    taplo
                    nixfmt-rfc-style
                    deno
                    just
                    shfmt
                  ];
                }
                ''
                  mkdir -p $out

                  cd $src
                  # just
                  echo '==> just format check'
                  just --unstable --fmt --check
                  # markdown
                  echo '==> markdown format check'
                  find . -type f -regextype egrep -regex '^.*\.(md|MD)$' -exec deno fmt --check --ext md {} +
                  # toml
                  echo '==> toml format check'
                  find . -type f -regextype egrep -regex '^.*\.toml$' -exec taplo format --check {} +
                  # yaml
                  echo '==> yaml format check'
                  find . -type f -regextype egrep -regex '^.*\.yml$' -exec deno fmt --check --ext yml {} +
                  # nix
                  echo '==> nix format check'
                  find . -type f -regextype egrep -regex '^.*\.nix$' -exec nixfmt --check {} +
                  # sh
                  echo '==> sh format check'
                  cd ./scripts && find . -type f -executable -exec shfmt -p -s -d -i 2 -ci -sr -kp -fn '{}' +
                '';
          in
          pkgs.symlinkJoin {
            name = "fmtCheck";
            paths = [
              restFmtCheck
              (craneLib.cargoFmt commonArgs)
            ];
          };
      in
      {
        packages = {
          mania = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = ''
                --example mania_multi_login
              '';
              postInstall = ''
                mkdir -p $out/bin

                cp ./target/release/examples/mania_multi_login $out/bin/mania
              '';
            }
          );
          default = self.packages."${system}".mania;
        };
        checks = {
          inherit (self.packages."${system}") mania;
          typo = typoCheck;
          audit = craneLib.cargoAudit (commonArgs // { inherit advisory-db; });
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          fmt = fmtCheck;
          doc = craneLib.cargoDoc (commonArgs // { inherit cargoArtifacts; });
          test = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
        };
        devShells.default = pkgs.mkShell {
          inherit env;
          inputsFrom = builtins.attrValues self.checks."${system}";
          packages = with pkgs; [
            # deps
            protobuf

            # dev
            rust-analyzer
            cargo-flamegraph
            cargo-tarpaulin
            lldb

            # fmt
            taplo
            nixfmt-rfc-style
            deno
            just
            shfmt
          ];
          shellHook = '''';
        };
      }
    )
    // {
      overlays.default = final: prev: { inherit (self.packages."${final.system}") mania; };
    };
}
