use crate::{entity::{Entities, Entity, goblin::GOBLIN_FACTION}, game::GameCtx, grid::{Grid, Pos}, job::Job};


#[derive(Clone, Copy)]
pub enum FightAction {
    Attack,
    Watch,
    Defend,
}

pub fn fight(grid: &mut Grid, pos: Pos, action: FightAction, game_ctx: &mut GameCtx, entities: &mut Entities) {
    
    let job = match action {
        // TODO: WatchManager that resets the watch forever until canceled...
        FightAction::Watch => {
            Job::watch(pos).create(grid, game_ctx);
            return;
        },
        FightAction::Attack => Job::fight(pos, (GOBLIN_FACTION, 0)),
        FightAction::Defend => Job::defend(pos),
    };

    for entity in entities.values_mut() {
        if let Entity::Gnome(gnome) = entity {
            gnome.order(job.clone(), grid, game_ctx);
        }
    }
}
