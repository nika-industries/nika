{ pkgs, config, common, self, ... }: let
  api_configure = tikv-urls: {
    imports = [ (common.assign-static 5) self.nixosModules.api ];
    services.api = {
      enable = true;
      mockTempStorage = true;
      tikvUrls = tikv-urls;
    };
    environment.systemPackages = with pkgs; [ curl jq ];
  };
in {

  domain-api-reject-empty-tikv-urls = pkgs.testers.runNixOSTest {
    name = "domain-api-reject-empty-tikv-urls";

    nodes = { api = api_configure [ ]; };

    testScript = ''
      api.wait_for_unit("api.service")
      # make sure "overall_status" has a "Down" key in it
      api.wait_until_succeeds("curl http://localhost:3000/health | jq -e '.[\"overall_status\"] | has(\"Down\")'")
    '';
  };

  domain-api-reject-blank-tikv-urls = pkgs.testers.runNixOSTest {
    name = "domain-api-reject-blank-tikv-urls";

    nodes = { api = api_configure [ "" ]; };

    testScript = ''
      api.wait_for_unit("api.service")
      # make sure "overall_status" has a "Down" key in it
      api.wait_until_succeeds("curl http://localhost:3000/health | jq -e '.[\"overall_status\"] | has(\"Down\")'")
    '';
  };

  domain-api-reject-invalid-tikv-urls = pkgs.testers.runNixOSTest {
    name = "domain-api-reject-invalid-tikv-urls";

    nodes = { api = api_configure [ "localhost:2379" ]; };

    testScript = ''
      api.wait_for_unit("api.service")
      # make sure "overall_status" has a "Down" key in it
      api.wait_until_succeeds("curl http://localhost:3000/health | jq -e '.[\"overall_status\"] | has(\"Down\")'")
    '';
  };

  domain-api-accept-valid-tikv-urls = pkgs.testers.runNixOSTest {
    name = "domain-api-accept-valid-tikv-urls";

    nodes = {
      api = api_configure [ "10.0.0.11:2379" ];
    } // common.tikv-basic;

    testScript = ''
      pd1.wait_for_unit("pd.service")
      tikv1.wait_for_unit("tikv.service")
      api.wait_for_unit("api.service")
      # make sure "overall_status" has a "Up" key in it
      api.wait_until_succeeds("curl http://localhost:3000/health | jq -e '.[\"overall_status\"] == \"Ok\"'")
    '';
  };

}
