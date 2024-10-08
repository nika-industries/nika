{ self, ... }: rec {
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

  # basic-tikv-cluster: A very basic 2-node TiKV cluster.
  basic-tikv-cluster = {
    tikv = { ... }: {
      imports = [ (assign-static 10) self.nixosModules.tikv ];

      services.tikv = {
        enable = true;
        addr = "0.0.0.0:20160";
        advertiseAddr = "tikv1:20160";
        statusAddr = "0.0.0.0:20180";
        advertiseStatusAddr = "tikv1:20180";
        pdServers = [ "10.0.0.11:2379" ];
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
}
