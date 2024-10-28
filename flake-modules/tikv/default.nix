localFlake: { ... }: {
  perSystem = { system, pkgs, ... }: let
    makeTikvBinary = (import ./makeTikvBinary.nix) { inherit system pkgs; };
    makeTikvDockerImage = (import ./makeTikvDockerImage.nix) { inherit pkgs; };
  in {
    packages = {
      tikv = makeTikvBinary "tikv";
      pd = makeTikvBinary "pd";
    };
    images = {
      tikv = makeTikvDockerImage {
        binary = makeTikvBinary "tikv";
        pname = "tikv";
        version = "8.1.1";
      };
      pd = makeTikvDockerImage {
        binary = makeTikvBinary "pd";
        pname = "pd";
        version = "8.1.1";
      };
    };
  };
}
