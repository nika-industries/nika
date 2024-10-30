localFlake: { self, ... } @ moduleTop: {
  perSystem = { pkgs, ... } @ perSystemTop: let
    test = pkgs.writeText "test" ''
      Hello, world!
    '';
  in {
    kubernetes = { inherit test; };
  };
}
