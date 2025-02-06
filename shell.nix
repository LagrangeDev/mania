# use flake's devShells.default for non-flake-enabled nix instances
let
  lock = builtins.fromJSON (builtins.readFile ./flake.lock);
  nodeName = lock.nodes.root.inputs.flake-compat;
  compat = fetchTarball {
    url =
      lock.nodes.${nodeName}.locked.url
        or "https://github.com/edolstra/flake-compat/archive/${lock.nodes.${nodeName}.locked.rev}.tar.gz";
    sha256 = lock.nodes.${nodeName}.locked.narHash;
  };
in
(import compat { src = ./.; }).shellNix.default
