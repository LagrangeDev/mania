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
        pkgs = import nixpkgs {
          inherit system overlays;
        };
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
        commonArgs = {
          inherit stdenv;
          inherit
            (craneLib.crateNameFromCargoToml {
              cargoToml = ./mania/Cargo.toml;
            })
            pname
            ;
          inherit
            (craneLib.crateNameFromCargoToml {
              cargoToml = ./Cargo.toml;
            })
            version
            ;
          src = filteredSource;
          strictDeps = true;
          nativeBuildInputs = [
            pkgs.protobuf
          ];
          doCheck = false;
          meta = {
            mainProgram = "mania";
            homepage = "https://github.com/LagrangeDev/mania";
            license = pkgs.lib.licenses.gpl3Only;
          };
        };
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        checkTypo =
          pkgs.runCommandNoCCLocal "check-typo"
            {
              src = ./.;
              nativeBuildInputs = with pkgs; [ typos ];
            }
            ''
              mkdir -p $out

              cd $src && typos -c $src/typos.toml
            '';
      in
      {
        packages = {
          mania = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoExtraArgs = ''
                --example multi-login
              '';
              postInstall = ''
                mkdir -p $out/bin

                cp ./target/release/examples/multi-login $out/bin/mania
              '';
            }
          );
          default = self.packages."${system}".mania;
        };
        checks = {
          inherit (self.packages."${system}") mania;
          typo = checkTypo;
          audit = craneLib.cargoAudit (
            commonArgs
            // {
              inherit advisory-db;
            }
          );
          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
          fmt = craneLib.cargoFmt commonArgs;
          doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          test = craneLib.cargoTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks."${system}";
          packages = with pkgs; [
            rust-analyzer
            cargo-flamegraph
            cargo-tarpaulin
            lldb
          ];
          shellHook = '''';
        };
      }
    )
    // {
      overlays.default = final: prev: {
        inherit (self.packages."${final.system}") mania;
      };
    };
}
