{ lib, ... }:
{
  narHashesFromCargoLock =
    lockFile:
    let
      inherit (lib) hasPrefix last head;
      inherit (builtins)
        split
        readFile
        fromTOML
        listToAttrs
        filter
        ;
      packages = (fromTOML (readFile lockFile)).package;
      filteredPackages = filter (p: p ? source && hasPrefix "git+" p.source) packages;
    in
    listToAttrs (
      map (p: {
        name = "${p.name}-${p.version}";
        value =
          (fetchGit {
            rev = last (split "#" p.source);
            url = last (split "\\+" (head (split "\\?" p.source)));
            allRefs = true;
          }).narHash;
      }) filteredPackages
    );
}
