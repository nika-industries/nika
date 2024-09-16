
{ bin-hl, ... }: [
  {
    name = "tikv";
    command = "mprocs \"run-tikv\" \"run-pd\"";
    help = "Run the ${bin-hl "tikv"} stack";
    category = "[stack actions]";
  }
  {
    name = "stack";
    command = "mprocs \"run-tikv\" \"run-pd\" \"redis-server\" \"fetcher\" \"api\"";
    help = "Run the whole stack";
    category = "[stack actions]";
  }
  {
    name = "stack-release";
    command = "mprocs \"run-tikv\" \"run-pd\" \"redis-server\" \"fetcher-release\" \"api-release\"";
    help = "Run the whole stack in release mode";
    category = "[stack actions]";
  }
]
