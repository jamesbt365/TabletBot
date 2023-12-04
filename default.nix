{ lib
, rustPlatform
, pkg-config
, openssl
, fetchurl
, ...
}:

rustPlatform.buildRustPackage rec {
  pname = "tabletbot";
  name = pname;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
  ];

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "poise-0.5.7" = "sha256-vI4FgRorQyv2FcrHI/hE6v/ISTAxOnenIQlt/mFQ4so=";
    };
  };
}
