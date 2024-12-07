{ lib, flake-parts-lib, ... }: let
  inherit (lib) mkOption types;

  mkFlakeOutput = { name, desc }:
    flake-parts-lib.mkTransposedPerSystemModule {
      name = name;
      option = mkOption {
        type = types.lazyAttrsOf types.package;
        default = { };
        description = desc;
      };
      file = ./default.nix;
    };

  images-output = mkFlakeOutput {
    name = "images";
    desc = ''
      An attribute set of derivations that produce tarballs to be used as OCI images.
    '';
  };
  kubernetes-output = mkFlakeOutput {
    name = "kubernetes";
    desc = ''
      An attribute set of derivations that produce tarballs to be used as OCI images.
    '';
  };
in {
  imports = [ images-output kubernetes-output ];
}
