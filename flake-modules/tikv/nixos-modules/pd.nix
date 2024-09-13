{ withSystem }: { pkgs, lib, config, ... }: let
  module-lib = (import ./module-lib.nix) { inherit lib; };
in {
  options = {
    services.pd = {
      enable = lib.mkEnableOption "PD";
      package = lib.mkOption {
        type = lib.types.package;
        default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }:
          config.packages.pd
        );
        description = "The PD package to use.";
      };

      clientUrls = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "The list of client URLs to be listened to by PD";
      };

      advertiseClientUrls = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = config.services.pd.clientUrls;
        description = "The list of advertise URLs for the client to access PD. For example, the internal IP address of Docker is 172.17.0.1, while the IP address of the host is 192.168.100.113 and the port mapping is set to -p 2379:2379. In this case, you can set --advertise-client-urls to \"http://192.168.100.113:2379\". The client can find this service through \"http://192.168.100.113:2379\".";
      };

      peerUrls = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "The list of peer URLs to be listened to by PD";
      };

      advertisePeerUrls = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = config.services.pd.peerUrls;
        description = "The list of advertise URLs for the peer to access PD. For example, the internal IP address of Docker is 172.17.0.1, while the IP address of the host is 192.168.100.113 and the port mapping is set to -p 2379:2379. In this case, you can set --advertise-peer-urls to \"http://192.168.100.113:2379\". The peer can find this service through \"http://192.168.100.113:2379\".";
      };

      config = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
        description = "The configuration file for TiKV. If you set the configuration using the command line, the same setting in the config file is overwritten.";
      };

      dataDir = lib.mkOption {
        type = lib.types.str;
        default = "/tmp/tikv/store";
        description = "The path to the data directory";
      };

      initialCluster = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "The initial cluster configuration for PD. The format is [ \"node1=http://host1:port1\" \"node2=http://host2:port2\" ... ].";
      };

      logLevel = lib.mkOption {
        type = lib.types.str;
        default = "info";
        description = "The log level (trace, debug, info, warn, error, or off)";
      };

      logFile = lib.mkOption {
        type = lib.types.nullOr lib.types.str;
        default = null;
        description = "The log file path. If this parameter is not set, logs are written to stderr. Otherwise, logs are stored in the log file which will be automatically rotated every day.";
      };

      name = lib.mkOption {
        type = lib.types.str;
        default = "pd";
        description = "The name of the PD server";
      };
    };
  };

  config = let
    cfg = config.services.pd;

    optional-flag = module-lib.optional-flag cfg;
    optional-array-flag = module-lib.optional-array-flag cfg;

    command = pkgs.writeShellScript "pd-command" ''
      ${cfg.package}/bin/pd-server \
        --name=${cfg.name} \
        ${optional-array-flag "client-urls" "clientUrls"} \
        ${optional-array-flag "peer-urls" "peerUrls"} \
        ${optional-array-flag "advertise-peer-urls" "advertisePeerUrls"} \
        ${optional-array-flag "initial-cluster" "initialCluster"} \
        --data-dir=${cfg.dataDir} \
        ${optional-array-flag "initial-cluster" "initialCluster"} \
        --log-level=${cfg.logLevel} \
        ${optional-flag "log-file" "logFile"}
        ${optional-flag "config" "config"}
    '';
  in lib.mkIf cfg.enable {
    systemd.services.pd = {
      description = "PD Server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = command;
        Restart = "on-failure";
      };

      environment = {
        TZ = "UTC";
      };
    };

    environment.systemPackages = [ pkgs.tzdata ];
    environment.variables = {
      TZ = "UTC";
    };
  };
}
