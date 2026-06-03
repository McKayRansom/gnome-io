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

## Modding
I have thought a lot about this, I'm not happy with pulling in lua or another scripting language, but it is possible
For now, let's see how for config files as .ron can get us (I think that's as far as gnomoria ever got!)
Before adding scripting support, there needs to be a clear use case or it will be difficult to get correct

## Traditional Faction ideas

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
