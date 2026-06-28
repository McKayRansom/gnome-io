use crate::{
    entity::{
        BaseEntity, DEFAULT_SPEED, EntityBehaviour, EntityId, Faction, HIDDEN_FACTION,
        gnome::GNOME_FACTION,
    },
    game::{Tick, time::hours},
    grid::{Pos, path::PathOutcome},
    item::ItemInfoFlags,
    tile::Content,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Goblin {
    pub base: BaseEntity,
    #[serde(default)]
    pub status: GoblinStatus,

    pub hide_delay: Tick,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub enum GoblinStatus {
    #[default]
    None,
    Fighting,
    Mining(Pos),
}

pub const GOBLIN_FACTION: Faction = 2;

const BASE_HEALTH: u8 = 10;
const BASE_FOOD: u16 = hours(20);
const VISIBLE_DISTANCE: i16 = 10;
const VISIBLE_TIME: Tick = hours(4);

impl Goblin {
    pub fn new(id: EntityId, pos: Pos, _grid: &mut crate::grid::Grid) -> Goblin {
        // don't enter the grid yet, we may want to be hidden...
        // grid.entity_enter(pos, (GOBLIN_FACTION, id));

        Goblin {
            base: BaseEntity {
                id,
                faction: HIDDEN_FACTION,
                pos,
                health: BASE_HEALTH,
                food: BASE_FOOD,
                ..Default::default()
            },
            status: GoblinStatus::None,
            hide_delay: 0,
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
        match self.status {
            GoblinStatus::None => {}
            GoblinStatus::Fighting => {}
            GoblinStatus::Mining(pos) => {
                // dig pos
                grid.destroy_block(pos, ctx);
            }
        }
        self.status = GoblinStatus::None;

        // TODO: Find path including tunneling
        match grid.find_path_or_destroy(
            self.base.pos,
            Content::Entity((GNOME_FACTION, 0)),
            GOBLIN_FACTION,
        ) {
            PathOutcome::Path(path) => {
                // hide
                if path.last().unwrap().diff(self.base.pos).abs() > VISIBLE_DISTANCE {
                    self.hide_delay = self.hide_delay.saturating_sub(1);
                } else {
                    // don't disapear immediatly, it's wack
                    self.hide_delay = VISIBLE_TIME;
                }

                if self.hide_delay == 0 {
                    self.base.faction = HIDDEN_FACTION;
                } else {
                    self.base.faction = GOBLIN_FACTION;
                }

                let next_pos = path[0];
                if !self.base.move_to(next_pos, DEFAULT_SPEED, grid) {
                    // we have reached an obstacle
                    let tile = grid.get_tile(next_pos).unwrap();
                    // first attack
                    if let Some(Content::Entity((_faction, id))) =
                        tile.find(&Content::Entity((GNOME_FACTION, 0)))
                    {
                        // attack!
                        self.base.timer = super::FIGHT_TIME;
                        self.status = GoblinStatus::Fighting;
                        return Some(super::EntityAction::Attack(id));
                    }
                    // 2nd try to remove block (Could be solid, could be gate...)
                    if tile.has_block() {
                        // remove block (after a delay...)
                        self.base.timer = hours(4);
                        self.status = GoblinStatus::Mining(next_pos);
                        return None;
                    }

                    // how did we get here???
                    panic!("Goblin unable to move into tile: {:?}", tile);
                }
                None
            }
            PathOutcome::Reached(attack_pos) => {
                self.base.timer = super::FIGHT_TIME;
                self.status = GoblinStatus::Fighting;
                Some(super::EntityAction::Attack({
                    let Content::Entity((_faction, id)) = grid
                        .get_tile(attack_pos)
                        .unwrap()
                        .find(&Content::Entity((GNOME_FACTION, 0)))
                        .unwrap()
                    else {
                        panic!();
                    };
                    id
                }))
            }
            PathOutcome::NoPath => {
                // TODO: Remove this case...
                self.base.faction = HIDDEN_FACTION;
                self.base.move_random(grid);
                None
            }
        }
    }

    fn die(&mut self, grid: &mut crate::grid::Grid, game_ctx: &mut crate::game::GameCtx) {
        grid.create(
            self.base.pos,
            Content::Item((
                game_ctx.items.get_id("goblin_dead").unwrap(),
                ItemInfoFlags::default(),
            )),
            &mut game_ctx.events,
        );
        self.base.die(grid, &mut game_ctx.events);
    }

    fn attacked(&mut self) {
        self.base.attacked();
    }

    fn base(&self) -> &BaseEntity {
        &self.base
    }
}
