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
- Simulate enemy or friendly factions
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
- Base builder OR RTS?? How does every game I make turn into an RTS!
- Traits/professions (We need some way to decide which tasks are prioritized...)
- Mining vs infinite mining workshops? how well does this work on 2d
- Enemies in the mines... Would be cool to mine too greedily and too deep
- Combat: Could be simplified, could keep it similar
- Stockpiles: Simpler would be nice but how would that be done?
  - Consider specific building size like in Timberborn.
  - Considering simple category/colors groups?
- Professions at all? is a lot of work to do this...
- Workshops: 
  - Set and forget or require active gnome
  - Single-tile or multi-tile

Choices:
High-level priorites: Mining vs Farming vs Woodcutting?
Building layout still b/c that's fun
Attack/defend priorities?

Allocate people FTL-style with bars?

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



### Decision history
 - Realistic Pixel (Gnomoria) vs Minimalistic Pixel (Gnorp) graphics
  - Decided on Gnorp graphics b/c it looks cuter, is more someting I can do, and goes with the minimalistic theme

 - Side-view instead of top-down
  - goes better with gnorp style we were considering
  - Goes better with building "big" and "tall" kingdoms
  - potentially simplifies a lot!
  - Easier graphically let's be honest
  - Simplifies combat a lot (just pick left or right lol)
  - Allows mining clearly and understandably
  - Downside: multiplayer is now inheirly unfair because someone is in the midde (fix: looping world!!!)
  - Downside: walls are CHEAP AF now??? (fix: unclimbable walls are VERY expensive...)


