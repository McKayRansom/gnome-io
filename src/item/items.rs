use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use super::*;

type ItemList = FxHashMap<ItemId, ItemType>;
type ItemIds = FxHashMap<String, ItemId>;

#[derive(Default, Deserialize, Serialize)]
pub struct Items {
    #[serde(skip_deserializing, skip_serializing)]
    item_list: ItemList,
    // we do need to save this...
    item_ids: ItemIds,
}

// Deserialize-only struct
#[derive(Debug, Clone, Deserialize)]
struct ItemsSave {
    items: FxHashMap<String, ItemTypeSave>,
    aliases: FxHashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ItemTypeSave {
    id: ItemId,
    #[serde(default)]
    sprite: String,
    #[serde(default)]
    recipe: Option<(String, Vec<String>)>,
}

impl ItemTypeSave {
    fn convert(&self, name: &String, items_id: &ItemIds) -> ItemType {
        ItemType {
            name: name.clone(),
            sprite: if self.sprite.is_empty() {
                name.clone()
            } else {
                self.sprite.clone()
            },
            recipe: self.recipe.as_ref().map(|(block_at, ingredients)| {
                (
                    block_at.clone(),
                    ingredients
                        .iter()
                        .map(|item_name| {
                            *items_id.get(item_name).unwrap_or_else(|| {
                                log::error!(
                                    "Could not find ingredient '{}' for item '{}'",
                                    item_name,
                                    name
                                );
                                &ITEM_NONE
                            })
                        })
                        .collect::<Vec<ItemId>>(),
                )
            }),
        }
    }
}

impl Items {
    pub fn add_item(&mut self, id: ItemId, item: ItemType) {
        if let Some(_old) = self.item_list.insert(id, item) {
            log::warn!("Item {} already exists!", id);
        }
    }

    pub fn get_item(&self, id: &ItemId) -> Option<&ItemType> {
        self.item_list.get(&id)
    }

    pub fn _iter_items(&self) -> std::collections::hash_map::Iter<'_, u32, ItemType> {
        self.item_list.iter()
    }

    pub fn get_item_id(&self, name: &str) -> Option<ItemId> {
        self.item_ids.get(name).copied()
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

        for (item_name, item_type_save) in items_save.items.iter() {
            let id = item_type_save.id;

            if let Some(old_id) = self.item_ids.insert(item_name.clone(), id) {
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

        for (item_name, item_type_save) in items_save.items.iter() {
            let id = item_type_save.id;
            let info = item_type_save.convert(item_name, &self.item_ids);
            if !self.item_list.contains_key(&id) {
                self.item_list.insert(id, info);
            } else {
                if self.item_list[&id] != info {
                    log::warn!("Item info changed for item {}", item_name);
                }
            }
        }

        // verify we aren't missing anything...
        for (name, id) in self.item_ids.iter() {
            if !self.item_list.contains_key(id) {
                log::error!("Item definition not found for '{}' (id: {})", name, id);
            }
        }

        for (alias, name) in items_save.aliases.iter() {
            self.item_ids.insert(
                alias.clone(),
                *self.item_ids.get(name).unwrap_or_else(|| {
                    log::error!("No item name '{}' for alias '{}'", name, alias);
                    &ITEM_NONE
                }),
            );
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

        assert!(items.item_ids.get("stone").is_some_and(|id| *id > 0));
        let stone_id = items.item_ids["stone"];

        assert!(
            items
                .item_list
                .get(&stone_id)
                .is_some_and(|info| info.recipe.is_none() && info.sprite == "stone_item")
        );
    }

    #[test]
    fn load_ids() {
        let original = br#"(items: { "wood": (id: 1), "stone_item": (id: 2) }, aliases: {})"#;
        let mut items = Items::default();
        items.load_from_bytes(original);

        // Capture ids as if they'd been baked into a save file.
        let wood = items.get_item_id("wood").expect("wood id");
        assert_eq!(wood, 1);
        let stone = items.get_item_id("stone_item").expect("stone id");
        assert_eq!(stone, 2);
        assert!(wood > 0 && stone > 0 && wood != stone);
    }
}
