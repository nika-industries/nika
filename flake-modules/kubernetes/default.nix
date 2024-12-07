{ self, ... } @ moduleTop: {
  perSystem = { pkgs, ... } @ perSystemTop: let
    yaml = (pkgs.formats.yaml { }).generate;
  
    test = yaml "test" {
      this = {
        is = "a test";
      };
    };
  
  in {
    kubernetes = { inherit test; };
  };
}
