use crate::{
    entity::{BaseEntity, EntityBehaviour, EntityId, Faction, gnome::{GNOME_FACTION, GNOME_SPEED}},
    game::time::hours,
    grid::Pos, tile::Content,
};

pub struct Goblin {
    base: BaseEntity,
}

pub const GOBLIN_FACTION: Faction = 2;

const BASE_HEALTH: u8 = 10;
const BASE_FOOD: u16 = hours(20);

impl Goblin {
    pub fn new(id: EntityId, pos: Pos, grid: &mut crate::grid::Grid) -> Goblin {
        grid.gnome_enter(pos, (GOBLIN_FACTION, id));

        Goblin {
            base: BaseEntity {
                id,
                faction: GOBLIN_FACTION,
                pos,
                health: BASE_HEALTH,
                food: BASE_FOOD,
                ..Default::default()
            },
        }
    }
}

impl EntityBehaviour for Goblin {
    fn update(
        &mut self,
        grid: &mut crate::grid::Grid,
        ctx: &mut crate::game::GameCtx,
    ) -> Option<super::EntityAction> {
        if let Some(action) = self.base.update(grid) {
            return Some(action);
        }
        if self.base.timer > 0 {
            return None;
        }
        if let Some(path) = grid.find_path(self.base.pos, self.base.pos, Some(Content::Entity((GNOME_FACTION, 0)))) {
            if path.len() > 2 {
                self.base.move_to(path[1], GNOME_SPEED, grid);
                None
            } else {
                let attack_pos = path[path.len() - 1];
                Some(super::EntityAction::Attack(grid.get_tile(attack_pos).unwrap().get_entity((GNOME_FACTION, 0))))
            }
        } else {
            self.base.move_random(grid);
            None
        }
    }

    fn die(&self, grid: &mut crate::grid::Grid, _ctx: &mut crate::game::GameCtx) {
        self.base.die(grid);
    }

    fn attacked(&mut self) {
        self.base.attacked();
    }

    fn base(&self) -> &BaseEntity {
        &self.base
    }
}
