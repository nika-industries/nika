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
    devshell.url = "github:numtide/devshell";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } (top @ { ... }: {
    systems = [ "x86_64-linux" "aarch64-linux" ];
    debug = true;

    imports = let
      inherit (top.flake-parts-lib) importApply;
    in [
      # configures an "images" flake output for OCI images
      (importApply ./flake-modules/images-output { })
      # configures nixpkgs with overlays
      (importApply ./flake-modules/nixpkgs { })
      # builds tikv packages and images
      (importApply ./flake-modules/tikv { })
      # builds workspace rust packages
      (importApply ./flake-modules/rust-builds { })
      # defines e2e tests as nix checks
      (importApply ./flake-modules/e2e { })
      # defines devshell
      (importApply ./flake-modules/devshell { })
    ];
  });
}
