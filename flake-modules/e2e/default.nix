localFlake: { self, ... }: {
  perSystem = { pkgs, ... }: let 
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
    nixvm-test-tikv-cluster-connect = pkgs.testers.runNixOSTest {
      name = "tikv-cluster-connect";

      nodes = {
        tikv1 = { ... }: {
          imports = [ (assign-static 10) self.nixosModules.tikv ];

          services.tikv = {
            enable = true;
            addr = "0.0.0.0:20160";
            advertiseAddr = "10.0.0.10:20160";
            statusAddr = "0.0.0.0:20180";
            advertiseStatusAddr = "10.0.0.10:20180";
            pdServers = [ "10.0.0.11:2379" ];
          };
          networking.firewall.allowedTCPPorts = [ 20180 ];
        };

        pd1 = { ... }: {
          imports = [ (assign-static 11) self.nixosModules.pd ];

          services.pd = {
            enable = true;
            clientUrls = [ "http://10.0.0.11:2379" ];
          };
          networking.firewall.allowedTCPPorts = [ 2379 ];
        };

        client = { pkgs, ... }: {
          imports = [ (assign-static 12) ];

          environment.systemPackages = with pkgs; [ curl ];
        };
      };

      testScript = ''
        pd1.wait_for_unit("network-online.target")
        pd1.wait_for_unit("pd.service")
        tikv1.wait_for_unit("network-online.target")
        tikv1.wait_for_unit("tikv.service")

        # curl tikv1:20180/metrics from client
        client.wait_for_unit("network-online.target")
        client.succeed("ping 10.0.0.10 -c 1")
        client.succeed("curl http://10.0.0.10:20180/metrics")
      '';
    };
  in {
    checks = {
      inherit nixvm-test-tikv-cluster-connect;
    };
  };
}