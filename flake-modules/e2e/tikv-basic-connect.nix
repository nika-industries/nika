{ pkgs, self, common, ... }: {
  tikv-basic-connect = pkgs.testers.runNixOSTest {
    name = "tikv-basic-connect";

    nodes = {
      tikv1 = { ... }: {
        imports = [ (common.assign-static 10) self.nixosModules.tikv ];

        services.tikv = {
          enable = true;
          addr = "0.0.0.0:20160";
          advertiseAddr = "tikv1:20160";
          statusAddr = "0.0.0.0:20180";
          advertiseStatusAddr = "tikv1:20180";
          pdServers = [ "10.0.0.11:2379" ];
        };
      };

      pd1 = { ... }: {
        imports = [ (common.assign-static 11) self.nixosModules.pd ];

        services.pd = {
          enable = true;
          clientUrls = [ "http://10.0.0.11:2379" ];
        };
      };

      client = { pkgs, ... }: {
        imports = [ (common.assign-static 12) ];

        environment.systemPackages = with pkgs; [ curl jq ];
      };
    };

    testScript = ''
      pd1.wait_for_unit("network-online.target")
      pd1.wait_for_unit("pd.service")
      tikv1.wait_for_unit("network-online.target")
      tikv1.wait_for_unit("tikv.service")

      client.wait_for_unit("network-online.target")
      # make sure we can reach the tikv node by ip
      client.succeed("ping 10.0.0.10 -c 1")
      # make sure pd reports the tikv node as up
      client.wait_until_succeeds("curl http://10.0.0.11:2379/pd/api/v1/stores | jq -e '.[\"stores\"][0][\"store\"][\"state_name\"] == \"Up\"'")
    '';
  };
}
