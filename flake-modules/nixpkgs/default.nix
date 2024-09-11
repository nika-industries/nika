localFlake: { inputs, ... }: {
  perSystem = { system, ... }: {
    config._module.args.pkgs = import inputs.nixpkgs {
      inherit system;
      overlays = [
        (import inputs.rust-overlay)
        inputs.devshell.overlays.default
      ];
    };
  };
}
