# this file is a function, which when called produces the function makeBinary.

{ pkgs, system, ... }: name: let
  pname = "${name}-server";
  version = "8.1.1";

  arch = { x86_64-linux = "amd64"; aarch64-linux = "arm64"; }.${system};
  url = "https://download.pingcap.org/tidb-community-server-v${version}-linux-${arch}.tar.gz";

  hashes = {
    aarch64-linux = "sha256-ZtFqm4PllBRIGiRLzBynWvdcmegXD8WMPzknXwJYKBg=";
    x86_64-linux = "sha256-CovqGP4nciRWfB+mGQcCP+VBkVOmC6hzRXO2gvXylpc=";
  };

  full-archive = pkgs.fetchzip {
    url = url;
    hash = hashes.${system};
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
}
