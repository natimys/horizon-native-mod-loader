use horizon_native_api::{ContentSnapshot, LoadContext, ModMetadata, ResourceId};

pub(super) fn collect(
    content: &mut ContentSnapshot,
    metadata: &ModMetadata,
    context: LoadContext,
) -> Result<(), String> {
    let owner = metadata.id;
    let horizon_native_api::ContentDeclarations {
        items,
        tabs,
        resources,
    } = context.into_descriptors();
    if metadata.side != horizon_native_api::ModSide::Common
        && (!items.is_empty() || !tabs.is_empty() || !resources.is_empty())
    {
        return Err(format!("{owner}: persistent content requires a common mod"));
    }
    for (path, bytes) in resources {
        let parts: Vec<_> = path.split('/').collect();
        if owner == "minecraft"
            || parts.len() < 3
            || !matches!(parts[0], "assets" | "data")
            || parts[1] != owner
            || parts[2..].iter().any(|p| {
                p.is_empty()
                    || *p == "."
                    || *p == ".."
                    || !p.bytes().all(|b| {
                        b.is_ascii_lowercase() || b.is_ascii_digit() || b"_.-".contains(&b)
                    })
            })
            || path.starts_with(&format!("data/{owner}/horizon/"))
        {
            return Err(format!("invalid or foreign resource path {path}"));
        }
        if content.resources.insert(path.clone(), bytes).is_some() {
            return Err(format!("duplicate resource path {path}"));
        }
    }
    let check = |id: &ResourceId, key: &str| {
        if id.namespace() == "minecraft" {
            Err(format!("reserved content namespace in {id}"))
        } else if id.namespace() != owner {
            Err(format!("{owner} cannot register foreign ID {id}"))
        } else if key.trim().is_empty() {
            Err(format!("{id} needs a translation key"))
        } else {
            Ok(())
        }
    };
    for item in items {
        check(&item.id, &item.translation_key)?;
        let id = item.id.clone();
        if content.items.insert(id.clone(), item).is_some() {
            return Err(format!("duplicate item ID {id}"));
        }
    }
    for tab in tabs {
        check(&tab.id, &tab.translation_key)?;
        let id = tab.id.clone();
        if content.tabs.insert(id.clone(), tab).is_some() {
            return Err(format!("duplicate tab ID {id}"));
        }
    }
    Ok(())
}

pub(super) fn validate<'a>(
    content: &ContentSnapshot,
    metadata: impl Iterator<Item = &'a ModMetadata>,
) -> Result<(), (String, String)> {
    let mods: std::collections::BTreeMap<_, _> = metadata.map(|m| (m.id, m)).collect();
    for tab in content.tabs.values() {
        let owner = tab.id.namespace();
        for item in std::iter::once(&tab.icon).chain(&tab.items) {
            let problem = if !content.items.contains_key(item) {
                Some(format!("tab {} references missing item {item}", tab.id))
            } else if item.namespace() != owner
                && !mods[owner]
                    .dependencies
                    .iter()
                    .any(|d| d.id == item.namespace())
            {
                Some(format!(
                    "tab {} references {item} without a declared dependency",
                    tab.id
                ))
            } else {
                None
            };
            if let Some(reason) = problem {
                return Err((owner.to_owned(), reason));
            }
        }
    }
    Ok(())
}
