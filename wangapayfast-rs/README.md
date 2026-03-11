# wangapayfast-rs

Helpers for working with [PayFast](https://www.payfast.co.za/) ITN (Instant Transaction Notification)
messages in Rust services.

This crate focuses on:

- Parsing the `application/x-www-form-urlencoded` ITN body
- Regenerating the PayFast signature according to their documentation
- Verifying that an incoming ITN is authentic before you update your own records

It intentionally does **not** make outbound HTTP requests – you can use it with
any HTTP framework.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
wangapayfast-rs = "0.1"
```

Example (pseudo‑handler):

```rust
use std::collections::BTreeMap;

use wangapayfast_rs::{ItnRequest, PayFastConfig};

fn handle_itn(raw_body: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    // 1) Parse the raw ITN body (application/x-www-form-urlencoded).
    let itn = ItnRequest::from_body(raw_body)?;

    // 2) Build your config (usually once at startup).
    let config = PayFastConfig::new(Some("your-payfast-passphrase"));

    // 3) Verify the signature.
    if !itn.is_valid(&config) {
        // Invalid ITN – reject.
        return Err("invalid ITN signature".into());
    }

    // 4) Use the parsed fields.
    let params: &BTreeMap<String, String> = itn.params();
    let payment_status = params.get("payment_status").cloned().unwrap_or_default();

    // ... act on `payment_status`, `amount_gross`, etc.

    Ok(())
}
```

## How signature verification works

For an incoming ITN:

1. Start from all ITN fields (key–value pairs) in the body.
2. Remove the `signature` field.
3. Remove any fields whose value is blank after trimming.
4. Sort the remaining fields by key (lexicographically).
5. URL‑encode each value like an HTML form (`application/x-www-form-urlencoded`,
   spaces become `+`).
6. Join into a query string: `key=value&key2=value2...`.
7. If you use a PayFast passphrase, append `&passphrase=your-passphrase`.
8. Compute the MD5 hex digest – that is the expected signature.

`wangapayfast-rs` implements this algorithm so you only need to hand it the raw
body and your passphrase.

## License

MIT OR Apache-2.0

---

![1nga Solutions](https://www.1nga.com/logo.svg)

**Developed with love by 1nga Solutions**

