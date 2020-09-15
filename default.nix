{
  pkgs ? import (import ./nix/sources.nix).nixpkgs {
    config = {
      allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
        "libtorch"
      ];
    };
  }

, python3Packages ? pkgs.python3Packages
}:

let
  sources = import ./nix/sources.nix;
  libtorch = pkgs.libtorch-bin;
  crateOverrides = with pkgs; defaultCrateOverrides // {
    pyo3 = attr: {
      buildInputs = [ python3Packages.python ];
    };

    sentencepiece-sys = attr: {
      nativeBuildInputs = [ pkgconfig ];

      buildInputs = [ sentencepiece ];
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
    defaultCrateOverrides = crateOverrides;
  };
  crateTools = pkgs.callPackage "${sources.crate2nix}/tools.nix" {};
  cargoNix = pkgs.callPackage (crateTools.generatedCargoNix {
    name = "sticker2";
    src = pkgs.nix-gitignore.gitignoreSource [ ".git/" "nix/" "*.nix" ] ./.;
  }) {
    inherit buildRustCrate;
  };
in cargoNix.rootCrate.build
