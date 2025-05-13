use crate::{
    game::{GameCtx, Tick},
    grid::{Pos, pos},
    item::ItemId,
    job::Job,
    tile::Entity,
};

pub type GnomeId = u32;

pub struct Gnome {
    pub id: GnomeId,
    pub job: Option<Job>,
    pub pos: Pos,
    pub path: Vec<Pos>,
    pub timer: Tick,
    pub items: Vec<ItemId>,
}

const GNOME_SPEED: Tick = 20;

impl Gnome {
    pub fn new(id: GnomeId, pos: Pos, grid: &mut crate::grid::Grid) -> Gnome {
        grid.gnome_enter(pos, id);

        Gnome {
            id,
            job: None,
            pos,
            path: Vec::new(),
            timer: 0,
            items: Vec::new(),
        }
    }

    pub fn update(&mut self, grid: &mut crate::grid::Grid, game_ctx: &mut GameCtx) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }

        if !self.path.is_empty() {
            // Move towards the destination

            let new_pos = self.path.remove(0);

            if grid
                .get_tile(new_pos)
                .is_none_or(|tile| !tile.is_passable())
            {
                // impassable terrain... abort?
                self.path.clear();
                return;
            }

            grid.gnome_exit(self.pos, self.id);
            self.pos = new_pos;
            grid.gnome_enter(self.pos, self.id);
            self.timer = GNOME_SPEED;
            return;
        }

        // find a new job first
        if self.job.is_none() {
            if let (Some(path), Some(job)) = grid.find_job(self.pos, &mut game_ctx.events) {
                self.path = path;
                self.job = Some(job);
            }
        }

        if let Some(job) = &mut self.job {
            // collect items
            for required_item in job.requires.iter() {
                if !self.items.contains(required_item) {
                    if let Some(item) = grid.remove_entity(self.pos, Entity::Item(*required_item)) {
                        let Entity::Item(item) = item else { panic!() };
                        self.items.push(item);
                    } else if let Some(path) =
                        grid.find_path(self.pos, job.pos, Some(*required_item))
                    {
                        self.path = path;
                        return;
                    } else {
                        // job.unreachable();
                        log::warn!("Unable to find item {} for job", required_item);
                        // JobManager::fail_job(&mut game_ctx.events, self.job.take().unwrap());
                        // TOdo; Fail job...
                        // job_manager.failed_job(pos);
                        game_ctx.events.remove_job(job.id);
                        grid.remove_entity(job.pos, Entity::Job(job.id));
                        return;
                    }
                }
            }
            if self.pos.diff(job.pos) > 1 {
                // we found the path earlier...
                if !self.path.is_empty() {
                    return;
                }
                // this needs to be changed for unreachable terrain...
                if let Some(path) = grid.find_path(self.pos, job.pos, None) {
                    self.path = path;
                } else {
                    // job.unreachable();
                    log::warn!("Job at {:?} is unreachable", job.pos);
                    // JobManager::fail_job(&mut game_ctx.events, self.job.take().unwrap());
                    // job_manager.failed_job(pos);
                    // self.job = None; // Job is unreachable, remove it
                    game_ctx.events.remove_job(job.id);
                    grid.remove_entity(job.pos, Entity::Job(job.id));
                }
                return;
            }
            // we are here!
            if job.time > 0 {
                self.timer = job.time;
                job.time = 0;
                return;
            }
            // perform the job
            self.items.retain(|item| !job.requires.contains(item));
            let _ = match job.entity {
                Some(Entity::Item(item_id)) => grid.add_entity(job.pos, Entity::Item(item_id)),
                Some(Entity::Block(block_id)) => {
                    // jank city, population Jank Jankerton III
                    // move ourself out of the way so we're not stuck
                    for pos in pos::dirs::ALL {
                        let new_pos = self.pos + pos;
                        if grid
                            .get_tile(new_pos)
                            .is_some_and(|tile| tile.is_passable())
                        {
                            grid.gnome_exit(self.pos, self.id);
                            grid.gnome_enter(new_pos, self.id);
                            self.pos = new_pos
                        }
                    }

                    grid.place_block(job.pos, Some(block_id), game_ctx)
                }
                None => grid.place_block(job.pos, None, game_ctx),
                Some(_) => todo!(),
            };
            game_ctx.events.remove_job(job.id);
            grid.remove_entity(job.pos, Entity::Job(job.id));
            log::info!("Finished job: {:?}", job);
            self.job = None;
        } else {
            self.timer = 30;
        }
    }
}
