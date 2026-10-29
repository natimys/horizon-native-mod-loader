//! Descriptor-only content contracts. These do not insert items into a live world.
use std::collections::BTreeMap;

/// Canonical persistent name; no transient integer handles cross the API.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceId(String);

impl ResourceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        self.0.split_once(':').expect("validated ID").0
    }
}

impl std::str::FromStr for ResourceId {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let valid = value.split_once(':').is_some_and(|(namespace, path)| {
            let base = |b: u8| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-' | b'.')
            };
            !namespace.is_empty()
                && namespace.bytes().all(base)
                && !path.is_empty()
                && path.bytes().all(|b| base(b) || b == b'/')
                && path.split('/').all(|part| !matches!(part, "" | "." | ".."))
        });
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(format!(
                "invalid resource ID {value:?}: expected namespace:path"
            ))
        }
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemDescriptor {
    pub id: ResourceId,
    pub translation_key: String,
    /// Host adapter must resolve this existing item's storage/render template.
    /// Freeze validates native references, not the host's resource catalog.
    pub template: ResourceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreativeTabDescriptor {
    pub id: ResourceId,
    pub translation_key: String,
    /// Initial contract: icon and members refer to registered native items.
    pub icon: ResourceId,
    /// Explicit display order; may reference another declared dependency's items.
    pub items: Vec<ResourceId>,
}

/// Runtime publishes one snapshot after validating every declaration.
/// Callbacks can read it via `ModContext`, never mutate the runtime's registry.
/// Maps iterate by stable resource name; item and tab namespaces are separate.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentSnapshot {
    pub items: BTreeMap<ResourceId, ItemDescriptor>,
    pub tabs: BTreeMap<ResourceId, CreativeTabDescriptor>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_names_roundtrip_and_reject_ambiguous_paths() {
        for text in [
            "demo:gear",
            "demo_mod:parts/copper_gear-2",
            "minecraft:stick",
        ] {
            let id: ResourceId = text.parse().unwrap();
            assert_eq!(id.to_string(), text);
            assert_eq!(id.to_string().parse::<ResourceId>().unwrap(), id);
        }
        for text in [
            "",
            "gear",
            ":gear",
            "demo:",
            "Demo:gear",
            "demo:a:b",
            "demo:../x",
            "demo:a//b",
            "demo:/a",
            "demo:a/",
            "demo:a/./b",
            "мод:gear",
        ] {
            assert!(text.parse::<ResourceId>().is_err(), "{text:?}");
        }
    }
}
