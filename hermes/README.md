pknames DNS server
=================

Very simple DNS Server that resolves pknames + pkarr pubkeys.

## Getting Started

Run it

```bash
cargo run --bin hermes -- -f 8.8.8.8
```

> Address already in use macOS: Docker Desktop on MacOS uses the  port 53. Checkout [this guide](https://developer.apple.com/forums/thread/738662) 
on how to free it.

**Resolve domain**

```bash
nslookup example.com localhost
```

**Web Browser**

Any domain that does not end in a well-known TLD (.com, .de, .ch) is turned into a search by modern browsers.
The trick is to add a slash `/` at the end of the domain to force the browser to load the website.

```
pknames.p2p/                -> Resolves to website
http://pknames.p2p          -> Resolves to website
http://pknames.p2p/about    -> Resolves to website
pknames.p2p                 -> Google Search
```


## Thanks

Thanks to Emil that programmed the original [DNS server](https://github.com/EmilHernvall/hermes/tree/master).