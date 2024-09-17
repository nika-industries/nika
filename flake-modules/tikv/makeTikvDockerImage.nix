# this file is a function, which when called produces the function makeDockerImage.

{ pkgs, ... }: { binary, pname, version }: pkgs.dockerTools.buildLayeredImage {
  name = "${pname}-server";
  tag = version;

  contents = [ pkgs.tzdata ];

  config = {
    Cmd = [ "${binary}/bin/${pname}-server" ];
    Entrypoint = [ "${pkgs.tini}/bin/tini" ];
    Env = [
      "RUST_BACKTRACE=1"
      "TZ=Etc/UTC"
    ];
  };

  # tikv and pd need to use the tmp directory, but nothing expects to have
  # to create it.
  extraCommands = ''
    mkdir tmp/
  '';
}
