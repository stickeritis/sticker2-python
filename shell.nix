with import <nixpkgs> {};

let
  sources = import ./nix/sources.nix;
  libtorch = libtorch-bin;
in mkShell {
  nativeBuildInputs = [
    cargo
    maturin
    pkgconfig
  ];

  buildInputs = [
    libtorch
    openssl
    python3
    sentencepiece
  ];

  LIBTORCH = "${libtorch.dev}";
}
