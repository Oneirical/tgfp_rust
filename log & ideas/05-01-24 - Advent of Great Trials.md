*The head of a gigantic mechanical snake, its blazing red eyes burning away the retinas of organics whom would dare stare too long. Its gold and chrome frills act as an attestation of the superiority of metal over muscle.*

# The Games Foxes Play
*([complete source code](https://github.com/Oneirical/rust_tgfp) | [view all previous posts](https://github.com/Oneirical/The-Games-Foxes-Play/tree/main/design/Development%20Logs))*

Happy 500th Sharing Saturday everyone!

My DCSS addiction had a small resurgence this week, with the completion of my unhinged 32k word guide. I hope that was all I needed to move onto something else. Despite this:

An "Inspect" mode with a cursor has been added. You can drag it to any creature on the screen to see their Souls, species passive, flavour text and status effects.

Epsilon has been given one such species passive. First, he no longer starts with his signature tail - and looks really funny without it. Instead, he has a Magnetize passive that makes him self-attach tail segments he comes into contact with.

Second, should he ever be adjacent to 4 solid Creatures of any type, he gains a stack of Meltdown. This represents his actuators forcing and whirring as he is stuck. Reaching 10 Meltdown will instantly defeat him, so he will naturally try to avoid this by casting his new emergency Axiom - "Exile Those Who Dared Touch" - which blinks away all surrounding **non-robotic** creatures in an attempt to free himself.

Keyword: non-robotic. The tail segments are robotic, so getting him to coil himself in a spiral would mark his instant downfall. But how to get him into such a disavantageous position?

The different Castes (think "magic schools") and their Virtues have different ideas:

* **Glamour** ("trickery"): Use the enemy's strength against themselves. Bait Epsilon into attaching lots of tail segments, then possess him, play as him and move into a spiral position, then return to your original body right before reaching critical Meltdown.
* **Discipline** ("honour"): No cowardly tactics, copy Epsilon's tail-segment magnetization, and try to suffocate him with your own tail. May the best snake win.
* **Grace** ("motion"): What was that about an emergency ability? Animate the walls to synchronize their motions with you, and suffocate Epsilon as he spams his emergency spell desperately.
* **Pride** ("disdain"): Forget about these childish games. Forcefully inject Epsilon with a Serene Soul - a terrible identity-wiping poison - and survive his attacks until he finally succumbs.

All of these are currently implemented, and just need a little polish to handle edge cases (like Discipline occasionally causing one to steal the other's tail). **Note: I am away from home right now, in 1-3 hours I'll add a video/image showcase**.

I'll have the player choose one Virtue to start off with and give them the ability to specialize or generalize as the game gets more challenges. This should get interesting.