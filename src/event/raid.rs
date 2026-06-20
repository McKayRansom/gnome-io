use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entities, Entity, gnome::GNOME_FACTION, goblin::GOBLIN_FACTION}, event::{EventTypes, FACTION_EXIST_EVENT}, game::{GameCtx, time::Season}, grid::Grid
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct RaidManager {}

impl RaidManager {
    pub fn update(game_ctx: &mut GameCtx, grid: &mut Grid, entities: &mut Entities) {
        if game_ctx.time.season_start(Season::Winter) {
            log::info!("Spawning RAID");
            // I have no idea how to pick this for real, let's just try 2?
            let goblin_count = entities.population(GNOME_FACTION) / 2;
            for _ in 0..goblin_count {
                entities.spawn_goblin((0, 0).into(), grid);
            }
        }

        // muster here for now???
        if let Some(event) = game_ctx.events.peek_event(FACTION_EXIST_EVENT) {
            if let EventTypes::FactionExistsEvent(faction, exists) = event.value {
                if faction == GOBLIN_FACTION {
                    for entity in entities.values_mut() {
                        if let Entity::Gnome(gnome) = entity {
                            gnome.set_muster(exists, grid, game_ctx);
                        }
                    }
                }
            }
        }
    }
}
