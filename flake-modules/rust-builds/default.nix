localFlake: { ... }: {
  perSystem = { system, pkgs, ... }: let
    filter = localFlake.inputs.nix-filter.lib;

    src = filter {
      root = ../../.;
      include = [
        "crates" "Cargo.toml" "Cargo.lock" # typical rust source
        ".cargo" # extra rust config
        (filter.matchExt "toml") # extra toml used by other projects
        "media" # static assets
      ];
    };

    toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal.override {
      extensions = [ "rustfmt" "clippy" ];
    });
    dev-toolchain = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
      extensions = [ "rust-src" "rust-analyzer" ];
      # targets = [ "wasm32-unknown-unknown" ];
    });

    craneLib = (localFlake.inputs.crane.mkLib pkgs).overrideToolchain toolchain;

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
  in {
    packages = {
      fetcher = build-crate "fetcher";
      api = build-crate "api";
      daemon = build-crate "daemon";
      toolchain = toolchain pkgs;
      dev-toolchain = dev-toolchain pkgs;
    };
    checks = {
      rust-cargo-clippy = craneLib.cargoClippy (common-args // {
        inherit cargoArtifacts;
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
      });
      rust-cargo-docs = craneLib.cargoDoc (common-args // {
        inherit cargoArtifacts;
        RUSTDOCFLAGS = "-D warnings";
      });
      rust-cargo-nextest = craneLib.cargoNextest (common-args // {
        inherit cargoArtifacts;
        partitions = 1;
        partitionType = "count";
      });
      rust-cargo-fmt = craneLib.cargoFmt common-args;
      rust-cargo-deny = craneLib.cargoDeny common-args;
    };
  };
}
