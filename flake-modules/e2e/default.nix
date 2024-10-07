localFlake: { self, ... }: {
  perSystem = { pkgs, ... } @ perSystemTop: let
    # the common module
    common = import ./common.nix { inherit pkgs; };
    # the args we'll give to each module
    module-args = perSystemTop // { inherit self common; };
    # helper to call a module with the args
    call = source: import source module-args;
    # renames attrset keys and adds prefix
    test-renamer = name: value: pkgs.lib.attrsets.nameValuePair "nixvm-test-${name}" value;
    # helper to call a test module
    callTestModule = source: pkgs.lib.attrsets.mapAttrs' test-renamer (call source);

    tikv-cluster-connect = callTestModule ./tikv-cluster-connect.nix;
  in {
    checks = { } // tikv-cluster-connect;
  };
}
