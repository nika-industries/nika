# basic-tikv-cluster: A very basic 1-node TiKV cluster.
{ self, assign-static, tikv-basic-config-toml, ... } @ top: {
  tikv1 = { ... }: {
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

  pd1 = { ... }: {
    imports = [ (assign-static 11) self.nixosModules.pd ];

    services.pd = {
      enable = true;
      clientUrls = [ "http://10.0.0.11:2379" ];
    };
  };
}
