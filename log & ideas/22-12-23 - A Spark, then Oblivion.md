*"Give a Soul insufficient faith, and it becomes paralyzed in apathy. Drown it in zeal, and the weight of its duty comes to immobilize it. I strive to walk the line between."*

# The Games Foxes Play
*([complete source code](https://github.com/Oneirical/rust_tgfp) | [view all previous posts](https://github.com/Oneirical/The-Games-Foxes-Play/tree/main/design/Development%20Logs))*

Not as productive as last week, but that did allow me to do some things with my free time than don't involve standing motionless in front of a monitor.

* An animation queue has been added. In the style of Rift Wizard/Cogmind, it memorizes all actions this turn, and plays quick animations for each one. Perfect for laser beams and soul draining!

* Very basic creature "AI" has been added. When it is a creature's turn to act, they will look through all of their available Soul casts. Each one is valued as being "positive" (should be cast on self/allies) or "negative" (should be cast on foes). The creature looks at how many foes it can penalize and how many allies it can boost with one cast, and if nothing is particularly interesting (calculated score does not exceed the threshold), it moves instead, hopefully putting itself in range for a better cast.

* When a creature runs out of Souls by having been drained too many times, it flips over and becomes unable to act. This is not death - death does not exist in TGFP and creatures are never removed. Breathing Souls inside a motionless husk will revive it.

[Quick demonstration.](https://yewtu.be/embed/qbINffA3cec?) There is a little offset in the soul drain animation I have yet to figure out, and the lack of proper pathfinding means creatures tend to get stuck on obstacles. Still, you can observe here Epsilon with a simple beam attack, as well as "Cosmos Worn as Robes" the moth with their beam harpoon, melee drain and occasional dash.