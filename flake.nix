{
  description = "Provides basic Rust toolchain support.";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
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

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (top @ { ... }: {
    systems = [ "x86_64-linux" "aarch64-linux" ];

    debug = true;
    imports = let
      inherit (top.flake-parts-lib) importApply;
    in [
      (importApply ./flake-modules/nixpkgs { })
      (importApply ./flake-modules/tikv { })
      (importApply ./flake-modules/rust-builds { inherit inputs; })
    ];
    
    # args with a `prime` have the system pre-selected
    perSystem = { config, inputs', system, pkgs, ... }: let
      mkShell = inputs.mkshell-minimal pkgs;
    in {
      devShells.default = mkShell {
        nativeBuildInputs = with pkgs; [
          config.packages.dev-toolchain

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
          inputs'.wrangler.packages.wrangler
          worker-build
          wasm-pack

          # service runtimes
          # redis
          # we don't use these directly but we keep them here to avoid
          # garbage collection for the docker images
          config.packages.tikv-server config.packages.pd-server
        ];
      };
    };
  });
}
