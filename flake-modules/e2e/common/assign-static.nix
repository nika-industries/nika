# assign-static: Assign a static IP address to a node.
{ ... } @ top:
  id: {
    networking = {
      useNetworkd = true;
      useDHCP = false;
    };
    systemd.network.networks."01-eth1" = {
      name = "eth1";
      networkConfig.Address = "10.0.0.${toString id}/24";
    };
  }
