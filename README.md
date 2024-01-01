# fancyd-rust

An experimental Web of Trust (WoT) library. WIP.

## Todos

- [x] Simple probability interference
- [x] Backpropagation to blame intermediate for the right/wrong results.
- [x] Graph visualization
- [x] Graph pruning
- [x] Transform cyclical graph to acyclical.
- [ ] CLI for a petname example use case.
- [ ] ICANN DNS fallback
- [ ] Reseach gameability
    - [ ] Graph cycle removal abuse
    - [ ] Distrust abuse
    - [ ] Because somebody that claims to own a domain can't at the same time follow somebody (follows get pruned), this is gameable. Add signed claims to prevent this being abused.