{ pkgs, config, common, ... }: {
  first-party-api-validate-tikv-urls = pkgs.testers.runNixOSTest {
    name = "first-party-api-validate-tikv-urls";

    nodes = {
      tikv1 = common.basic-tikv-cluster.tikv;
      pd1 = common.basic-tikv-cluster.pd;
      api = {
        imports = [ (common.assign-static 5) ];
        environment.systemPackages = [ config.packages.api ];
      };
    };

    testScript = ''
      pd1.wait_for_unit("network-online.target")
      pd1.wait_for_unit("pd.service")
      tikv1.wait_for_unit("network-online.target")
      tikv1.wait_for_unit("tikv.service")
    
      api.wait_for_unit("network-online.target")

      command = "api --mock-temp-storage health"
      # API bin fails without the TIKV_URLS env var
      api.fail(command)
      # API bin fails with an empty TIKV_URLS env var
      api.fail("TIKV_URLS=\'\' " + command)
      # API bin fails with an unreachable URL in the TIKV_URLS env var
      api.fail("TIKV_URLS=localhost:1234 " + command)

      # API bin succeeds with a reachable URL in the TIKV_URLS env var
      api.succeed("TIKV_URLS=10.0.0.11:2379 " + command)
    '';
  };
}
