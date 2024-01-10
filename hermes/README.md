pknames DNS server
=================

Very simple DNS Server that resolves pknames + pkarr uris.

## Getting Started

Run it

```bash
cargo run --bin hermes -- -p 54 -f 8.8.8.8
```

Resolve domain

```bash
nslookup example.com localhost -port=54
```


## Thanks

Thanks to Emil that programmed the original [DNS server](https://github.com/EmilHernvall/hermes/tree/master).