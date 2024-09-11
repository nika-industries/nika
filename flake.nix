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

    imports = let
      inherit (top.flake-parts-lib) importApply;
    in [
      (importApply ./flake-modules/nixpkgs { })
      (importApply ./flake-modules/tikv { })
      (importApply ./flake-modules/rust-builds { inherit inputs; })
      (importApply ./flake-modules/devshell { inherit inputs; })
    ];
  });
}
