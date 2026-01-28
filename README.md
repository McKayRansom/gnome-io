
# Gnome-IO
2d colony-sim inspired by Gnomoria but designed to be simpler. 
There is an urge (I do it to) when programming a video game to add every fun feature imaginable, 
and have many types of plants and animals and blocks that do essentially the same thing (see minecraft's development).
I don't think this adds depth to games. 1 type of everything and people can add mods later if they really want to.
I hope this game will still be interesting because of the greater interaction between factions and a more dynamic map.
The point should be more trying to survivie (rimworld but way simpler) an interseting simulation (sim ant) than complicated base builder (but I do want building a well-protected, productive base to be important to survival).
The one exception is I want there to be multiple races that play very differently and interact in interesting ways.
This will be almost impossible to balance, but I'm hoping to make it casual enough that it doesn't super matter.

# MVP: Survive Winter (requires seasons) 
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

# MMVP: Kingdom
- Four supply chains:
  - Grain -> Bread
  - Ore -> Metal -> Weapons?
  - Wood -> ??
  - Soldiers will pick up available weapons
- Assign to professions FTL style

## Features
- Grow a colony of gnomes and other fantasy races and survive the seasons and goblin raids.
- Procedual world generation
- Simplified and visual-based whenever possible. All features should have trade-offs with no optimal solution. 
- Performant and highly-scalable to large maps and large numbers of gnomes
- Highly abstracted and modable
- Eventually multiplayer co-op or vs


### Modding design
Factorio modding design seems reasonable. Lua files for settings, data, and control.
Settings and data are called on game load, control can register event handlers.

Seems pretty reasonable, but a large amount of work. The changes they had to make to make lua deterministic are concerning. (Although it looks like the default rust HashMap is also non-deterministic to prevent DOS). Having a dedicated storage table in lua for saving and multiplayer also makes sense. 

A first step would be just data creation, control could be done later.


## Gnomoria comparisons:

### Added features
- More easily modable in a scripting language rather than XML
- scales to larger numbers
- Visual display of stats instead of just a number
- Simulate enemy or friendly factions
  - Gnomes: well-balanced overall, can grow wheat and mushrooms, can mine and build okay and reproduce medium
  - Elves: great with forests and animals, stronger but reproduce slower (if this is even possible to balance, not required)
  - Goblins: Hide in mountains, okay at mining but mostly steal.
  - Orcs: Can poorly build stuff and farm, mostly have to survive by stealing (nomadic?)
  - Dwarves: Very good at mining, as long as they don't mine too greedily or too deep. Poor at farming and do best trading metals for foods
  - Men: Good at building castles and forts above ground, can mine okay
  - Modded factins: I.E. Freemen that live only in deserts, spiders, mants, beetles
  would add replayability in a more interesting way than just more items.
- Proceduraly generated instead of fixed map size?
- More interesting terrain: Biomes(Deserts, forests, mountains), water, oceans (boats?)

### Removed features
- Lighting up the mines: This is just busy work IMO
- Depth levels: Hard to visualize and traverse
- Beatles obviously
- Fixed screen size
- Duplicate/similar items. The base game will probably just have 1 type of food and 1 type of ore, 
Maybe mods can add this but I want to keep the base game as simple as possible.
- Mechanisms: Add mod support for these sorts of features if time
- Complex damage simulation

### Undecided:
- Realistic Pixel (Gnomoria) vs Minimalistic Pixel (Gnorp) graphics
- Base builder OR RTS?? How does every game I make turn into an RTS!
- Traits/professions (We need some way to decide which tasks are prioritized...)
- Mining vs infinite mining workshops? how well does this work on 2d
- Enemies in the mines... Would be cool to mine too greedily and too deep
- Combat: Could be simplified, could keep it similar
- Stockpiles: Simpler would be nice but how would that be done?
  - Consider specific building size like in Timberborn.
- Professions at all? is a lot of work to do this...

### Ideas:
#### Side-view instead of top-down
 - goes better with gnorp style we were considering
 - Goes better with building "big" and "tall" kingdoms
 - potentially simplifies a lot!
 - Easier graphically let's be honest
 - Simplifies combat a lot (just pick left or right lol)
 - Allows mining clearly and understandably
 - Downside: multiplayer is now inheirly unfair because someone is in the midde (fix: looping world!!!)
 - Downside: walls are CHEAP AF now??? (fix: unclimbable walls are VERY expensive...)

      -----------
      |bbb  bbbb|     |
      -----------     |
 . .  |     cccc|  ...|
*********** *************
*********** *************
*********** *************
*********** *************
*********** *************
*******           ********
*********** *************
*********** *************


Choices:
High-level priorites: Mining vs Farming vs Woodcutting?
Building layout still b/c that's fun
Attack/defend priorities?

Allocate people FTL-style with bars?

Factions:
- Mants: Only move via hormones, limited high-level control but can eat plants and reproduce faster, weaker in combat but numbers
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

Bees:
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

Millipede/Caterpillar
  - Move around psudo-random (but in a direction) looking for food
  - Eventually lay eggs

Spider:
  - Move around psudeo-random looking for ants
  - Eventually lay eggs

Pillbug:
  - Move around psudeo-random
  - Invulnerable OR Curl into ball



- Dwarves: Give jobs, limited attack/defend, good at mining
- Humans: good at building
- Goblins: Hide in mountains and steal food
- Orcs: Only attack and steal stuff

