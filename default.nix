{ pkgs ? import (import ./nix/sources.nix).nixpkgs {}
, python3Packages ? pkgs.python3Packages
}:

let
  sources = import ./nix/sources.nix;
  danieldk = pkgs.callPackage sources.danieldk {};
  mozilla = pkgs.callPackage "${sources.mozilla}/package-set.nix" {};
  rustc = (mozilla.rustChannelOf { channel = "nightly"; date = "2020-04-01"; }).rust;
  sticker = pkgs.callPackage sources.sticker {};
  libtorch = danieldk.libtorch.v1_5_0;
  crateOverrides = with pkgs; defaultCrateOverrides // {
    pyo3 = attr: {
      buildInputs = [ python3Packages.python ];
    };

    sentencepiece-sys = attr: {
      nativeBuildInputs = [ pkgconfig ];

      buildInputs = [
        (sticker.sentencepiece.override {
          withGPerfTools = false; }).dev
      ];
    };

    sticker2-python = attr: rec {
      pname = "sticker2-python";
      name = "${pname}-${attr.version}";

      buildInputs = [ libtorch ] ++
        lib.optional stdenv.isDarwin darwin.Security;

      features = [];

      installPhase = let
        sitePackages = python3Packages.python.sitePackages;
        sharedLibrary = stdenv.hostPlatform.extensions.sharedLibrary;
      in ''
        runHook preInstall

        mkdir -p "$out/${sitePackages}"
        cp target/lib/libsticker2_python*${sharedLibrary} \
          "$out/${sitePackages}/sticker2.so"
        export PYTHONPATH="$out/${sitePackages}:$PYTHONPATH"

        runHook postInstall
      '';
    };

    torch-sys = attr: {
      buildInputs = lib.optional stdenv.isDarwin curl;

      LIBTORCH = "${libtorch.dev}";
    };
  };
  buildRustCrate = pkgs.buildRustCrate.override {
    inherit rustc;

    defaultCrateOverrides = crateOverrides;
  };
  crateTools = import "${sources.crate2nix}/tools.nix" {};
  cargoNix = pkgs.callPackage (crateTools.generatedCargoNix {
    name = "sticker2";
    src = pkgs.nix-gitignore.gitignoreSource [ ".git/" "nix/" "*.nix" ] ./.;
  }) {
    inherit buildRustCrate;
  };
in cargoNix.rootCrate.build
