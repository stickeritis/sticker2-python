with import <nixpkgs> {};

let
  sources = import ./nix/sources.nix;
  danieldk = callPackage sources.danieldk {};
  mozilla = callPackage "${sources.mozilla}/package-set.nix" {};
  rust = mozilla.rustChannelOf { channel = "nightly"; date = "2020-04-01"; };
  sticker = callPackage sources.sticker {};
  libtorch = danieldk.libtorch.v1_5_0;
in mkShell {
  nativeBuildInputs = [
    maturin
    pkgconfig
    rust.rust
  ];

  buildInputs = [
    libtorch
    openssl
    python3
    sticker.sentencepiece
  ];

  LIBTORCH = "${libtorch.dev}";
}
