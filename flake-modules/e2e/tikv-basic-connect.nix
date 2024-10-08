{ pkgs, common, ... }: {
  tikv-basic-connect = pkgs.testers.runNixOSTest {
    name = "tikv-basic-connect";

    nodes = {
      tikv1 = common.basic-tikv-cluster.tikv;
      pd1 = common.basic-tikv-cluster.pd;

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
