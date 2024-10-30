{ pkgs, common, self, config, ... }: {
  domain-api-upload-pathway = pkgs.testers.runNixOSTest {
    name = "domain-api-upload-pathway";

    nodes = {
      api = {
        imports = [ (common.assign-static 16) self.nixosModules.api ];
        services.api = {
          enable = true;
          mockTempStorage = true;
          tikvUrls = [ "10.0.0.10:2379" "10.0.0.11:2379" "10.0.0.12:2379" ];
        };
        environment.systemPackages = [ config.packages.migrator ];
      };

      client = { pkgs, ... }: {
        imports = [ (common.assign-static 17) ];
        environment.systemPackages = with pkgs; [ curl jq ];
      };
    } // common.tikv-cluster;

    testScript = ''
      start_all()

      # PD reports at least the first store as "Up"
      client.wait_for_unit("network.target")
      client.wait_until_succeeds("curl http://10.0.0.11:2379/pd/api/v1/stores | jq -e '.[\"stores\"][0][\"store\"][\"state_name\"] == \"Up\"'")
    
      api.wait_for_unit("api.service")
      api.succeed("TIKV_URLS='10.0.0.10:2379,10.0.0.11:2379,10.0.0.12:2379' migrator")

      client.execute("curl -X POST -F 'file=@${../../../flake.nix}' http://10.0.0.16:3000/naive-upload/albert/a")
    '';
  };
}
