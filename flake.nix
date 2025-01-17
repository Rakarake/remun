{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
      flake-utils.url = "github:numtide/flake-utils";
    };
  
  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShell = pkgs.mkShell rec {
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
          packages = with pkgs; [ 
            rustc
            cargo
            gcc
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
