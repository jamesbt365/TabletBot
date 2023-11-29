{ lib, rustPlatform, pkg-config, openssl }:

        (pkgs.makeRustPlatform {
          cargo = toolchain;
          rustc = toolchain;
        }).buildRustPackage {
          pname = "example";
          version = "0.1.0";
