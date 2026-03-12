mod registry;

use include_dir::{include_dir, Dir};
use serde_json::Value;

/// Embedded schema directory, compiled into the binary.
pub static SCHEMA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/schemas");

/// Look up a schema by its export name (e.g. "kind1Schema", "noteSchema").
pub fn get(key: &str) -> Option<&'static Value> {
    registry::REGISTRY.get(key)
}

/// Iterate over all available schema export names.
pub fn keys() -> impl Iterator<Item = &'static str> {
    registry::REGISTRY.keys().map(|s| s.as_str())
}
