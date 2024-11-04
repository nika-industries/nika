
{ bin-hl, ... }: let
  perBinaryCommands = binary: [
    {
      name = binary;
      command = "cargo run --bin ${binary} -- $@";
      help = "Run the `${bin-hl binary}` binary";
      category = "[local binary actions]";
    }
    {
      name = "${binary}-release";
      command = "cargo run --release --bin ${binary} -- $@";
      help = "Run the `${bin-hl binary}` binary in release mode";
      category = "[local binary actions]";
    }
    {
      name = "${binary}-watch";
      command = "bacon -j run -- --bin ${binary} -- $@";
      help = "Watch for changes and run the `${bin-hl binary}` binary";
      category = "[local binary actions]";
    }
  ];
in [
  {
    name = "cart-watch";
    command = "cargo leptos watch --project cart $@";
    help = "Watch for changes and run the cart server";
    category = "[cart actions]";
  }
  {
    name = "cart";
    command = "cargo leptos serve --project cart --release $@";
    help = "Run the cart server";
    category = "[cart actions]";
  }
]
  ++ perBinaryCommands "fetcher"
  ++ perBinaryCommands "api"
  ++ perBinaryCommands "daemon"
