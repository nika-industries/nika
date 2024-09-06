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
    mkshell-minimal.url = "github:viperML/mkshell-minimal";
  };

  outputs = { nixpkgs, wrangler, rust-overlay, crane, nix-filter, mkshell-minimal, flake-utils, ... }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        mkShell = mkshell-minimal pkgs;
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

        toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
          extensions = [ "rustfmt" "clippy" ];
        });
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
          doCheck = false;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = with pkgs; [
            openssl
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly common-args;

        individual-crate-args = crate-name: common-args // {
          inherit cargoArtifacts;
          pname = crate-name;
          cargoExtraArgs = "-p ${crate-name}";
        };

        build-crate = name: craneLib.buildPackage (individual-crate-args name);

        crates = {
          fetcher = build-crate "fetcher";
          api = build-crate "api";
          daemon = build-crate "daemon";
        };

        tikv = (import ./nix/tikv.nix) { inherit pkgs; };
      in {
        devShells.default = mkShell {
          nativeBuildInputs = with pkgs; [
            # toolchain with the current pkgs
            (dev-toolchain pkgs)

            # libraries used in local rust builds
            pkg-config
            openssl

            # dev tools
            mprocs # parallel process execution
            bacon # change detection
            cargo-nextest # testing
            cargo-deny # package auditing

            # cf worker deployment
            yarn
            wrangler.packages.${system}.wrangler
            worker-build
            wasm-pack

            # service runtimes
            # redis
            # we don't use these directly but we keep them here to avoid
            # garbage collection for the docker images
            tikv.tikv-server tikv.pd-server
          ];
        };
        packages = {} // crates // tikv;
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
