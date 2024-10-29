{ withSystem }: { pkgs, lib, config, ... }: let
  module-lib = (import ./module-lib.nix) { inherit lib; };
in {
  options = {
    services.api = {
      enable = lib.mkEnableOption "API";

      package = lib.mkOption {
        type = lib.types.package;
        default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }:
          config.packages.api
        );
        description = "The API package to use.";
      };

      tikvUrls = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "The list of TiKV PD server URLs to use. They must be specified without a protocol, e.g. `localhost:2379`.";
      };

      address = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "The address to listen on for API requests.";
      };

      port = lib.mkOption {
        type = lib.types.nullOr lib.types.int;
        default = null;
        description = "The port to listen on for API requests.";
      };

      mockTempStorage = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = "Whether to mock temp storage within the API binary.";
      };

      logLevel = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "The log level to pass to `RUST_LOG`.";
      };
    };
  };

  config = let
    cfg = config.services.api;

    optional-flag = module-lib.optional-flag cfg;

    command = pkgs.writeShellScript "api" ''
      ${cfg.package}/bin/api \
        ${pkgs.lib.optionalString cfg.mockTempStorage "--mock-temp-storage"} \
        start \
        ${optional-flag "address" "address"} \
        ${optional-flag "port" "port"} \
    '';
    actualPort = if cfg.port != null then cfg.port else 3000;
  in lib.mkIf cfg.enable {
    networking.firewall.allowedTCPPorts = [ actualPort ];
    systemd.services.api = {
      description = "API Server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = command;
        Restart = "on-failure";
      };

      environment = {
        TIKV_URLS = lib.concatStringsSep "," cfg.tikvUrls;
      };
    };
  };
}
    
