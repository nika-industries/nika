{ pkgs, common, config, ... }: {
  tikv-cluster-connect = pkgs.testers.runNixOSTest {
    name = "tikv-cluster-connect";

    nodes = {
      client = { pkgs, ... }: {
        imports = [ (common.assign-static 16) ];

        environment.systemPackages = with pkgs; [ curl jq config.packages.migrator ];
      };
    } // common.tikv-cluster;

    testScript = ''
      start_all()
    
      # PD reports at least the first store as "Up"
      client.wait_until_succeeds("curl http://10.0.0.11:2379/pd/api/v1/stores | jq -e '.[\"stores\"][0][\"store\"][\"state_name\"] == \"Up\"'")

      # Perform some writes
      client.succeed("TIKV_URLS=10.0.0.10:2379,10.0.0.11:2379,10.0.0.12:2379 migrator")
    '';
  };
}
