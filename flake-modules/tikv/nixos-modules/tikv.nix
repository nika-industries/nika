{ withSystem }: { pkgs, lib, config, ... }: let
  module-lib = (import ./module-lib.nix) { inherit lib; };
in {
  options = {
    services.tikv = {
      enable = lib.mkEnableOption "TiKV";
      package = lib.mkOption {
        type = lib.types.package;
        default = withSystem pkgs.stdenv.hostPlatform.system ({ config, ... }:
          config.packages.tikv
        );
        description = "The TiKV package to use.";
      };

      addr = lib.mkOption {
        type = lib.types.str;
        default = "127.0.0.1:20160";
        description = "The address that the TiKV server monitors";
      };

      advertiseAddr = lib.mkOption {
        type = lib.types.str;
        default = config.services.tikv.addr;
        description = "The server advertise address for client traffic from outside. If the client cannot connect to TiKV through the --addr address because of Docker or NAT network, you must manually set the --advertise-addr address. For example, the internal IP address of Docker is \"172.17.0.1\", while the IP address of the host is \"192.168.100.113\" and the port mapping is set to \"-p 20160:20160\". In this case, you can set --advertise-addr=\"192.168.100.113:20160\". The client can find this service through \"192.168.100.113:20160\".";
      };

      statusAddr = lib.mkOption {
        type = lib.types.str;
        default = "127.0.0.1:20180";
        description = "The port through which the TiKV service status is listened";
      };

      advertiseStatusAddr = lib.mkOption {
        type = lib.types.str;
        default = config.services.tikv.statusAddr;
        description = "The address through which TiKV accesses service status from outside. If the client cannot connect to TiKV through the --status-addr address because of Docker or NAT network, you must manually set the --advertise-status-addr address. For example, the internal IP address of Docker is \"172.17.0.1\", while the IP address of the host is \"192.168.100.113\" and the port mapping is set to \"-p 20180:20180\". In this case, you can set --advertise-status-addr=\"192.168.100.113:20180\". The client can find this service through \"192.168.100.113:20180\".";
      };

      config = lib.mkOption {
        type = lib.types.nullOr lib.types.path;
        default = null;
        description = "The configuration file for TiKV. If you set the configuration using the command line, the same setting in the config file is overwritten.";
      };

      capacity = lib.mkOption {
        type = lib.types.str;
        default = "0";
        description = "The store capacity (0 means unlimited). PD uses this parameter to determine how to balance TiKV servers. (Tip: you can use 10GB instead of 1073741824)";
      };

      dataDir = lib.mkOption {
        type = lib.types.str;
        default = "/tmp/tikv/store";
        description = "The path to the data directory";
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

      pdServers = lib.mkOption {
        type = lib.types.listOf lib.types.str;
        default = [ ];
        description = "The list of PD servers. The format is \"host:port\".";
      };
    };
  };

  config = let
    cfg = config.services.tikv;

    optional-flag = module-lib.optional-flag cfg;
    optional-array-flag = module-lib.optional-array-flag cfg;

    command = pkgs.writeShellScript "tikv-command" ''
      ${cfg.package}/bin/tikv-server \
        --addr=${cfg.addr} \
        --advertise-addr=${cfg.advertiseAddr} \
        --status-addr=${cfg.statusAddr} \
        --advertise-status-addr=${cfg.advertiseStatusAddr} \
        --capacity=${cfg.capacity} \
        --data-dir=${cfg.dataDir} \
        --log-level=${cfg.logLevel} \
        ${optional-array-flag "pd" "pdServers"} \
        ${optional-flag "config" "config"} \
        ${optional-flag "log-file" "logFile"}
    '';
  in lib.mkIf cfg.enable {
    systemd.services.tikv = {
      description = "TiKV Server";
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
  };
}
