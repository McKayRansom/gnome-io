use crate::{entity::{BaseEntity, Faction}, event::EventManager, grid::{Grid, Pos}, job::Job, tile::{Content, Tile}};

pub enum PathOutcome {
    Reached(Pos),
    Path(Vec<Pos>),
    NoPath,
}

pub type JobSearchFn = fn(Pos, &Tile, &EventManager) -> Option<Job>;

// Grid pathfinding function
impl Grid {
    // pub fn successors(&self, pos: &Pos) -> Option<

    pub fn find_path(&self, start: Pos, end: Pos, faction: Faction) -> PathOutcome {
        self.find_path_or_content(start, end, None, faction)
    }

    pub fn find_content(&self, start: Pos, content: Content, faction: Faction) -> PathOutcome {
        self.find_path_or_content(start, start, Some(content), faction)
    }

    fn find_path_or_content(
        &self,
        start: Pos,
        end: Pos,
        content: Option<Content>,
        faction: Faction,
    ) -> PathOutcome {
        if let Some(mut path) = pathfinding::prelude::bfs(
            &start,
            |pos| {
                // check adjacent walls
                if self
                    .get_tile(*pos)
                    .is_some_and(|tile| tile.is_passable(faction))
                {
                    Some([
                        Pos::new(pos.x + 1, pos.y),
                        Pos::new(pos.x - 1, pos.y),
                        Pos::new(pos.x, pos.y + 1),
                        Pos::new(pos.x, pos.y - 1),
                    ])
                    .into_iter()
                    .flatten()
                } else {
                    None.into_iter().flatten()
                }
            },
            |pos| {
                if let Some(content) = content {
                    self.get_tile(*pos)
                        .is_some_and(|tile| tile.contains(&content))
                // } else if is_passable {
                // pos == &end
                } else {
                    pos == &end
                    // pos.diff(end) <= 1
                }
            },
        ) {
            if path.len() <= 1
                || (path.len() == 2
                    && !self
                        .get_tile(path[1])
                        .is_some_and(|tile| tile.is_passable(faction)))
            {
                // We are at the destination already
                // OR
                // we cannot reach the destination, but we've decided we can reach things 1-away so we can mine, fight, etc...
                PathOutcome::Reached(*path.last().unwrap())
            } else {
                // TODO: I hate this but it does seem to be needed or else the first time the entity tries to move it will be to the current pos
                // this can clearly also be handled there, but why should it?
                assert_eq!(path.remove(0), start);
                PathOutcome::Path(path)
            }
        } else {
            PathOutcome::NoPath
        }
    }

    pub fn find_job(
        &mut self,
        entity: &BaseEntity,
        events: &mut EventManager,
        searches: &[JobSearchFn],
    ) -> Option<Job> {
        let mut found_job: Option<Job> = None;
        // we will continue past the first job we find, to see if we find a better one...
        // TOOD: this probably needs to be much large and dynamic?
        // Or we may starve important jobs that are far away
        // let mut continue_past: usize = 16;

        for pos in pathfinding::prelude::bfs_reach(entity.pos, |pos| {
            // check adjacent walls
            if self
                .get_tile(*pos)
                .is_some_and(|tile| tile.is_passable(entity.faction))
            {
                Some([
                    Pos::new(pos.x + 1, pos.y),
                    Pos::new(pos.x - 1, pos.y),
                    Pos::new(pos.x, pos.y + 1),
                    Pos::new(pos.x, pos.y - 1),
                ])
                .into_iter()
                .flatten()
            } else {
                None.into_iter().flatten()
            }
        }) {
            if let Some(tile) = self.get_tile(pos) {
                for search in searches {
                    if let Some(new_job) = search(pos, tile, events) {
                        if found_job
                            .as_ref()
                            .is_none_or(|job| new_job.is_higher_priority(&job))
                        {
                            found_job = Some(new_job);
                        }
                    }
                }

                // if found_job.is_some() {
                //     continue_past -= 1;
                // }
                // if continue_past == 0 {
                //     break;
                // }
            }
        }
        if let Some(job) = &mut found_job {
            job.in_progress = true;
            if job.id == 0 {
                job.id = events.add_job(job.clone());
                self.create(job.pos, Content::Job(job.id));
            }
            events.job_in_progress(job);
        }
        found_job
    }
}
