# Projects

These projects are individual efforts that organize the way we push the whole
project forward. Some projects depend on others, and some are independent.

## Hexaplathesis
We're adopting a "hexagonal architecture" with regards to how we handle
validation, business logic, and external dependencies. The idea is to make a
"garden of eden" (in the shape of a hexagon :), within which all implementation
details are hidden and business logic happens.

- [X] DB interface is migrated to a model-based interface, on top of the `kv` crate
- [X] Repos are built for each major model, on top of the db interface
- [X] Service traits and canonical implementations are built for each major model, on top of the repos
- [X] The `tasks` crate is refactored to use the services, and the `api` crate compiles
- [ ] The `storage` crate is abstracted in dynamic and temp storage services
- [ ] Services are consolidated and entrypoints are limited
- [ ] Crate dependencies are reduced and locked behind features

## Nixos End-to-End Testing
We need to test the whole system end-to-end, (as close as possible to every
user interaction), and we can do so with lightweight VMs using NixOS.

We have some basic tests for our nixification of TiKV, but we need Hexaplathesis
to be in place before we can test the whole system. The primary blocker is temp
storage through R2.

```
Roadmap not yet solidified
```

## Compliance with the Nix Binary Cache Specification
We need to be able to serve Nix archives and closures to Nix users, and we need
public caches to work with the native Nix tooling.

