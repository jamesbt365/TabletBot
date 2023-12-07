{ pkgs ? nixpkgs, nixpkgs ? import <nixpkgs> {}, packages, ... }:

pkgs.mkShell rec {
  inputsFrom = [ packages.tabletbot ];

  buildInputs = with pkgs; [
    cargo
    cachix
    jq
  ];

  shellHook = with pkgs.lib; ''
    PROJECT_ROOT="$(git rev-parse --show-toplevel)"

    export CARGO_HOME="$PROJECT_ROOT/.build/cargo"
    export CARGO_TARGET_DIR="$PROJECT_ROOT/.build/target"
    export TABLETBOT_DATA="$PROJECT_ROOT/.build/data"

    export LD_LIBRARY_PATH=${escapeShellArg (makeLibraryPath buildInputs)}
    export PATH=$PATH:${escapeShellArg (makeBinPath buildInputs)}
  '';
}
