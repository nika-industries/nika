localFlake: { ... }: {
  perSystem = { pkgs, config, ... }: let
    # run `cmp` to compare config.packages.crate-graph-image with ./media/crate-graph.png
    crate-graph-image-is-updated = pkgs.stdenv.mkDerivation {
      src = ../../media;
      name = "crate-graph-image-is-updated";
      # buildInputs = [ pkgs.cmp ];
      buildPhase = ''
        if cmp ${config.packages.crate-graph-image}/crate-graph.png crate-graph.png; then
          echo "Crate graph image is up to date.";
        else
          echo "Crate graph image is outdated. Run 'update-crate-graph' to update it.";
          exit 1;
        fi
      '';
      installPhase = ''
        mkdir -p $out
      '';
    };
  in {
    checks = {
      inherit crate-graph-image-is-updated;
    };
  };
}
