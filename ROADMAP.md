# Roadmap

# Current epic

# MVP3: Kingdom

Focus on survival: The player has to allocate resources to:
- food: need enough food to survive winter/spring (this should be difficult the first year)
- military: need enough soldiers/weapons to survive goblin raids (also could be difficult)
- building: beds/storage/walls/etc...
- future growth: have kids (more resources but also more manpower?)

So the proffesions would be:
- farmer: 
  - 1st harvest fruit
  - 2nd plant trees
  - 3rd bake bread ???
- soldier:
  - 1st fight enimies
  - 2nd Stand watch
- builder
  - 1st build
  - 2nd haul
- parent
  - 1st tend kids
  - 2nd have kids
- miner
  - 1st mine
  - 2nd haul?
- crafter???
  - 1st bake bread
  - 2nd craft weapons
  - 3rd smelt ore

Key Decisions to make:
 - Do gnomes level up? This would shape a lot of the decisions around allocation and personel
  - If yes: 
    - Exactly which gnomes are assigned to which task REALLY matters
    - Need insight into each gnomes' stats
    - Have to make difficult decisions about highly skilled gnomes
  - If no:
    - managment is MUCH easier and more about short-term
    - no loss in throwing everyone into farming or fighting...
    - could be good starting ground, and add or experiment with level-up later
 - How are gnomes assigned to professions?
  - I really want something visual, and I don't want spreadsheets
  - Can we please avoid creating own professions? Base ones could somehow be good enough?

