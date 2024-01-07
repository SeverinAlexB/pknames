# fancyd-rust

An experimental Web of Trust (WoT) domain name resolver built on [pkarr](https://github.com/nuhvi/pkarr). WIP.

```
➜  cargo run -- --help

Usage: fancyd-cli [OPTIONS] [COMMAND]

Commands:
  getinfo  General information.
  lookup   Lookup the pubkey of a domain.
  ls       List your follow lists.
  add      Add a follow to your list.
  remove   Remove a follow from your list.
  help     Print this message or the help of the given subcommand(s)

Options:
  -d, --directory <directory>  FancyDns source directory. [default: ~/.fancydns]
  -v, --verbose                Show verbose output.
  -h, --help                   Print help
```



## Todos

- [x] Simple probability interference
- [x] Backpropagation to blame intermediate for the right/wrong results.
- [x] Graph visualization
- [x] Graph pruning
- [x] Transform cyclical graph to acyclical.
- [x] CLI for a petname example use case.
- [ ] ICANN DNS fallback
- [ ] Reseach gameability