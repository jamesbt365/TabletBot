# https://nixos.wiki/wiki/Rust#Installation_via_rustup
{ pkgs ? import <nixpkgs> { } }:
let
  readFileIfExists = path:
    with pkgs.lib;
    if pathExists path then readFile path else null;

  default = pkgs.callPackage ./. { };

  rustDeps = with pkgs; [
    llvmPackages_latest.llvm
    llvmPackages_latest.bintools
    zlib.out
    rustup
    xorriso
    grub2
    qemu
    llvmPackages_latest.lld
    python3
  ];

  deps = default.nativeBuildInputs;

  buildPath = toString ./.build/out;

  build = pkgs.writers.writeBashBin "build" ''
    cd ${toString ./src}
    cargo build $@
  '';

  run = pkgs.writers.writeBashBin "run" ''
    cd ${toString ./src}
    cargo run $@
  '';

  utils = with pkgs; [
    build # cargo build
    run # cargo run
    cachix
    jq
  ];

in pkgs.mkShell rec {
  RUSTC_VERSION = readFileIfExists ./rust-toolchain;
  CARGO_HOME = toString ./.build/cargo;
  CARGO_TARGET_DIR = toString ./.build/target;
  RUSTUP_HOME = toString ./.build/rustup;
  TABLETBOT_DATA = toString ./.build/data;

  buildInputs = with pkgs; rustDeps ++ deps ++ utils;

  shellHook = with pkgs.lib; ''
    export LD_LIBRARY_PATH=${escapeShellArg (makeLibraryPath buildInputs)}
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
    export PATH=$PATH:${escapeShellArg (makeBinPath buildInputs)}

    [ -r ${TABLETBOT_DATA}/tokens ] && source ${TABLETBOT_DATA}/tokens
  '';
}
