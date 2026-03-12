use include_dir::{Dir, DirEntry};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;

use crate::SCHEMA_DIR;

/// Remove all characters that aren't [a-zA-Z0-9].
fn sanitize(s: &str) -> String {
    s.chars().filter(|c| c.is_ascii_alphanumeric()).collect()
}

/// "client-req" → "clientReq", "kind-3" → "kind3"
fn camel_case_hyphens(s: &str) -> String {
    let parts: Vec<&str> = s.split('-').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return String::new();
    }
    let first = parts[0].to_lowercase();
    let rest: String = parts[1..]
        .iter()
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    let lower: String = chars.collect::<String>().to_lowercase();
                    format!("{}{}", upper, lower)
                }
                None => String::new(),
            }
        })
        .collect();
    format!("{}{}", first, rest)
}

/// If base is "schema" → "", if starts with "schema." → capitalize remainder.
fn process_base_name(base_name: &str) -> String {
    if base_name == "schema" {
        return String::new();
    }
    if let Some(after) = base_name.strip_prefix("schema.") {
        let mut chars = after.chars();
        match chars.next() {
            Some(c) => {
                let upper: String = c.to_uppercase().collect();
                format!("{}{}", upper, chars.collect::<String>())
            }
            None => String::new(),
        }
    } else {
        base_name.to_string()
    }
}

/// Handle tag directory special cases.
fn handle_tag_case(dir_parts: &[&str], base_name: &str) -> Option<String> {
    let last_dir = dir_parts.last().copied().unwrap_or("");
    let second_last = if dir_parts.len() >= 2 {
        dir_parts[dir_parts.len() - 2]
    } else {
        ""
    };

    // tag/schema.json → "tagSchema"
    if last_dir == "tag" && base_name == "schema" {
        return Some("tagSchema".to_string());
    }

    // tag/e/schema.json → "eTagSchema", tag/_A/schema.json → "ATagSchema"
    if second_last == "tag" {
        if base_name == "schema" {
            let mut name = last_dir.to_string();
            if name.starts_with('_') && name.len() > 1 {
                let rest = &name[1..];
                let mut chars = rest.chars();
                if let Some(c) = chars.next() {
                    let upper: String = c.to_uppercase().collect();
                    name = format!("{}{}", upper, chars.collect::<String>());
                }
            }
            return Some(format!("{}TagSchema", name));
        } else {
            return Some(String::new()); // skip non-schema files in tag subdirs
        }
    }

    // tag/amount.json → "amountTagSchema", tag/_A.json → "ATagSchema"
    if last_dir == "tag" && !base_name.is_empty() {
        if base_name.starts_with('_') && base_name.len() > 1 {
            let rest = &base_name[1..];
            let mut chars = rest.chars();
            if let Some(c) = chars.next() {
                let upper: String = c.to_uppercase().collect();
                let letter = format!("{}{}", upper, chars.collect::<String>());
                return Some(format!("{}TagSchema", letter));
            }
        }
        return Some(format!("{}TagSchema", base_name));
    }

    None
}

/// Generate export name for nips/ and mips/ files.
fn generate_nips_export(path: &str) -> Option<String> {
    let base_name = path
        .rsplit('/')
        .next()?
        .strip_suffix(".json")?;
    let processed = process_base_name(base_name);

    let dir = path.rsplit_once('/')?.0;
    let dir_parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();

    if let Some(tag_result) = handle_tag_case(&dir_parts, base_name) {
        if tag_result.is_empty() {
            return None; // skip
        }
        return Some(sanitize(&tag_result));
    }

    let parent = dir_parts.last().copied().unwrap_or("");
    let parent_cased = camel_case_hyphens(parent);
    let mut combined = format!("{}{}", parent_cased, processed);
    if combined.is_empty() {
        combined = "Unnamed".to_string();
    }
    combined.push_str("Schema");
    Some(sanitize(&combined))
}

/// Generate export name for _aliases/ files.
/// Direct children of _aliases get lowercase naming.
fn generate_alias_export(path: &str) -> Option<String> {
    let base_name = path
        .rsplit('/')
        .next()?
        .strip_suffix(".json")?;
    let processed = process_base_name(base_name);

    let dir = path.rsplit_once('/')?.0;
    let dir_parts: Vec<&str> = dir.split('/').filter(|p| !p.is_empty()).collect();

    let last = dir_parts.last().copied().unwrap_or("");
    if last == "_aliases" {
        let final_name = format!("{}Schema", processed.to_lowercase());
        return Some(sanitize(&final_name));
    }

    if let Some(tag_result) = handle_tag_case(&dir_parts, base_name) {
        if tag_result.is_empty() {
            return None;
        }
        return Some(sanitize(&tag_result));
    }

    let parent = dir_parts.last().copied().unwrap_or("");
    let parent_cased = camel_case_hyphens(parent);
    let mut combined = format!("{}{}", parent_cased, processed);
    if combined.is_empty() {
        combined = "Unnamed".to_string();
    }
    combined.push_str("Schema");
    Some(sanitize(&combined))
}

/// Recursively collect all .json file paths from an embedded Dir.
fn collect_json_paths(dir: &'static Dir<'static>, prefix: &str) -> Vec<(String, &'static [u8])> {
    let mut results = Vec::new();
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(d) => {
                let sub_prefix = format!("{}/{}", prefix, d.path().file_name().unwrap_or_default().to_string_lossy());
                results.extend(collect_json_paths(d, &sub_prefix));
            }
            DirEntry::File(f) => {
                let name = f.path().file_name().unwrap_or_default().to_string_lossy();
                if name.ends_with(".json") {
                    let path = format!("{}/{}", prefix, name);
                    results.push((path, f.contents()));
                }
            }
        }
    }
    results
}

/// Build the registry: HashMap from export name → parsed JSON schema.
fn build_registry() -> HashMap<String, Value> {
    let mut map = HashMap::new();

    type NameGen = fn(&str) -> Option<String>;
    // Process nips/, mips/, _aliases/ in order (first-wins dedup)
    let dirs_and_generators: Vec<(&str, NameGen)> = vec![
        ("nips", generate_nips_export as NameGen),
        ("mips", generate_nips_export as NameGen),
        ("_aliases", generate_alias_export as NameGen),
    ];

    for (dir_name, generator) in dirs_and_generators {
        if let Some(dir) = SCHEMA_DIR.get_dir(dir_name) {
            let mut entries = collect_json_paths(dir, dir_name);
            // Sort lexicographically for deterministic ordering (matches JS readdirSync behavior)
            entries.sort_by(|a, b| a.0.cmp(&b.0));

            for (path, contents) in entries {
                if let Some(export_name) = generator(&path) {
                    if !export_name.is_empty() {
                        // First-wins: skip if already exists
                        map.entry(export_name).or_insert_with(|| {
                            serde_json::from_slice(contents).unwrap_or(Value::Null)
                        });
                    }
                }
            }
        }
    }

    map
}

pub(crate) static REGISTRY: Lazy<HashMap<String, Value>> = Lazy::new(build_registry);
