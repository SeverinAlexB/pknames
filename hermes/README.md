pknames DNS server
=================

Very simple DNS Server that resolves pknames + pkarr uris.

## Getting Started

Run it

```bash
cargo run --bin hermes -- -f 8.8.8.8
```

*Docker Desktop on MacOS uses the default DNS port 53. Checkout [this guide](https://developer.apple.com/forums/thread/738662) 
on how to free it to run hermes.

Resolve domain

```bash
nslookup example.com localhost -port=54
```


## Thanks

Thanks to Emil that programmed the original [DNS server](https://github.com/EmilHernvall/hermes/tree/master).