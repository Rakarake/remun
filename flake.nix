{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
      flake-utils.url = "github:numtide/flake-utils";
      rust-overlay.url = "github:oxalica/rust-overlay";
    };
  
  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        muslPkgs = pkgs.pkgsMusl;
      in
      {
        devShell = pkgs.mkShell rec {
          LD_LIBRARY_PATH = "${muslPkgs.lib.makeLibraryPath buildInputs}";
          packages = with muslPkgs; [ 
            # the rust package
            ((rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)).override {
              targets = [ "x86_64-unknown-linux-musl" ];
            })
            # cc without glibc
            #musl.dev
            gcc
            #(wrapCCWith {
            #  # could use gcc.cc
            #  cc = pkgs.llvmPackages.clang.cc;
            #  bintools = wrapBintoolsWith {
            #    bintools = pkgs.llvmPackages.bintools-unwrapped;
            #    libc = musl;
            #  };
            #})
          ];
          buildInputs = with muslPkgs; [
            libdisplay-info
            libgbm
            #mesa
            libinput
            pixman
            seatd
            udev
            libxkbcommon
            wayland
            wayland.dev
            wayland-protocols
            libGL
            #libglvnd
          ];
        };
      }
    );
}
