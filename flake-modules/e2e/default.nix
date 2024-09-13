localFlake: { ... }: {
  perSystem = { pkgs, ... }: let 
    nixvm-test-minimal = pkgs.testers.runNixOSTest {
      name = "minimal-test";

      nodes.machine = { pkgs, ... }: {
        environment.systemPackages = with pkgs; [
          cowsay
        ];

        system.stateVersion = "23.11";
      };

      testScript = ''
        machine.wait_for_unit("default.target")
        machine.succeed("su -- root -c 'which cowsay'")
      '';
    };
  in {
    checks = {
      inherit nixvm-test-minimal;
    };
  };
}
