{ lib
, rustPlatform
, pkg-config
, openssl
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
  };
}
