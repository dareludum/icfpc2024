{
  description = "A Nix flake dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    alejandra = {
      url = "github:kamadorueda/alejandra/3.0.0";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    fenix,
    alejandra,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [fenix.overlays.default];
        };

        # Rust environment
        rustVer = fenix.packages.${system}.stable;
        rustChan = rustVer.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];
      in
        with pkgs; {
          devShells.default = mkShell {
            buildInputs = [
              rustChan

              cmake
              mesa libGLU glfw
              xorg.libX11 xorg.libXi xorg.libXcursor xorg.libXext xorg.libXrandr xorg.libXinerama
              wayland.dev
              libpulseaudio
              pkg-config
              openssl
            ];
            LIBCLANG_PATH = "${lib.getLib llvmPackages.libclang}/lib";
          };
        }
    );
}
