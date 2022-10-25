{
  description = "TabletBot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = attrs @ { self, nixpkgs, ... }: let

    system = "x86_64-linux";

    pkgs = import nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };

  in {

    packages.${system} = rec {
      tabletbot = pkgs.callPackage ./. {};
      default = tabletbot;
    };

    devShell.${system} = import ./shell.nix { inherit pkgs; };

  };
}
