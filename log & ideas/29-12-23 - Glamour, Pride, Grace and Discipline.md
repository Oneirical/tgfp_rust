*"I have been preparing for this battle ever since I learned about your existence. That was 2 milliseconds ago. Such fond memories."*

# The Games Foxes Play
*([complete source code](https://github.com/Oneirical/rust_tgfp) | [view all previous posts](https://github.com/Oneirical/The-Games-Foxes-Play/tree/main/design/Development%20Logs))*

Some nasty virus is giving me a hard time this week. But, there are people in this very thread who work hard, again and again, despite having much worse health problems. If they can do it, so can I. Thank you for the (indirect) motivation!

A lot of "Axioms" - varied spell effects to mix and match - have been added. Since attention is a limited resource, I'll go straight to the coolest one: Possession. Basically the equivalent of Domination from Caves of Qud, it allows the caster to take control of any creature for a limited time, use all their abilities as one sees fit, and potentially force them to move into dangerous positions.

This also marks the introduction of status effects. Tracked in the sidebar, there is support for all kinds of possibilities, including the "Cardinal Virtues" - a new experimental gameplay mechanic.

Here's how it works: all creatures start at 1 stack each of Glamour, Discipline, Grace and Pride. Each one of these is incremented and decremented by complementary events:

* **Taking damage** increases Discipline, but lowers Pride.
* **Dealing damage** increases Pride, but lowers Glamour.
* **Casting Axioms** increases Glamour, but lowers Grace.
* **Moving** increases Grace, but lowers Discipline.

All Axioms scale off one or multiple of these Virtues to define their power level. For example, the aforementioned Possession lasts longer the more Glamour stacks you have. 

Another example is Coil, which scales off Discipline and deals more damage if the target is surrounded by solid entities. Or Pull, which drags targeted creatures closer with distance relative to your Grace. This paves the way to some character archetypes:

* "Trickster/debuffer" for Glamour.
* "Tank" for Discipline.
* "Agile acrobat" for Grace.
* "Blaster caster" for Pride.

The goal of this mechanic is twofold:

A) Since every instance of "damage" in the game is basically lifesteal, some battles can go on forever as two powerful foes drain each other over and over. Virtues prevent this, as they (especially Discipline) keep on getting stacked until Axioms have become so powerful that ties will be broken.

B) Certain Virtues, especially Glamour, open the way to unconventional strategies. How can one defeat foes without being allowed to deal damage? [In the case of Epsilon's challenge, it is possible to bait the snake into attaching to itself the segments scattered around the arena, then Possessing it, and moving Epsilon's body so that it becomes stuck on a wall with no way to exit. But to do this, one has to stack their Glamour first, which involves finding safe spaces to cast Axioms without getting obliterated by Epsilon's attacks.](https://yewtu.be/embed/zwRAGfCumTo?) Check out the video for a full demonstration!

I am mildly suspicious that this mechanic will lead to oneshot fiestas and cheesy mechanics - moving back and forth in a corner over and over to stack Grace, for example. Perhaps it will have to be removed, or more hopefully, it will be able to shine with a few tweaks.

My next objective is shipping the reworked Epsilon boss battle in playable form soon-ish. It has been too long since the last release.