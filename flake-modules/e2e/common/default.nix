{ self, pkgs, ... }: let
  # assign-static: Assign a static IP address to a node.
  assign-static = id: {
    networking = {
      useNetworkd = true;
      useDHCP = false;
    };
    systemd.network.networks."01-eth1" = {
      name = "eth1";
      networkConfig.Address = "10.0.0.${toString id}/24";
    };
  };

  tikv-basic-config = {
    storage.reserve-space = "50MB";
  };
  tikv-basic-config-toml = (pkgs.formats.toml { }).generate "tikv-config" tikv-basic-config;

  # basic-tikv-cluster: A very basic 2-node TiKV cluster.
  basic-tikv-cluster = {
    tikv = { ... }: {
      imports = [ (assign-static 10) self.nixosModules.tikv ];

      services.tikv = {
        enable = true;
        addr = "0.0.0.0:20160";
        advertiseAddr = "10.0.0.10:20160";
        statusAddr = "0.0.0.0:20180";
        advertiseStatusAddr = "10.0.0.10:20180";
        pdServers = [ "10.0.0.11:2379" ];
        config = tikv-basic-config-toml;
      };
    };

    pd = { ... }: {
      imports = [ (assign-static 11) self.nixosModules.pd ];

      services.pd = {
        enable = true;
        clientUrls = [ "http://10.0.0.11:2379" ];
      };
    };
  };

  # tikv-cluster: A 3-node TiKV cluster with 3 PD nodes.
  tikv-cluster = {
    pd1 = {
      imports = [ (assign-static 10) self.nixosModules.pd ];
      services.pd = {
        enable = true;
        name = "pd1";
        clientUrls = [ "http://10.0.0.10:2379" ];
        peerUrls = [ "http://10.0.0.10:2380" ];
        initialCluster = [ "pd1=http://10.0.0.10:2380" "pd2=http://10.0.0.11:2380" "pd3=http://10.0.0.12:2380" ];
      };
    };
    pd2 = {
      imports = [ (assign-static 11) self.nixosModules.pd ];
      services.pd = {
        enable = true;
        name = "pd2";
        clientUrls = [ "http://10.0.0.11:2379" ];
        peerUrls = [ "http://10.0.0.11:2380" ];
        initialCluster = [ "pd1=http://10.0.0.10:2380" "pd2=http://10.0.0.11:2380" "pd3=http://10.0.0.12:2380" ];
      };
    };
    pd3 = {
      imports = [ (assign-static 12) self.nixosModules.pd ];
      services.pd = {
        enable = true;
        name = "pd3";
        clientUrls = [ "http://10.0.0.12:2379" ];
        peerUrls = [ "http://10.0.0.12:2380" ];
        initialCluster = [ "pd1=http://10.0.0.10:2380" "pd2=http://10.0.0.11:2380" "pd3=http://10.0.0.12:2380" ];
      };
    };
    
    tikv1 = {
      imports = [ (assign-static 13) self.nixosModules.tikv ];
      virtualisation.memorySize = 4096;
      services.tikv = {
        enable = true;
        addr = "0.0.0.0:20160";
        advertiseAddr = "10.0.0.13:20160";
        statusAddr = "0.0.0.0:20180";
        advertiseStatusAddr = "10.0.0.13:20180";
        pdServers = [ "10.0.0.10:2379" "10.0.0.11:2379" "10.0.0.12:2379" ];
        config = tikv-basic-config-toml;
      };
    };
    tikv2 = {
      imports = [ (assign-static 14) self.nixosModules.tikv ];
      virtualisation.memorySize = 4096;
      services.tikv = {
        enable = true;
        addr = "0.0.0.0:20160";
        advertiseAddr = "10.0.0.14:20160";
        statusAddr = "0.0.0.0:20180";
        advertiseStatusAddr = "10.0.0.14:20180";
        pdServers = [ "10.0.0.10:2379" "10.0.0.11:2379" "10.0.0.12:2379" ];
        config = tikv-basic-config-toml;
      };
    };
    tikv3 = {
      imports = [ (assign-static 15) self.nixosModules.tikv ];
      virtualisation.memorySize = 4096;
      services.tikv = {
        enable = true;
        addr = "0.0.0.0:20160";
        advertiseAddr = "10.0.0.15:20160";
        statusAddr = "0.0.0.0:20180";
        advertiseStatusAddr = "10.0.0.15:20180";
        pdServers = [ "10.0.0.10:2379" "10.0.0.11:2379" "10.0.0.12:2379" ];
        config = tikv-basic-config-toml;
      };
    };
  };
in {
  inherit tikv-cluster basic-tikv-cluster assign-static;
}
