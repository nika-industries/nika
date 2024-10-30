{ self, pkgs, ... } @ top: let
  # assign-static: Assign a static IP address to a node.
  # call with assign-static <id>
  assign-static = import ./assign-static.nix top;

  tikv-basic-config-toml = (pkgs.formats.toml { }).generate "tikv-config" {
    storage.reserve-space = "50MB";
  };

  # tikv-basic: A very basic 1-node TiKV cluster.
  tikv-basic = import ./tikv-bare-metal-basic.nix (top // {
    inherit assign-static tikv-basic-config-toml;
  });

  # tikv-cluster: A 3-node TiKV cluster with 3 PD nodes.
  tikv-cluster = import ./tikv-bare-metal-cluster.nix (top // {
    inherit assign-static tikv-basic-config-toml;
  });
in {
  inherit tikv-cluster tikv-basic assign-static;
}
