# pknames

An experimental Web of Trust (WoT) domain name resolver built on [pkarr](https://github.com/nuhvi/pkarr). WIP.

```
➜ cargo run -- --help

Usage: pknamescli [OPTIONS] [COMMAND]

Commands:
  getinfo  General information.
  lookup   Lookup the pubkey of a domain.
  ls       List your follow lists.
  add      Add a follow to your list.
  remove   Remove a follow from your list.
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --directory <directory>  pknames source directory. [default: ~/.pknames]
  -v, --verbose                Show verbose output.
  -h, --help                   Print help
```


Checkout the [example](./examples/simple/) for a first glance into the system.



## Todos

- [x] Simple probability interference
- [x] Backpropagation to blame intermediate for the right/wrong results.
- [x] Graph visualization
- [x] Graph pruning
- [x] Transform cyclical graph to acyclical.
- [ ] CLI
- [ ] DNS server
- [ ] Follow list sharing aka Datastores
- [ ] Reseach gameability