use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::tile::ContentItem;

use super::*;

type ItemInfos = FxHashMap<ItemId, ItemInfo>;
type ItemIds = FxHashMap<String, ItemId>;

#[derive(Default, Deserialize, Serialize)]
pub struct Items {
    #[serde(skip_deserializing, skip_serializing)]
    infos: ItemInfos,
    // we do need to save this...
    ids: ItemIds,
}

// Deserialize-only struct
#[derive(Debug, Clone, Deserialize)]
struct ItemsSave {
    items: FxHashMap<String, ItemInfoSave>,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemInfoSave {
    id: ItemId,
    #[serde(default)]
    sprite: String,
    #[serde(default)]
    recipe: Option<RecipeSave>,
    #[serde(default)]
    flags: ItemInfoFlags,
    #[serde(default)]
    food_value: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeSave {
    workshop: String,
    requires: Vec<String>,
    #[serde(default)]
    quantity: u8,
}

impl ItemInfoSave {
    fn convert(&self, name: &String, ids: &ItemIds) -> ItemInfo {
        ItemInfo {
            name: name.clone(),
            sprite: if self.sprite.is_empty() {
                name.clone()
            } else {
                self.sprite.clone()
            },
            flags: self.flags,
            recipe: self.recipe.as_ref().map(|recipe_save| Recipe {
                workshop: recipe_save.workshop.clone(),
                requires: recipe_save
                    .requires
                    .iter()
                    .map(|item_name| {
                        *ids.get(item_name).unwrap_or_else(|| {
                            log::error!(
                                "Could not find ingredient '{}' for item '{}'",
                                item_name,
                                name
                            );
                            &ITEM_NONE
                        })
                    })
                    .collect::<Vec<ItemId>>(),
                quantity: recipe_save.quantity.max(1),
            }),
            food_value: self.food_value,
        }
    }
}

impl Items {
    pub fn get_info(&self, id: &ItemId) -> Option<&ItemInfo> {
        self.infos.get(&id)
    }

    pub fn get_id(&self, name: &str) -> Option<ItemId> {
        self.ids.get(name).copied()
    }

    pub fn get_content(&self, id: &ItemId) -> Option<ContentItem> {
        Some((*id, self.infos.get(id)?.flags))
    }

    pub fn get_content_name(&self, name: &str) -> Option<ContentItem> {
        self.get_content(&self.get_id(name)?)
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, u32, ItemInfo> {
        self.infos.iter()
    }

    pub async fn load(&mut self) {
        // load new stuff
        let ron_str = macroquad::file::load_file("assets/data/items.ron")
            .await
            .expect("Failed to load items.ron");
        self.load_from_bytes(&ron_str);
    }

    fn load_from_bytes(&mut self, bytes: &[u8]) {
        let items_save: ItemsSave =
            ron::de::from_bytes(&bytes).expect("Failed to deserialize items!");

        for (item_name, item_info_save) in items_save.items.iter() {
            let id = item_info_save.id;

            if let Some(old_id) = self.ids.insert(item_name.clone(), id) {
                if old_id != id {
                    log::error!(
                        "Item id changed for '{}'! Was: {} Now: {}",
                        item_name,
                        old_id,
                        id
                    );
                }
            }
        }

        for (item_name, item_info_save) in items_save.items.iter() {
            let id = item_info_save.id;
            let info = item_info_save.convert(item_name, &self.ids);
            if !self.infos.contains_key(&id) {
                self.infos.insert(id, info);
            } else {
                if self.infos[&id] != info {
                    log::warn!("Item info changed for item {}", item_name);
                }
            }
        }

        // verify we aren't missing anything...
        for (name, id) in self.ids.iter() {
            if !self.infos.contains_key(id) {
                log::error!("Item definition not found for '{}' (id: {})", name, id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_items() {
        let mut items = Items::default();
        let bytes = std::fs::read("assets/data/items.ron").expect("Failed to load items.ron");

        items.load_from_bytes(&bytes);

        assert!(items.ids.get("stone").is_some_and(|id| *id > 0));
        let stone_id = items.ids["stone"];

        assert!(
            items
                .infos
                .get(&stone_id)
                .is_some_and(|info| info.recipe.is_none() && info.sprite == "stone_item")
        );
    }

    #[test]
    fn load_ids() {
        let original = br#"(items: { "wood": (id: 1), "stone": (id: 2) })"#;
        let mut items = Items::default();
        items.load_from_bytes(original);

        // Capture ids as if they'd been baked into a save file.
        let wood = items.get_id("wood").expect("wood id");
        assert_eq!(wood, 1);
        let stone = items.get_id("stone").expect("stone id");
        assert_eq!(stone, 2);
        assert!(wood > 0 && stone > 0 && wood != stone);
    }
}
