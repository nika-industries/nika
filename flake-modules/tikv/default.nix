localFlake: { ... }: {
  perSystem = { system, pkgs, ... }: let

    makeBinary = name: let
      pname = "${name}-server";
      version = "8.1.1";

      arch = { x86_64-linux = "amd64"; aarch64-linux = "arm64"; }.${system};
      url = "https://download.pingcap.org/tidb-community-server-v${version}-linux-${arch}.tar.gz";

      full-archive = pkgs.fetchzip {
        url = url;
        hash = "sha256-ZtFqm4PllBRIGiRLzBynWvdcmegXD8WMPzknXwJYKBg=";
      };
    in pkgs.stdenv.mkDerivation {
      inherit pname version;
    
      src = "${full-archive}/${name}-v${version}-linux-${arch}.tar.gz";

      nativeBuildInputs = [ pkgs.autoPatchelfHook ];
      buildInputs = [ pkgs.glibc pkgs.libgcc ];

      dontUnpack = true;
      buildPhase = ''
        tar -xzf $src
        autoPatchelf $pname/bin/
      '';
   
      installPhase = ''
        mkdir -p $out/bin
        cp ${pname} $out/bin/${pname}
      '';
    };

    makeDockerImage = binary: let
    in pkgs.dockerTools.buildLayeredImage {
      name = "${binary.pname}-server";
      tag = binary.version;

      contents = [ pkgs.tzdata ];

      config = {
        Cmd = [ "${binary}/bin/${binary.pname}-server" ];
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
    };

  in {
    packages = {
      tikv = makeBinary "tikv";
      pd = makeBinary "pd";
    };
    images = {
      tikv = makeDockerImage "tikv";
      pd = makeDockerImage "pd";
    };
  };
}
