{
  description = "Provides basic Rust toolchain support.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    wrangler = {
      url = "github:ulrikstrid/nix-wrangler";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.18.tar.gz";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = { self, nixpkgs, wrangler, rust-overlay, crane, nix-filter, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
          config.allowUnfreePredicate = pkg: builtins.elem (pkgs.lib.getName pkg) [
            "surrealdb"
          ];
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
            "media"
          ];
        };

        toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default);
        dev-toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        });

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        common-args = {
          inherit src;
          strictDeps = true;

          pname = "nika";
          version = "0.1";

          buildInputs = [];
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly common-args;

        individual-crate-args = crate-name: common-args // {
          inherit cargoArtifacts;
          pname = crate-name;
          cargoExtraArgs = "-p ${crate-name}";
          doCheck = false;
        };

        build-crate = name: craneLib.buildPackage (individual-crate-args name);

        crates = {
          fetcher = build-crate "fetcher";
          api = build-crate "api";
          daemon = build-crate "daemon";
        };

      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            (dev-toolchain pkgs)

            bacon # change detection
            cargo-nextest # testing
            cargo-deny # package auditing

            # cf worker deployment
            yarn
            wrangler.packages.${system}.wrangler
            worker-build
            wasm-pack

            surrealdb surrealdb-migrations

            redis
          ];
        };
        packages = {
          inherit (crates) fetcher api daemon;
        };
        checks = {
          clippy = craneLib.cargoClippy (common-args // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          docs = craneLib.cargoDoc (common-args // {
            inherit cargoArtifacts;
          });
          nextest = craneLib.cargoNextest (common-args // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
          fmt = craneLib.cargoFmt common-args;
          deny = craneLib.cargoDeny common-args;

          inherit (crates) fetcher api daemon;
        };
      });
}
