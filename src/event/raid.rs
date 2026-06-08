use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entities, gnome::GNOME_FACTION},
    game::{GameCtx, time::Season},
    grid::Grid,
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RaidManager {}

impl RaidManager {
    pub fn update(game_ctx: &mut GameCtx, grid: &mut Grid, entities: &mut Entities) {
        if game_ctx.time.season_start(Season::Spring) {
            log::info!("Spawning RAID");
            // I have no idea how to pick this for real, let's just try 2?
            let goblin_count = entities.population(GNOME_FACTION) / 2;
            for _ in 0..goblin_count {
                entities.spawn_goblin((0, 0).into(), grid);
            }
        }
    }
}
