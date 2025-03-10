{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
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
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
          (final: prev: {
            lib = prev.lib // (import ./nix/lib.nix final);
          })
        ];
        pkgs = import nixpkgs { inherit system overlays; };
        env =
          let
            inherit (pkgs) libclang lib;
            version = lib.getVersion libclang;
            majorVersion = lib.versions.major version;
          in
          {
            BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${libclang.lib}/lib/clang/${majorVersion}/include";
            LIBCLANG_PATH = lib.makeLibraryPath [ libclang.lib ];
          };
        rust-bin = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        package = pkgs.callPackage ./nix/package.nix { inherit env rust-bin; };
      in
      {
        packages = {
          mania = package;
          default = self.packages."${system}".mania;
        };
        checks = {
          inherit (pkgs.callPackage ./nix/checks.nix { inherit package advisory-db rust-bin; })
            package
            clippy
            audit
            typo
            fmt
            ;
        };
        devShells.default = pkgs.mkShell {
          inherit env;
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
      overlays.default = final: prev: { inherit (self.packages."${final.system}") mania; };
    };
}
