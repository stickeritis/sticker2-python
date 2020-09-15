{
  pkgs ? import (import ./nix/sources.nix).nixpkgs {
    config = {
      allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
        "libtorch"
      ];
    };
  }
}:

with pkgs;

[
  (callPackage ./default.nix { python3Packages = python37Packages; })
  (callPackage ./default.nix { python3Packages = python38Packages; })
]
