{ pkgs, config, common, ... }: {
  first-party-api-validate-tikv-urls = pkgs.testers.runNixOSTest {
    name = "first-party-api-validate-tikv-urls";

    nodes = {
      api = {
        imports = [ (common.assign-static 5) ];
        environment.systemPackages = [ config.packages.api ];
      };
    };

    testScript = ''
      # API bin fails without the TIKV_URLS env var
      api.fail("api")
      # API bin fails with an empty TIKV_URLS env var
      api.fail("TIKV_URLS=\'\' api")
      # API bin fails with an unreachable URL in the TIKV_URLS env var
      api.fail("TIKV_URLS=localhost:1234 api")
    '';
  };
}
