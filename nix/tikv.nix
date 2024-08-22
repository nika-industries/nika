{ pkgs, crane, ... }: let
  pname = "tikv";
  version = "8.3.0";

  src = builtins.fetchGit {
    url = "https://github.com/tikv/tikv.git";
    rev = "a4c0ea1657b3d939da51ea1cbbe77aff94bb60d3";
    ref = "release-8.3";
  };

  craneLib = (crane.mkLib pkgs).overrideToolchain
    (p: p.rust-bin.fromRustupToolchainFile "${src}/rust-toolchain.toml");

  titan_rocksdb_src = builtins.fetchGit {
    url = "https://github.com/tikv/rocksdb.git";
    rev = "45509f0f530ad370863876fc1ee95ccf85bfe96d";
    ref = "6.29.tikv";
  };

  tikv = craneLib.buildPackage {
    inherit src pname version;
    doCheck = false;

    nativeBuildInputs = with pkgs; [ cmake pkg-config ];
    buildInputs = with pkgs; [ openssl snappy lz4 zstd rocksdb protobuf ];

    postPatch = ''
      # copy the vendored dependencies to a temporary, mutable directory
      mkdir -p "$TMPDIR/nix-vendor"
      cp -Lr "$cargoVendorDir" -T "$TMPDIR/nix-vendor"
      sed -i "s|$cargoVendorDir|$TMPDIR/nix-vendor/|g" "$TMPDIR/nix-vendor/config.toml"
      chmod -R +w "$TMPDIR/nix-vendor"
      cargoVendorDir="$TMPDIR/nix-vendor"

      # inject the rocksdb source needed by titan
      echo "${titan_rocksdb_src}"
      cp -Lr "${titan_rocksdb_src}" "$TMPDIR/nix-vendor/25a4b890dbac8dc2ef649efe5a53969715c01f9f372954e542691b3fd36e942e/rocksdb"
    '';
  };

in {
  inherit tikv;
}
