with import <nixpkgs> {};

let
  sources = import ./nix/sources.nix;
  mozilla = callPackage "${sources.mozilla}/package-set.nix" {};
  rust = mozilla.rustChannelOf { channel = "nightly"; date = "2020-04-01"; };
in mkShell {
  nativeBuildInputs = [
    maturin
    rust.rust
  ];
}
