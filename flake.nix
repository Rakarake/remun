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
      in
      {
        devShell = pkgs.mkShell rec {
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          packages = with pkgs; [ 
            # the rust package
            ((rust-bin.selectLatestNightlyWith (toolchain: toolchain.default)).override {
              targets = [ "x86_64-unknown-linux-musl" ];
            })
            # cc without glibc
            (wrapCCWith {
              # could use gcc.cc
              cc = clang.cc;
              bintools = wrapBintoolsWith {
                bintools = binutils-unwrapped;
                libc = musl;
              };
            })
          ];
          buildInputs = with pkgs; [
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
