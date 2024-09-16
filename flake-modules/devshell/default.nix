localFlake: { ... }: {
  perSystem = ps @ { pkgs, inputs', config, ... }: let
    mkShell = pkgs.devshell.mkShell;

    # note; there's a UTF-8 control character in the esc string below
    esc = "";
    # for highlighting binary names in the help text
    bin-hl = s: "${esc}[31;1m${s}${esc}[0m";
  in {
    devShells.default = mkShell {
      packages = with pkgs; [
        # pull in the rust toolchain from the `rust-builds` module
        config.packages.dev-toolchain

        # libraries used in local rust builds
        pkg-config
        openssl

        # other things used in local rust builds
        gcc

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
        redis
      ];

      motd = "\n  Welcome to the {2}nika{reset} dev shell. Run {1}menu{reset} for commands.\n";

      commands = let
        import-commands-module = path: (import path) (ps // { inherit bin-hl; });
      in [
        {
          name = "test";
          command = "cargo nextest run";
          help = "Run tests with nextest";
          category = "[testing]";
        }
        {
          name = "test-all";
          command = "cargo nextest run --run-ignored all";
          help = "Run all tests, including ones that require other services";
          category = "[testing]";
        }
        {
          name = "clippy";
          command = "cargo clippy --all-targets";
          help = "Run clippy on all targets";
          category = "[cargo actions]";
        }
        {
          name = "check";
          command = "nix flake check -L";
          help = "Run nix checks";
          category = "[nix actions]";
        }
      ]
        ++ import-commands-module ./bin-commands.nix
        ++ import-commands-module ./docker-commands.nix
        ++ import-commands-module ./stack-commands.nix;
    };
  };
}
