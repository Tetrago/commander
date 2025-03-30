{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          rustToolchain
          pkg-config
          gtk4
          libadwaita
        ];

        RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
      };

      packages.${system}.default = pkgs.stdenv.mkDerivation rec {
        pname = "commander";
        version = "0.1.0";

        src = ./.;

        cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
          inherit src pname version;
          hash = "sha256-VKh+iI3w6K+dTxK5DL+7vqKQBMaGQFZZbjS38ONLYSg=";
        };

        nativeBuildInputs = with pkgs; [
          rustToolchain
          meson
          ninja
          blueprint-compiler
          pkg-config
          rustPlatform.cargoSetupHook
          rustPlatform.bindgenHook
          wrapGAppsHook4
        ];

        buildInputs = with pkgs; [
          gtk4
          libadwaita
          pipewire
        ];
      };
    };
}
