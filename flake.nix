{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      nixpkgs,
      systems,
      rust-overlay,
      ...
    }:
    let
      eachSystem = nixpkgs.lib.genAttrs (import systems);
    in
    {
      devShells = eachSystem (
        system:
        let
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
        pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            pkg-config
            gtk4
            libadwaita
            pipewire
            rustPlatform.bindgenHook
            blueprint-compiler
            wrapGAppsHook4
          ];

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
        }
      );

      packages = eachSystem (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        rec {
          commander = pkgs.rustPlatform.buildRustPackage rec {
            pname = "commander";
            version = "0.1.0";

            src = ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
              rustPlatform.bindgenHook
              blueprint-compiler
              wrapGAppsHook4
            ];

            buildInputs = with pkgs; [
              gtk4
              libadwaita
              pipewire
            ];

            useFetchCargoVendor = true;
            cargoHash = "sha256-0Ves6dNh6h8dcVc8iJ5Jd9w1ey9GDH4dDdEnCAapeAE=";

            meta.mainProgram = "commander";
          };

          default = commander;
        }
      );
    };
}
