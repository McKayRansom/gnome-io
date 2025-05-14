use std::collections::HashMap;

use quad_lib::tileset::Sprite;

use crate::{block::BlockId, draw::sprites};

pub type ItemId = u32;

pub struct ItemType {
    pub name: &'static str,
    pub sprite: Sprite,
    pub recipe: Option<(BlockId, Vec<ItemId>)>,
    // food value?
}

impl Default for ItemType {
    fn default() -> Self {
        Self {
            name: "unnamed",
            sprite: sprites::UNKNOWN_ITEM,
            recipe: None,
        }
    }
}

impl ItemType {
    pub fn new(name: &'static str, sprite: Sprite) -> Self {
        Self {
            name,
            sprite,
            recipe: None,
        }
    }
}

pub struct Items {
    item_list: HashMap<ItemId, ItemType>,
}

impl Items {
    pub fn new() -> Self {
        Items {
            item_list: HashMap::new(),
        }
    }

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
}
