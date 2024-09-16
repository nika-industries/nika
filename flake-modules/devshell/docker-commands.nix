
{ config, bin-hl, ... }: let
  dockerLoad = imageName: "docker load -i ${imageName}";
  ephemeralDockerCommand = { imageName, imageVersion }: {
    name = "run-${imageName}";
    command = ''
      ${dockerLoad config.images."${imageName}"} \
      && docker run --rm --network host ${imageName}-server:${imageVersion}
    '';
    help = "Run the ${bin-hl imageName} server in an ephemeral container";
    category = "[docker actions]";
  };
in [
  (ephemeralDockerCommand { imageName = "tikv"; imageVersion = "8.1.1"; })
  (ephemeralDockerCommand { imageName = "pd"; imageVersion = "8.1.1"; })
]
