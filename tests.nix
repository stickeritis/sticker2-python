{ pkgs ? import (import ./nix/sources.nix).nixpkgs {} }:

with pkgs;

[
  (callPackage ./default.nix { python3Packages = python37Packages; })
  (callPackage ./default.nix { python3Packages = python38Packages; })
]
