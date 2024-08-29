{ pkgs }: let
  version = "8.1.1";

  arch = {
    x86_64-linux = "amd64";
    aarch64-linux = "arm64";
  }.${pkgs.system};
  
  url = "https://download.pingcap.org/tidb-community-server-v${version}-linux-${arch}.tar.gz";

  full-archive = pkgs.fetchzip {
    url = url;
    hash = "sha256-ZtFqm4PllBRIGiRLzBynWvdcmegXD8WMPzknXwJYKBg=";
  };

  make-binary = name: let
    pname = "${name}-server";
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

  make-docker-image = name: let
    binary = make-binary name;
  in pkgs.dockerTools.buildLayeredImage {
    name = "${name}-server";
    tag = version;

    contents = [ pkgs.tzdata ];

    config = {
      Cmd = [ "${binary}/bin/${name}-server" ];
      Entrypoint = [ "${pkgs.tini}/bin/tini" ];
      Env = [
        "RUST_BACKTRACE=1"
        "TZ=Etc/UTC"
      ];
    };

    extraCommands = ''
      mkdir tmp/
    '';
  };

in {
  tikv-server = make-binary "tikv";
  tikv-image = make-docker-image "tikv";
  pd-server = make-binary "pd";
  pd-image = make-docker-image "pd";
}
