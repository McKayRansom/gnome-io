# Icebox

Speculative content & faction ideas — captured so they don't get lost, not scheduled.
Pull into the Roadmap only once the core survival loop is playable.

## Entity ideas
We have built a very flexible entity system, could do some cool things
- Livestock: simple entity that eats plants and walks around randomly (needs enclosure)
- Minecart: DO NOT OVERCOMPLICATE THIS, totaly something I want for obvious reasons, could really complicate pathfinding...

## Automation
Obviously there is extreme potential here for factorio-style game:
- Conveyor-belts, hoppers, auto-builders, etc...
- Trains, etc...
I think we could explore this at some point, or at least allow the hooks for mods to add it

## Research
Research options:
- Supply-chain based (gnomoria, keflings) need to build the next thing thing to unlock building the next thing (workshops are expensive-ish or require difficult-to-get items)
- Tech-tree based (factorio) specific research workshop that requires a lot of time and/or resources. 
  Could be interesting from a personel allocation standpoint
  Also nice from a game-discover standpoint, but does require a new UI
- Milestone based (shapez) at specific milestones (pop/items) new things are unlocked automatically

## Modding
I have thought a lot about this, I'm not happy with pulling in lua or another scripting language, but it is possible
For now, let's see how for config files as .ron can get us (I think that's as far as gnomoria ever got!)
Before adding scripting support, there needs to be a clear use case or it will be difficult to get correct

### Modding design
Factorio modding design seems reasonable. Lua files for settings, data, and control.
Settings and data are called on game load, control can register event handlers.

Seems pretty reasonable, but a large amount of work. The changes they had to make to make lua deterministic are concerning. (Although it looks like the default rust HashMap is also non-deterministic to prevent DOS). Having a dedicated storage table in lua for saving and multiplayer also makes sense. 

A first step would be just data creation, control could be done later.


## Traditional Faction ideas

- Gnomes: well-balanced overall, can grow wheat and mushrooms, can mine and build okay and reproduce medium
- Elves: great with forests and animals, stronger but reproduce slower (if this is even possible to balance, not required)
  - LIVE IN TREES, like og minecraft! This would be so fun!
- Goblins: Hide in mountains, okay at mining but mostly steal.
- Orcs: Can poorly build stuff and farm, mostly have to survive by stealing (nomadic?)
- Dwarves: Very good at mining, as long as they don't mine too greedily or too deep. Poor at farming and do best trading metals for foods
- Men: Good at building castles and forts above ground, can mine okay
- Modded factins: I.E. Freemen that live only in deserts, spiders, mants, beetles

# Dwarves: 
Give jobs, limited attack/defend, good at mining
# Humans: 
good at building
# Goblins: 
Hide in mountains and steal food
# Orcs: 
Only attack and steal stuff
# Mants:
Only here for your food, but if they get it they will be back for more!

## Insect faction ideas
Overlap with ant-io? Lots of decisons to be made here, could work really well as enimies.
Also relates to dwarf fortress's adventure mode... For sure leave for later if at all
### Mants: 
  Only move via hormones, limited high-level control but can eat plants and reproduce faster, weaker in combat but numbers
Blocks: 
  - Grass
  - Eggs
  - Food?
  - Nest marker?
Mobs:
  - Queen
  - Worker
  - Soldier
Professions:
  - Gathering: Find closest unjobed food, mark food, and bring back to nest?
  - Nursing: Bring food to queen, move eggs/feed eggs?
  - Digging: Dig out nest?
Unuique:
  - Transform the terrain as they grow by destroying anythign that isn't grass/aphids

### Bees:
Blocks:
  - Flowers
  - Honeycomb
  - Honeycomb w/honey
  - Honeycomb w/larvae
Mobs:
  - Queen
  - Worker
Professions:
  - Gathering: Find closest flower w/pollen, and bring back to nest?

# Millipede/Caterpillar
  - Move around psudo-random (but in a direction) looking for food
  - Eventually lay eggs

# Spider:
  - Move around psudeo-random looking for ants
  - Eventually lay eggs

# Pillbug:
  - Move around psudeo-random
  - Invulnerable OR Curl into ball


## LOTR-inspired
- Dragons that go after gold
- Spiders in evil forests
- Balrogs hidden deep down

## Graphics
- Steal shaders (CRT + Lighting) from that one itch.io game...