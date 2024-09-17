{ lib, ... }: {
  optional-flag = cfg: flag-name: cfg-name: (lib.optionalString (cfg.${cfg-name} != null) "--${flag-name}=${cfg.${cfg-name}}");
  optional-array-flag = cfg: flag-name: cfg-name: (lib.optionalString ((builtins.length cfg.${cfg-name}) > 0) "--${flag-name}=${lib.concatStringsSep "," cfg.${cfg-name}}");
}
