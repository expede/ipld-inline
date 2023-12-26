{
  description = "inline_ipld";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-23.11";
    nixos-unstable.url = "nixpkgs/nixos-unstable-small";

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
      nixos-unstable,
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

        unstable = import nixos-unstable {
          inherit system;
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
          unstable.cargo-component
          pkgs.cargo-criterion
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
          name = "inline_ipld";
          packages = [
            # For nightly rustfmt to be used instead of the rustfmt provided by `rust-toolchain`, it must appear first in the list
            # nightly-rustfmt
            rust-toolchain
            self.packages.${system}.irust
            pkgs.sccache

            unstable.wasmtime
            unstable.nodejs_20
            pkgs.binaryen

            pkgs.wasm-pack
            pkgs.chromedriver
          ]
          ++ format-pkgs
          ++ cargo-installs
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin darwin-installs;

          env = [
            {
              name  = "RUSTC_WRAPPER";
              value =  "${pkgs.sccache}/bin/sccache";
            }
          ];

          commands = [
            {
              name     = "release";
              help     = "[DEFAULT] Release (optimized build) for current native target";
              category = "release";
              command  = "release:native";
            }
            {
              name     = "release:native";
              help     = "Release for current native target";
              category = "release";
              command  = "${pkgs.cargo}/bin/cargo build --release -p inline_ipld";
            }
            {
              name     = "release:wasm";
              help     = "Release for current native target";
              category = "release";
              command  = "${pkgs.cargo}/bin/cargo build --release -p inline_ipld_wasm";
            }
            # Build
            {
              name     = "build";
              help     = "[DEFAULT] Build for current native target";
              category = "build";
              command  = "build:native";
            }
            {
              name     = "build:native";
              help     = "Build for current native target";
              category = "build";
              command  = "${pkgs.cargo}/bin/cargo build -p inline_ipld";
            }
            {
              name     = "build:wasm";
              help     = "Build for wasm32-unknown-unknown";
              category = "build";
              command  = "${pkgs.cargo}/bin/cargo build -p inline_ipld_wasm --target=wasm32-unknown-unknown";
            }
            {
              name     = "build:wasi";
              help     = "Build for WASI";
              category = "build";
              command  = "${pkgs.cargo}/bin/cargo build --target wasm32-wasi";
            }
            # Bench
            {
              name     = "bench:native";
              help     = "Run native Criterion benchmarks";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo criterion -p inline_ipld";
            }
            {
              name     = "bench:native:open";
              help     = "Open native Criterion benchmarks in browser";
              category = "dev";
              command  = "${pkgs.xdg-utils}/bin/xdg-open ./target/criterion/report/index.html";
            }
            # Lint
            {
              name     = "lint";
              help     = "Run Clippy";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo clippy";
            }
            {
              name     = "lint:pedantic";
              help     = "Run Clippy pedantically";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo clippy -- -W clippy::pedantic";
            }
            {
              name     = "lint:fix";
              help     = "Apply non-pendantic Clippy suggestions";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo clippy --fix";
            }
            # Watch
            {
              name     = "watch:build:native";
              help     = "Rebuild native target on save";
              category = "watch";
              command  = "${pkgs.cargo}/bin/cargo watch --clear -C ./inline_ipld";
            }
            {
              name     = "watch:build:wasm";
              help     = "Rebuild native target on save";
              category = "watch";
              command  = "${pkgs.cargo}/bin/cargo watch --clear --features=serde -C ./inline_ipld_wasm";
            }
            {
              name     = "watch:lint";
              help     = "Lint on save";
              category = "watch";
              command  = "${pkgs.cargo}/bin/cargo watch --clear --exec clippy";
            }
            {
              name     = "watch:lint:pedantic";
              help     = "Pedantic lint on save";
              category = "watch";
              command  = "${pkgs.cargo}/bin/cargo watch --clear --exec 'clippy -- -W clippy::pedantic'";
            }
            {
              name     = "watch:test:native";
              help     = "Run all tests on save";
              category = "watch";
              command  = "${pkgs.cargo}/bin/cargo watch --clear --workdir ./inline_ipld --exec test";
            }
            # Test
            {
              name     = "test:all";
              help     = "Run Cargo tests";
              category = "test";
              command  = "test:native && test:docs && test:wasm";
            }
            {
              name     = "test:native";
              help     = "Run Cargo tests for native target";
              category = "test";
              command  = "${pkgs.cargo}/bin/cargo test -p inline_ipld";
            }
            {
              name     = "test:wasm";
              help     = "Run wasm-pack tests on all targets";
              category = "test";
              command  = "test:wasm:node && test:wasm:chrome";
            }
            {
              name     = "test:wasm:nodejs";
              help     = "Run wasm-pack tests in Node.js";
              category = "test";
              command  = "${pkgs.wasm-pack}/bin/wasm-pack test --node inline_ipld_wasm";
            }
            {
              name     = "test:wasm:chrome";
              help     = "Run wasm-pack tests in headless Chrome";
              category = "test";
              command  = "${pkgs.wasm-pack}/bin/wasm-pack test --headless --chrome inline_ipld_wasm";
            }
            {
              name     = "test:docs";
              help     = "Run Cargo doctests";
              category = "test";
              command  = "${pkgs.cargo}/bin/cargo test --doc";
            }
            # Docs
            {
              name     = "docs";
              help     = "[DEFAULT]: Open refreshed docs";
              category = "dev";
              command  = "docs:open";
            }
            {
              name     = "docs:build";
              help     = "Refresh the docs";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo doc";
            }
            {
              name     = "docs:open";
              help     = "Open refreshed docs";
              category = "dev";
              command  = "${pkgs.cargo}/bin/cargo doc --open";
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
