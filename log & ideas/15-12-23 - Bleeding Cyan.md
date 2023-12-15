*"To them, size and power are equivalent. It is odd to see a prisoner believe a bigger cage will set them free."*

# The Games Foxes Play
*([complete source code](https://github.com/Oneirical/rust_tgfp) | [view all previous posts](https://github.com/Oneirical/The-Games-Foxes-Play/tree/main/design/Development%20Logs))*

Maximum productivity.

[Video showcasing the Soul Wheel, Epsilon and some dashing around](https://yewtu.be/embed/yhfpTkU6osk?).

[Video showcasing the message log (it only prints random placeholder messages currently) and the Hypnotic Well (with a small graphical glitch)](https://yewtu.be/embed/Uw5KMwgKH0U).

1. "Axioms" have went through a lot. Starting off as generic "press button and pre-coded effect happens", they ramped up in complexity to hilarious heights, to the point where it was possible to craft spells such as "when taking a step, across the entire map, teleport every creature in random places, then cast all of their spells in random directions" or other such ridiculous combinations. Not only is this completely OP, it is also very hard to understand, especially for those who have not touched coding a lot. 

In this new Rust version, I have returned them under a simplified version, with only:
* A Form, dictating where the effect happens.
* A Function, dictating what the effect is.

For example, "Self-Dash" makes you dash around, but "Beam-Dash" forces a creature hit by the beam to dash, basically acting as a knockback effect. I hope that, with varied Forms and Functions, I can strike a good middlepoint between creativity and simplicity. There is still room for crazy stuff down the line.

2. The Soul Wheel is back, now with extra crispy animations. It always offers 4 Soul casts of Vile, Ordered, Feral, Saintly or Serene type, each one linked to a Form-Function combo. For example, you can set Saintly to be your dash and Vile to be your attack Axiom. It progressively empties a "draw pile" until it "reshuffles" everything back.
    * Serene Souls are infectious, and if a creature has at least one, when it "reshuffles", a random soul will be converted to Serene type. This means that casting a lot of Axioms will eventually turn your Wheel completely cyan.
    * Some Functions can "deal damage" - there is no traditional health in TGFP, but it does steal a number of Souls from the victim, disabling them into a stunned husk once all their Souls have been absorbed.
        * Yes, this means that Serene Souls can spread to creatures which do not have them like a disease. If a room is full of creatures fighting each other and only one has a single Serene Soul, given enough time, all Souls in the room will turn Serene.
    * In order to improve the "decision-to-keypress ratio", as DCSS devs would put it, it refills itself automatically - meaning that the complete list of controls required to play my game is now composed exactly of "WASD1234" (rebindable in the future).

3. The message log is back, with smooth and graceful scrolling. It supports coloured text with [cyan]tags[/cyan] in old school forum style - a feature that was possibly *the* most horrendously gory piece of code in my JavaScript version. Glad to see it be much simpler in Rust/Bevy.

4. Epsilon the robotic snake is back to some extent. Like every creature currently, it has no AI beyond moving in random directions, but its signature multi-tile slithering is pleasing to watch.

5. I tried to bring the Hypnotic Wells, probably the coolest thing about the JavaScript edition of my game. However, it is much harder to pull off now, because I chose to zoom out the player's view a little bit, resulting in miserably minuscule "previews" of the next level. It probably gets even worse with screen resolutions smaller than 1920x1080. Still, I got it to work, though I might end up dropping them entirely in favour of something else.

6. I don't understand scaling and resolutions at all. How do people get this stuff right? After frustrating amounts of cyber-butchery, I can only get 1920x1080 and 960x540 to render without ugly artifacts. Meanwhile, it feels as if every single game I ever played clamps nicely to my screen no matter what, while mine is constantly too big or too small if I ask some friends to test it. How do you scale a 16x16 sprite to 24x24?? How does DCSS manage to scale its tiles so nicely no matter what size you give to your browser window???

More to come soon, I hope. I have sane and insane ideas bubbling about in equal parts, and I intend to separate the wheat from the chaff.