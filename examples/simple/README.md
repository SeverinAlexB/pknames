# Simple example

Web of trust with multiple attested domains.

![Graph preview](./imgs/main_graph.png)

- Our identity: `me`
- We follow: `Alice`, `Dave`, `pk:s9y93`
- Attested domains:
    - example.com: `pk:obngk`, `pk:1zpo3`, `pk:s9y93`, `Dave`
    - microsoft.com: `pk:s9y93`
    - apple.com: `pk:s9y93`


You can use this example in the cli by using this as your main directory.

```
cargo run -- -d ./examples/simple getinfo
```

## Commands 

**Show all lists**

```bash
cargo run -- -d ./examples/simple ls
```

**Show graph in UI**

```bash
cargo run -- -d ./examples/simple ls
```

**Predict domain**

```bash
cargo run -- -d ./examples/simple lookup example.com
```

**Predict domain in UI**

Darker colors equals higher power/probability.

```bash
cargo run -- -d ./examples/simple lookup example.com --ui
```

![Prediction UI](./imgs/prediction.png)

**Add trust pubkey**

Adds pubkey to your list. Trust value must be between -1 and 1 (float).

- `1` means full trust
- `0` means neutral
- `-1` means distrust

Trusting `pk:123456` fully.

```bash

cargo run -- -d ./examples/simple add pk:123456 1  
```

**Update trust**

```bash
cargo run -- -d ./examples/simple add pk:123456 0.1  
```

**Remove trust**

```bash
cargo run -- -d ./examples/simple remove pk:123456 
```

**Attest domain to pubkey**

Attests that `pk:123456` owns `example.com` with a trust value of `1`.


```bash
cargo run -- -d ./examples/simple add pk:123456 1 example.com
```

**Update attestation**

Update attestation that `pk:123456` owns `example.com` with a trust value of `-0.5`.


```bash
cargo run -- -d ./examples/simple add pk:123456 -0.5 example.com
```

**Remove attestation**

```bash
cargo run -- -d ./examples/simple remove pk:123456 example.com
```