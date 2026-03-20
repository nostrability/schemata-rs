use std::collections::BTreeSet;

#[test]
fn golden_keys_match() {
    let golden: BTreeSet<String> = include_str!("golden_keys.txt")
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    let actual: BTreeSet<String> = schemata_rs::keys().map(|s| s.to_string()).collect();

    let missing: Vec<_> = golden.difference(&actual).collect();
    let extra: Vec<_> = actual.difference(&golden).collect();

    if !missing.is_empty() || !extra.is_empty() {
        panic!(
            "Key mismatch!\n  Missing from Rust ({}):\n    {}\n  Extra in Rust ({}):\n    {}",
            missing.len(),
            missing.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n    "),
            extra.len(),
            extra.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n    "),
        );
    }

    assert_eq!(golden.len(), actual.len(), "Key count mismatch");
}

#[test]
fn values_are_valid_json() {
    // Spot-check that values are non-null JSON objects
    let spot_checks = ["kind1Schema", "noteSchema", "nip11Schema", "tagSchema"];
    for key in spot_checks {
        let val = schemata_rs::get(key);
        assert!(val.is_some(), "Expected key '{}' to exist", key);
        let v = val.unwrap();
        assert!(v.is_object(), "Expected '{}' to be a JSON object, got {:?}", key, v);
    }
}

#[test]
fn get_nonexistent_returns_none() {
    assert!(schemata_rs::get("nonExistentSchema").is_none());
}
