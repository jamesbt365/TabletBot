{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, fenix, flake-utils, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      packages.default = let
        toolchain = fenix.packages.${system}.minimal.toolchain;
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };

      in (pkgs.makeRustPlatform {
        cargo = toolchain;
        rustc = toolchain;
      }).buildRustPackage {
        pname = "tabletbot";
        version = "0.1.0";

        src = ./.;
        nativeBuildInputs = with pkgs; [ pkg-config ];

        buildInputs = with pkgs; [ openssl ];

        cargoLock = {
          lockFile = ./Cargo.lock;
          outputHashes = {
            "poise-0.5.7" =
              "1jp2a1hzwv8946kpffii614wizza8kw27iya2pv2nhrb3a0hb3mw";
          };
        };
      };
    });
}
