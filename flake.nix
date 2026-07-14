{
  description = "ASCII Cube WebAssembly Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Provide a rust toolchain with WASM target
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };

        # Override rustPlatform to use our specific toolchain
        rustPlatform = pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        };
      in
      {
        packages.default = rustPlatform.buildRustPackage {
          pname = "ascii-cube-wasm";
          version = "0.1.0";
          src = ./.;

          # Tells rustPlatform to manage cargo dependencies using Cargo.lock
          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = [ pkgs.wasm-pack pkgs.wasm-bindgen-cli ];

          # We override the build phase to use wasm-pack
          # Because rustPlatform fetches dependencies, wasm-pack will use the offline cache!
          buildPhase = ''
            export HOME=$(mktemp -d)
            wasm-pack build --target web --mode no-install --release
          '';

          # The output of wasm-pack is the `pkg` directory. We copy it to $out
          installPhase = ''
            mkdir -p $out
            cp -r pkg/* $out/
          '';

          doCheck = false;
        };

        # A devShell to manually run `wasm-pack` if needed
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
            pkgs.wasm-pack
          ];
        };
      }
    );
}
