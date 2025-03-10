{
  lib,
  package,
  advisory-db,
  runCommandNoCCLocal,
  cargo-audit,
  typos,
  taplo,
  nixfmt-rfc-style,
  deno,
  just,
  shfmt,
  rust-bin,
}:
let
  makePackageCheck =
    suffix:
    {
      doCheck ? false,
      doBuild ? true,
      doInstall ? true,
      ...
    }@args:
    package.passthru.override (
      prev:
      {
        pname = prev.pname + "-${suffix}";
      }
      // (lib.optionalAttrs (!doBuild) {
        dontCargoBuild = true;
      })
      // (lib.optionalAttrs doCheck {
        checkType = "debug";
        useNextest = true;
      })
      // (lib.optionalAttrs (!doInstall) {
        dontCargoInstall = true;
        preInstall = ''
          mkdir -p $out
        '';
      })
      // args
    );
in
{
  inherit package;
  test = makePackageCheck "test" {
    doCheck = true;
    doBuild = false;
    doInstall = false;
  };
  clippy = makePackageCheck "clippy" {
    doCheck = false;
    doInstall = false;
    doBuild = false;
    buildPhase = ''
      cargo clippy --locked --offline --all-targets -- --deny warnings
    '';
  };
  audit =
    runCommandNoCCLocal "mania-audit"
      {
        src = ./..;
        nativeBuildInputs = [ cargo-audit ];
      }
      ''
        mkdir -p $out

        cd $src
        cargo-audit audit -n -d ${advisory-db}
      '';
  typo =
    runCommandNoCCLocal "mania-typo"
      {
        src = ./..;
        nativeBuildInputs = [ typos ];
      }
      ''
        mkdir -p $out

        cd $src
        typos --config ./typos.toml
      '';
  fmt =
    runCommandNoCCLocal "mania-fmt"
      {
        src = ./..;
        nativeBuildInputs = [
          taplo
          nixfmt-rfc-style
          deno
          just
          shfmt
          rust-bin
        ];
      }
      ''
        mkdir -p $out

        cd $src
        # rust
        cargo fmt --check
        # just
        echo '==> just format check'
        just --unstable --fmt --check
        # markdown
        echo '==> markdown format check'
        find . -type f -regextype egrep -regex '^.*\.md$' -exec deno fmt --check --ext md {} +
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
}
