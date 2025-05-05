
# Gnome-IO
2d colony-sim inspired by Gnomoria but designed to be simpler. 
There is an urge (I do it to) when programming a video game to add every fun feature imaginable, 
and have many types of plants and animals and blocks that do essentially the same thing (see minecraft's development).
I don't think this adds depth to games. 1 type of everything and people can add mods later if they really want to.
I hope this game will still be interesting because of the greater interaction between factions and a more dynamic map.
The point should be more trying to survivie (rimworld but way simpler) an interseting simulation (sim ant) than complicated base builder (but I do want building a well-protected, productive base to be important to survival).
The one exception is I want there to be multiple races that play very differently and interact in interesting ways.
This will be almost impossible to balance, but I'm hoping to make it casual enough that it doesn't super matter.

## Features
- Grow a colony of gnomes and other fantasy races and survive the seasons and goblin raids.
- Procedual world generation
- Simplified and visual-based whenever possible. All features should have trade-offs with no optimal solution. 
- Performant and highly-scalable to large maps and large numbers of gnomes
- Highly abstracted and modable
- Eventually multiplayer co-op or vs


## Gnomoria comparisons:

### Added features
- More easily modable in a scripting language rather than XML
- scales to larger numbers
- Visual display of stats instead of just a number
- Simulate enemy of friendly factions
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

### Undecided:
- Mining vs infinite mining workshops? how well does this work on 2d
- Enemies in the mines... Would be cool to mine too greedily and too deep
- Combat: Could be simplified, could keep it similar
- Stockpiles: Simpler would be nice but how would that be done?
- Professions at all? is a lot of work to do this...
