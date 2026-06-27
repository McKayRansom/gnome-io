use rustc_hash::FxHashMap;

use crate::{
    entity::Entities, event::Events, game::GameCtx, grid::Grid, item::ItemId, tile::Content,
};

#[derive(Default)]
pub struct Stocks {
    stocks: FxHashMap<ItemId, Stock>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stock {
    available: usize,
    reserved: usize,
    // could add inventories/equiped if desired

    // user-set values
    // minimum: usize,
    pub pinned: bool,
}

impl Stock {
    pub fn total(&self) -> usize {
        self.available + self.reserved
    }
}

const DEFAULT_STOCK: Stock = Stock {
    available: 0,
    reserved: 0,
    pinned: false,
};

impl Stocks {
    #[allow(unused)]
    pub fn total(&self, item: ItemId) -> usize {
        self.stocks.get(&item).copied().unwrap_or_default().total()
    }
    pub fn available(&self, item: ItemId) -> usize {
        self.stocks
            .get(&item)
            .copied()
            .unwrap_or_default()
            .available
    }
    pub fn pinned(&self, item: ItemId) -> bool {
        self.stocks.get(&item).copied().unwrap_or_default().pinned
    }
    pub fn add(&mut self, item: ItemId, events: &mut Events) {
        let avail = &mut self
            .stocks
            .entry(item)
            .or_insert(Stock::default())
            .available;
        *avail += 1;
        if *avail == 1 {
            events.item_appears(item);
        }
    }
    pub fn pin(&mut self, item: ItemId, pinned: bool) {
        self.stocks.entry(item).or_insert(Stock::default()).pinned = pinned;
    }

    pub fn get(&self, item: ItemId) -> &Stock {
        self.stocks.get(&item).unwrap_or(&DEFAULT_STOCK)
    }

    // pub fn reserve(&mut self, item: ItemId) {
    //     let stock = self.stocks.entry(item).or_insert(Stock::default());
    //     stock.reserved += 1;
    //     if stock.available == 0 {
    //         log::error!("Tried to reserve item {} which is not available!", item);
    //     }
    //     stock.available = stock.available.saturating_sub(1);
    // }

    // pub(crate) fn unreserve(&mut self, item: ItemId) {
    //     if let Some(stock) = self.stocks.get_mut(&item) {
    //         if stock.reserved == 0 {
    //             log::error!("Map stock mismatch for item {}", item);
    //         }
    //         stock.reserved = stock.reserved.saturating_sub(1);
    //         stock.available += 1;
    //     } else {
    //         log::error!("Tried to remove from non-existant stock for item: {}", item);
    //     }
    // }

    // Not sure if this belongs here, but it does kina fit...
    pub fn total_food(&self, game_ctx: &GameCtx) -> usize {
        game_ctx.items.iter().fold(0, |acc, (id, item)| {
            if item.food() {
                acc + self.available(*id) * item.food_value as usize
            } else {
                acc
            }
        })
    }

    pub fn remove(&mut self, item: ItemId) {
        if let Some(stock) = self.stocks.get_mut(&item) {
            if stock.available == 0 {
                log::error!("Map stock mismatch for item {}", item);
            }
            stock.available = stock.available.saturating_sub(1);
        } else {
            log::error!("Tried to remove from non-existant stock for item: {}", item);
        }
    }
    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, u32, Stock> {
        self.stocks.iter()
    }
}

pub fn stocks_verify(grid: &mut Grid, _entities: &Entities) {
    let stocks = &grid.stocks;
    let mut new_stocks: Stocks = Stocks::default();
    for y in 0..grid.size.y {
        for x in 0..grid.size.x {
            for content in grid.get_tile((x, y).into()).unwrap().iter_content() {
                if let Content::Item(item) = content {
                    new_stocks
                        .stocks
                        .entry(item.0)
                        .or_insert(Stock::default())
                        .available += 1;
                }
            }
        }
    }
    // for entity in entities.values() {
    //     for item in entity.base().items.iter() {
    //         new_stocks.add(item.0);
    //     }
    // }

    if stocks.stocks.len() > 0 {
        // check against old
        for item in new_stocks.stocks.keys() {
            let old = stocks.get(*item);
            let new = new_stocks.get(*item);
            if old != new {
                log::error!(
                    "Stock mismatch for {}: stock: {:?} actual: {:?}",
                    item,
                    old,
                    new,
                );
            }
        }
        // check for any not in new
        for item in stocks.stocks.keys() {
            let old = stocks.get(*item);
            let new = new_stocks.get(*item);
            if old != new {
                log::error!(
                    "Stock mismatch for '{}': stock: {:?} actual: {:?}",
                    item,
                    old,
                    new,
                );
            }
        }
    } else {
        // new stocks, set it up
        grid.stocks = new_stocks;
    }
}
