{ lib
, rustPlatform
}:

rustPlatform.buildRustPackage rec {
  pname = "tabletbot";
  name = pname;

  src = ./src;

  cargoLock = {
    lockFile = ./src/Cargo.lock;
  };
}
