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
        // clear any old data...
        // assert!(
        //     self.item_list.is_empty(),
        //     "Items already loaded! Not safe to proceed!"
        // );

        // load new stuff
        let ron_str = macroquad::file::load_file("assets/data/items.ron")
            .await
            .expect("Failed to load items.ron");
        self.load_from_bytes(&ron_str);
    }

    fn load_from_bytes(&mut self, bytes: &[u8]) {
        let items_save: ItemsSave =
            ron::de::from_bytes(&bytes).expect("Failed to deserialize items!");

        // Do item_ids first.
        //
        // ID stability: item_ids is persisted in the save, so any name we've
        // already assigned keeps its id (the contains_key guard). New items are
        // appended past the current max id so they never collide with ids baked
        // into existing saves. Sorting the new names keeps first-time assignment
        // deterministic instead of depending on hash iteration order.
        let mut next_id: ItemId = self.item_ids.values().copied().max().unwrap_or(0) + 1;
        let mut new_names: Vec<&String> = items_save
            .items
            .keys()
            .filter(|name| !self.item_ids.contains_key(*name))
            .collect();
        new_names.sort();
        for item_name in new_names {
            self.item_ids.insert(item_name.clone(), next_id);
            next_id += 1;
        }

        for (item_name, item_type_save) in items_save.items.iter() {
            let id = self.item_ids[item_name];
            let info = item_type_save.convert(item_name, &self.item_ids);
            if !self.item_list.contains_key(&id) {
                self.item_list.insert(id, info);
            } else {
                if self.item_list[&id] != info {
                    log::warn!("Item info changed for item {}", item_name);
                }
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
    fn test_load_items() {
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

    // Saves persist item_ids and store raw ItemId values inside the world, so an
    // id assigned to a name must never change when items.ron is later edited.
    #[test]
    fn ids_are_stable_when_data_changes() {
        let original = br#"(items: { "wood": (), "stone_item": () }, aliases: {})"#;
        let mut items = Items::default();
        items.load_from_bytes(original);

        // Capture ids as if they'd been baked into a save file.
        let wood = items.get_item_id("wood").expect("wood id");
        let stone = items.get_item_id("stone_item").expect("stone id");
        assert!(wood > 0 && stone > 0 && wood != stone);

        // Reload a newer data file that adds an item, with the previously
        // assigned ids already present (as they would be coming from a save).
        let extended = br#"(items: { "wood": (), "stone_item": (), "gold": () }, aliases: {})"#;
        items.load_from_bytes(extended);

        // Existing ids must not move...
        assert_eq!(items.get_item_id("wood"), Some(wood));
        assert_eq!(items.get_item_id("stone_item"), Some(stone));
        // ...and the new item gets a fresh, non-colliding id.
        let gold = items.get_item_id("gold").expect("gold id");
        assert_ne!(gold, wood);
        assert_ne!(gold, stone);
    }
}
