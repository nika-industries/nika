{
  description = "Provides basic Rust toolchain support.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.17.tar.gz";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = { self, nixpkgs, rust-overlay, crane, nix-filter, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        filter = nix-filter.lib;

        src = filter {
          root = ./.;
          include = [
            "crates"
            "Cargo.toml"
            "Cargo.lock"
            (filter.matchExt "toml")
            ".cargo"
          ];
        };

        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        dev-toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        });

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        fetcher-crane-args = {
          inherit src;
          strictDeps = true;

          pname = "fetcher";
          version = "0.1.0";

          buildInputs = [];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        fetcher-deps-only = craneLib.buildDepsOnly fetcher-crane-args;
        fetcher = craneLib.buildPackage (fetcher-crane-args // {
          cargoArtifacts = fetcher-deps-only;
        });
        
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            dev-toolchain
            bacon # change detection
            cargo-nextest # testing
            cargo-deny # package auditing
          ];
        };
        packages = {
          inherit fetcher;
        };
        checks = {
          clippy = craneLib.cargoClippy (fetcher-crane-args // {
            cargoArtifacts = fetcher-deps-only;
            cargoClippyExtraArgs = "-- --deny warnings";
          });
          docs = craneLib.cargoDoc (fetcher-crane-args // {
            cargoArtifacts = fetcher-deps-only;
          });
          site-server-nextest = craneLib.cargoNextest (fetcher-crane-args // {
            cargoArtifacts = fetcher-deps-only;
            partitions = 1;
            partitionType = "count";
          });
          fmt = craneLib.cargoFmt {
            inherit (fetcher-crane-args) pname version;
            inherit src;
          };
          deny = craneLib.cargoDeny {
            inherit (fetcher-crane-args) pname version;
            inherit src;
          };
          build-succeeds = fetcher;
        };
      });
}