- [ ] Re-enable stakes
  - [x] Fix starvation ([gnome.rs:184-189](src/entity/gnome.rs#L184-L189))
    -[x] Fix eating wacking out
  - [x] Fix plants/farming ([grid.rs:349](src/grid.rs#L349))
    - [x] Requires minor rework to how block events work with farms
    - [x] Add plants to world-get (but those won't be managed by farms unless added)
  - [x] Fix indicators
    - [x] Need some kind of indicator that a tile is farmed...
    - [x] Better job indicators
  - [x] Wire goblins back into generation + combat into the gnome update loop, as periodic raids.
    - [x] WINTER IS COMING!
  - [x] Fix bread/crafting?
  - [x] Add snow/winter for real
    - [ ] kill plants
    - [ ] Moves slow?
- [ ] Create attack/flee logic
    - [x] Create attack job and fix many, many bugs
    - [x] Make enemy entities unpathable (for fights and for doors...)
    - [ ] Create flee job when it makes sense to do so
    - [x] Add grave blocks
- [ ] Create parrenting logic
- [ ] Professions/labor allocations
  - [ ] Add proffesions
    - [ ] Auto-assign gnomes for debugging
  - [ ] Create proffesions UI
    - [ ] Draw status
    - [ ] Allocate gnomes
    - [ ] Show open jobs
    - [ ] Fix stocks UI while we're there
- [ ] Create tools OR some other way to allocate resources to enhance one profession (forces more choices...)
  - [ ] Fix ore-gen and add ore/metal blocks/items/recipes
  - [ ] Add equipment item enum/attribute
  - [ ] Add sprites/drawing
  - [ ] Make required for profession and/or jobs
  - [ ] Picaxe/hoe/armor/etc...
- [ ] Add sunlight to tiles
  - [ ] use this to drive plant growth
  - [ ] use for drawing tile color
- [x] Add sight
  - [x] Use to detect the baddies attacking

- [x] Sidetrack Job Step refactor (it's really cool tho...)

We need a way to store block metadata:
- Is there a farm here
- Is this furnace WIP, and what does it grow into
- Is this plant growing and what does it grow into

We can continue to store it in events for now but is this really the best place? 
Eventually I think there will be static block info that should go somewhere else...

Profession assignment:
- Columns: Free, farm, soldier, mine, etc...
  - Shows: 
    - Available slots for miner/soldier that require armor (so you can't dump every-one into fight/mine/etc... could add option with penalty)
    - Green for working, Blue for idle, purple # of jobs outstanding (or below?)
    - left-click to allocate from free pool (forces decisions!)
    - right-click to move to free pool (deallocate from specicifc task)

Once the colony gets huge, We could switch to 5s of gnomes or a %-based view
  
Examples:
- I look at board, notice that I have 100 outstanding farm jobs, so I move some crafters over to help with the harvest
  - This is only possible because I have allocated metal to hoes, instead of weapons, giving more food but less military (tradeoffs!)
- I look at the board and see there are no mining/building jobs outstanding so I go create some
- I look at the board and see there are no farming jobs and it is till ealy spring, so I add gnomes to that
- A large goblin attack is incoming, I allocate all the soliders I have and check some sort of box to have gnomes fight unarmed
  - This has a higher risk of getting gnomes killed, but surviving the attack
- A single monster attacks, I allocate only a few gnomes to deal with it in heavy armor
- I decide we can afford to allocate a few gnomes to have children, then are "locked-in" to this role until their kids are raised
  - This is a semi-permanant decision OR once they are born we could re-alloc?

# Future Epics
Planned items, unplanned go in ICEBOX.md. Roughly in priority order:

## Polish for first release
- Performance pass: Add benchmarks and tests, take a pass at low-handing fruit
- Animations/draw polish
- Lighting/limit view to in-sight
- UI: Title screen/menus/settings/saves/autosave
- Error handling: Clean reporting of save/load errors with popup, and pass at unwraps/expects
- onboarding: tutorial/objectives/help
- Webasm build
- Soundfxs
- Music
- Itch-io: Polising and testing for itch.io release!

## Explore future direction
- Gnome skill exploration/gnome details
- faction exploration
- multiplayer exploration
- mod exploration

## Content Updates
- Life update: trees that grow high, funguses down below, animals and stuff
- Delved too deep update: Caves/enimies in the deep as well as rewards
- Biome update: Desert/forest/mountains/ocean changes how you build your colony and what you face
- Automation update: Machines/carts/etc

# Minor tasks
- Color pallate pass
- Tile biome remove?
- Unit test pass
- controls pass: Fix menu shortcuts, zoom into point, right-click pan
- debug features: Show reachable, more info, etc...

# Past Epics

## MVP1: Survive Winter (requires seasons) 
-  Tile-based drawing and sprites
-  Procedural terrain
-  Mining
-  Farming
-  Crafting (Just crafting bread automatically for now)
-  Saving
-  UI basics (Toolbar, Menu, tile details, skin, stocks)
Thoughts after MVP completion:
- Balance of timing is tricky, seasons feel long but days are too short (Gnomes barely reach 30 tiles)
- Bug testing is a little tricker because the reproduce time is longer. Will need read savefiles and command line.
- Some kind of scripting or JSON for the block/item/sprite definitions would make so much sense
- Managing of different struct mutability in Rust is hard, but at this point it would be so much work to switch
- Maybe Rust can be justified due to performance + correctness of rare edge cases
- Start on unit tests once structure is finalized? At the very least start adding tests for bug cases
- Have enimies show up in the winter? (Winter is coming?) (White walkers?)
- Hauling structures like in Beaver game

## MVP2: Sideview
Goal: Test out all current features on a side-view
- [x] Map generation/size change
  - [x] Fix Ore gen
  - [x] Fix trees (Do we need to even fix trees????)
  - [x] Fix Plants not being able to be planted on air biome
- [x] Pathfinding change
- [x] Climbing or ramps or something???
- [x] Graphics
  - [x] Make not as shitty
  - [x] Switch to cute bushbes?
Thoughts after completion:
- OBSESSED with the art style, super cute
- White-on-white can get difficult to read, may need some sort of border/background/thin line around sprites to make them not merge together
- Little animations like jumping up and down when performing an action would go a long way
- Theme from gnorp works well for now, but I think we will want to tweak the color pallate before release
- Does go with the minimalistic theme
- I am concerned about running out of ore/stone to mine eventually but there are solutions to this:
  - Some expensive way to get infinite resources
  - Recycling so stuff is never truely gone
  - Ways to get around the map faster 
- I do want to keep some distinguishing colors between above/below ground, but the current tile biome may need to be dropped
  - Replace with light/dark shaders based on sunlight/artifical light? (Shaders famously go a long way!)
  - Current system but with some kind of blending???
- Medium sidetrack to add haul/drop jobs
- Gigantic sidetrack to load blocks.ron and items.ron
- Medium sidetrack to add blockflags and tileflags
- Medium sidetrack to rework job finding to not be a total mess
