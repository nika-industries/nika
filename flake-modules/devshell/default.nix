localFlake: { ... }: {
  perSystem = { pkgs, inputs', config, ... }: let
    mkShell = pkgs.devshell.mkShell;
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
        
        # we don't use these from the shell but we keep them here to avoid
        #   garbage collection for the docker images
        config.packages.tikv-server config.packages.pd-server
      ];

      motd = "

    ######################################################################
    #                                                                    #
    #                      '0WWk.  ;KWX0:                                #
    #                      .d00l.  ;XMNKc                                #
    #                        ..    ;XMNKc                                #
    #    cc;..;lodol;.      ,cc'   ;XMNKc      ':cc,     .;coodol:'      #
    #    MMXk0XXXXWMWKc.   .OMMd.  ;XMNKc    'dXWXd'   .oKWWK00KNWNk;    #
    #    MMMKo,...;xNMNl   .OMMd.  ;XMNKc  'dXWXd'    .kWWO:.  .'xNMX    #
    #    MMNc      .xMMO.  .OMMd.  ;XMNKl'oXWXx'      .,:;.     .cXMM    #
    #    MMK,       oWM0'  .OMMd.  ;XMWNXXMMNo.         .,cldxkO0XWMM    #
    #    MMK,       oWM0'  .OMMd.  ;XMMMW0ONMXo.      .oKWNKkxolcoXMM    #
    #    MMK,       oWM0'  .OMMd.  ;XMWXd..:0WWO;     oWMKc.     ,KMM    #
    #    MMK,       oWM0'  .OMMd.  ;XMNKc   .dXMNd.   oWMK;     ,kWMM    #
    #    MMK,       oWM0'  .OMMd.  ;KMNKc     ;OWW0c. .kNWXOxxxO00NMM    #
    #    NWO'       lNNO.  .xNNo.  ,0WKO:      'kNNK:  .l0WMMMW0:;ONN    #
    #                                                                    #
    ######################################################################
 
    Welcome to the nika dev shell! Below are some commands you can run.
$(type -p menu &>/dev/null && menu)
      ";

      commands = let
        # note; there's a UTF-8 control character in the esc string below
        esc = "";
        boldRed = s: "${esc}[31;1m${s}${esc}[0m";
        perBinaryCommands = binary: [
          {
            name = binary;
            command = "cargo run --bin ${binary}";
            help = "Run the `${boldRed binary}` binary";
            category = "[local binary actions]";
          }
          {
            name = "${binary}-release";
            command = "cargo run --release --bin ${binary}";
            help = "Run the `${boldRed binary}` binary in release mode";
            category = "[local binary actions]";
          }
          {
            name = "${binary}-watch";
            command = "bacon -j run -- --bin ${binary}";
            help = "Watch for changes and run the `${boldRed binary}` binary";
            category = "[local binary actions]";
          }
        ];
        dockerLoad = imageName: "docker load -i ${imageName}";
        ephemeralDockerCommand = { imageName, imageVersion }: {
          name = "run-${imageName}";
          command = ''
            ${dockerLoad config.packages."${imageName}-image"} \
            && docker run --rm --network host ${imageName}-server:${imageVersion}
          '';
          help = "Run the ${boldRed imageName} server in an ephemeral container";
          category = "[docker actions]";
        };
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

        (ephemeralDockerCommand { imageName = "tikv"; imageVersion = "8.1.1"; })
        (ephemeralDockerCommand { imageName = "pd"; imageVersion = "8.1.1"; })

        {
          name = "tikv";
          command = "mprocs \"run-tikv\" \"run-pd\"";
          help = "Run the ${boldRed "tikv"} stack";
          category = "[stack actions]";
        }
        {
          name = "stack";
          command = "mprocs \"run-tikv\" \"run-pd\" \"redis-server\" \"fetcher\" \"api\"";
          help = "Run the whole stack";
          category = "[stack actions]";
        }
        {
          name = "stack-release";
          command = "mprocs \"run-tikv\" \"run-pd\" \"redis-server\" \"fetcher-release\" \"api-release\"";
          help = "Run the whole stack in release mode";
          category = "[stack actions]";
        }
      ]
        ++ perBinaryCommands "fetcher"
        ++ perBinaryCommands "api"
        ++ perBinaryCommands "daemon";
    };
  };
}
