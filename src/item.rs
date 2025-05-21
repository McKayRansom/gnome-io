use std::collections::HashMap;

use crate::{block::BlockId};

pub type ItemId = u32;

pub mod items;

pub struct ItemType {
    pub name: &'static str,
    pub sprite: String,
    pub recipe: Option<(BlockId, Vec<ItemId>)>,
    // food value?
}

impl Default for ItemType {
    fn default() -> Self {
        Self {
            name: "unnamed",
            sprite: "unknown".into(),
            recipe: None,
        }
    }
}

impl ItemType {
    pub fn new(name: &'static str, sprite: String) -> Self {
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

impl Default for Items {
    fn default() -> Self {
        let mut items = Self { item_list: Default::default() };
        items::init(&mut items);
        items
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
}
