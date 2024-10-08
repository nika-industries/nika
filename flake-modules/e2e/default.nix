localFlake: { self, ... }: {
  perSystem = { pkgs, ... } @ perSystemTop: let
    # prefix for test checks
    test-prefix = "nixvm-test-";

    # the common module
    common = import ./common.nix { inherit pkgs; };
    # the args we'll give to each module
    module-args = perSystemTop // { inherit self common; };
    # helper to call a module with the args
    call = source: import source module-args;
    # renames attrset keys and adds test check prefix
    test-renamer = name: value: pkgs.lib.attrsets.nameValuePair "${test-prefix}${name}" value;
    # helper to call a test module
    callTestModule = source: pkgs.lib.attrsets.mapAttrs' test-renamer (call source);

    # modules here
    tikv-basic-connect = callTestModule ./tikv-basic-connect.nix;
  in {
    checks = { } // tikv-basic-connect;
  };
}
