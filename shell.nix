{ pkgs ? import <nixpkgs> { } }:
with pkgs; mkShell rec {
  nativeBuildInputs = [
    pkg-config
  ];
  buildInputs = [
    udev alsa-lib vulkan-loader
    # To use the x11 feature
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr 
    # To use the wayland feature
    libxkbcommon wayland
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
