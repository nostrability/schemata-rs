# schemata-rs

Rust data crate for [Nostr](https://nostr.com/) protocol JSON schemas. This is the Rust equivalent of the [`@nostrability/schemata`](https://github.com/nostrability/schemata) npm package.

Schemas are vendored from the [schemata releases](https://github.com/nostrability/schemata/releases) and embedded into the binary at compile time. A naming registry maps export names (e.g. `kind1Schema`, `noteSchema`) to parsed JSON values, using the same logic as the JS [build.js](https://github.com/nostrability/schemata/blob/main/build.js).

## Related projects

| Project | Language | Role |
|---------|----------|------|
| [nostrability/schemata](https://github.com/nostrability/schemata) | JSON/JS | Canonical schema definitions |
| [schemata-validator-rs](https://github.com/nostrability/schemata-validator-rs) | Rust | Validator using this crate |
| [@nostrwatch/schemata-js-ajv](https://github.com/sandwichfarm/nostr-watch) | JS/TS | JS validator (AJV-based) |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
schemata-rs = { git = "https://github.com/nostrability/schemata-rs.git" }
```

```rust
// Look up a schema by name
let schema = schemata_rs::get("kind1Schema").unwrap();
println!("{}", schema);

// Iterate all available schema names
for key in schemata_rs::keys() {
    println!("{}", key);
}
```

## API

| Function | Description |
|----------|-------------|
| `get(key: &str) -> Option<&'static Value>` | Look up a schema by export name |
| `keys() -> impl Iterator<Item = &'static str>` | Iterate all available schema names |
| `SCHEMA_DIR` | Raw embedded directory (for advanced use) |

## Updating schemas

Requires [just](https://github.com/casey/just):

```sh
just update-schemas
```

This downloads the latest [schemata release](https://github.com/nostrability/schemata/releases), extracts schemas into `schemas/`, and writes `schemas/SCHEMATA_VERSION`.

## License

GPL-3.0-or-later
