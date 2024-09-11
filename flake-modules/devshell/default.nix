localFlake: { ... }: {
  perSystem = { pkgs, inputs', config, ... }: let
    mkShell = localFlake.inputs.mkshell-minimal pkgs;
  in {
    devShells.default = mkShell {
      nativeBuildInputs = with pkgs; [
        # pull in the rust toolchain from the `rust-builds` module
        config.packages.dev-toolchain

        # libraries used in local rust builds
        pkg-config
        openssl

        # dev tools
        mprocs # parallel process execution
        bacon # change detection
        cargo-nextest # testing
        cargo-deny # package auditing

        # cf worker deployment
        yarn
        inputs'.wrangler.packages.wrangler
        worker-build
        wasm-pack

        # service runtimes
        # redis
        # we don't use these directly but we keep them here to avoid
        # garbage collection for the docker images
        config.packages.tikv-server config.packages.pd-server
      ];
    };
  };
}
