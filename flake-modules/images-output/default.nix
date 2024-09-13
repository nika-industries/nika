localFlake: { lib, flake-parts-lib, ... }: let
  inherit (lib) mkOption types;
in flake-parts-lib.mkTransposedPerSystemModule {
  name = "images";
  option = mkOption {
    type = types.lazyAttrsOf types.package;
    default = { };
    description = ''
      An attribute set of derivations that produce tarballs to be used as OCI images.
    '';
  };
  file = ./default.nix;
}
