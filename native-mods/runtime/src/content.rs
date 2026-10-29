use horizon_native_api::{ContentSnapshot, LoadContext, ModMetadata, ResourceId};

pub(super) fn collect(
    content: &mut ContentSnapshot,
    metadata: &ModMetadata,
    context: LoadContext,
) -> Result<(), String> {
    let owner = metadata.id;
    let horizon_native_api::ContentDeclarations {
        items,
        blocks,
        recipes,
        commands,
        block_entities,
        tabs,
        resources,
    } = context.into_declarations();
    if metadata.side != horizon_native_api::ModSide::Common
        && (!items.is_empty()
            || !blocks.is_empty()
            || !recipes.is_empty()
            || !commands.is_empty()
            || !block_entities.is_empty()
            || !tabs.is_empty()
            || !resources.is_empty())
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
        if content.blocks.contains_key(&id) || content.items.insert(id.clone(), item).is_some() {
            return Err(format!("duplicate item ID {id}"));
        }
    }
    for block in blocks {
        check(&block.id, &block.translation_key)?;
        let id = block.id.clone();
        if content.items.contains_key(&id) || content.blocks.insert(id.clone(), block).is_some() {
            return Err(format!("duplicate block/item ID {id}"));
        }
    }
    for recipe in recipes {
        check(&recipe.id, "recipe")?;
        if recipe.id.as_str().contains('/')
            || !(1..=9).contains(&recipe.ingredients.len())
            || recipe.count == 0
            || recipe.count > 64
        {
            return Err(format!(
                "{}: recipe needs a flat ID, 1..9 ingredients and 1..64 result count",
                recipe.id
            ));
        }
        let id = recipe.id.clone();
        if content.recipes.insert(id.clone(), recipe).is_some() {
            return Err(format!("duplicate recipe ID {id}"));
        }
    }
    for command in commands {
        check(&command.id, "command")?;
        if command.id.as_str().contains('/') {
            return Err(format!("{}: command ID must be flat", command.id));
        }
        let id = command.id.clone();
        if content.commands.insert(id.clone(), command).is_some() {
            return Err(format!("duplicate command ID {id}"));
        }
    }
    for entity in block_entities {
        check(&entity.id, "block entity")?;
        if entity.version == 0
            || entity.fields.len() > 64
            || entity.fields.keys().any(|key| {
                key.is_empty()
                    || key.len() > 64
                    || !key
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
            })
        {
            return Err(format!(
                "{}: invalid block entity version/fields",
                entity.id
            ));
        }
        if entity.block.namespace() != owner {
            return Err(format!("{}: block entity must own its block", entity.id));
        }
        if content
            .block_entities
            .values()
            .any(|e| e.block == entity.block)
        {
            return Err(format!("duplicate block entity for {}", entity.block));
        }
        let id = entity.id.clone();
        if content.block_entities.insert(id.clone(), entity).is_some() {
            return Err(format!("duplicate block entity ID {id}"));
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
            let problem = if !content.items.contains_key(item) && !content.blocks.contains_key(item)
            {
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
    for recipe in content.recipes.values() {
        let owner = recipe.id.namespace();
        for item in recipe
            .ingredients
            .iter()
            .chain(std::iter::once(&recipe.result))
        {
            if item.namespace() == "minecraft" {
                continue;
            }
            if !content.items.contains_key(item) && !content.blocks.contains_key(item) {
                return Err((
                    owner.into(),
                    format!("recipe {} references missing item {item}", recipe.id),
                ));
            }
            if item.namespace() != owner
                && !mods[owner]
                    .dependencies
                    .iter()
                    .any(|d| d.id == item.namespace())
            {
                return Err((
                    owner.into(),
                    format!(
                        "recipe {} references {item} without a declared dependency",
                        recipe.id
                    ),
                ));
            }
        }
    }
    for entity in content.block_entities.values() {
        if !content.blocks.contains_key(&entity.block) {
            return Err((
                entity.id.namespace().into(),
                format!("{}: missing registered block {}", entity.id, entity.block),
            ));
        }
    }
    Ok(())
}
