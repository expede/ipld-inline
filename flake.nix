{
  description = "ipld-inline";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    devshell.url    = "github:numtide/devshell";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    devshell,
    rust-overlay,
  } @ inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            devshell.overlays.default
            (import rust-overlay)
          ];
        };

        rust-toolchain =
          (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
            extensions = [
              "cargo"
              "clippy"
              "llvm-tools-preview"
              "rust-src"
              "rust-std"
              "rustfmt"
            ];

            targets = [
              "wasm32-unknown-unknown"
              "wasm32-wasi"
            ];
          };

        nightly-rustfmt = pkgs.rust-bin.nightly.latest.rustfmt;

        format-pkgs = [
          pkgs.nixpkgs-fmt
          pkgs.alejandra
        ];

        darwin-installs = [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.CoreFoundation
          pkgs.darwin.apple_sdk.frameworks.Foundation
        ];

        cargo-installs = [
          pkgs.cargo-bootimage
          pkgs.cargo-deny
          pkgs.cargo-expand
          pkgs.cargo-outdated
          pkgs.cargo-sort
          pkgs.cargo-udeps
          pkgs.cargo-watch
          pkgs.llvmPackages.bintools
          pkgs.twiggy
          pkgs.wasm-tools
        ];

      in rec {
        devShells.default = pkgs.devshell.mkShell {
          name = "ipld-inline";
          packages =
            [
              # For nightly rustfmt to be used instead of the rustfmt provided by `rust-toolchain`, it must appear first in the list
              # nightly-rustfmt
              rust-toolchain

              pkgs.wasmtime
              self.packages.${system}.irust
            ]
            ++ format-pkgs
            ++ cargo-installs
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwin-installs;

          commands = [
            {
              name     = "build:*";
              help     = "Build for current native target";
              category = "task";
              command  = "${pkgs.cargo}/bin/cargo build";
            }
            {
              name     = "build:wasm";
              help     = "Build for wasm32-unknown-unknown";
              category = "task";
              command  = "${pkgs.cargo}/bin/cargo build --target=wasm32-unknown-unknown";
            }
            {
              name     = "build:wasi";
              help     = "Build for WASI";
              category = "task";
              command  = "${pkgs.cargo}/bin/cargo build --target wasm32-wasi";
            }
          ];
        };

        packages.irust = pkgs.rustPlatform.buildRustPackage rec {
          pname   = "irust";
          version = "1.65.1";
          src     = pkgs.fetchFromGitHub {
            owner  = "sigmaSd";
            repo   = "IRust";
            rev    = "v${version}";
            sha256 = "sha256-AMOND5q1XzNhN5smVJp+2sGl/OqbxkGPGuPBCE48Hik=";
          };

          doCheck     = false;
          cargoSha256 = "sha256-A24O3p85mCRVZfDyyjQcQosj/4COGNnqiQK2a7nCP6I=";
        };
    });
}
