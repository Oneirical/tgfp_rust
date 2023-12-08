*"Its bindings are not forged of steel, iron or even rope - but rather its own dramatically lengthy tongue, pulsating with strength as the creature attempts to suffocate itself."*

# The Games Foxes Play
*([complete source code](https://github.com/Oneirical/rust_tgfp) | [view all previous posts](https://github.com/Oneirical/The-Games-Foxes-Play/tree/main/design/Development%20Logs))*

Hello. How long has it been? 3 weeks? I've been busy with the student's plight that comes with the end of a semester, but still snuck in some progress once in a while. The features showcased today are therefore the product of stray and scattered hours.

It is time to dial down the dreams and get working on real work that produces real results. I showed my genetic algorithm experiment last time I posted here, but I also did another experiment with peer-to-peer multiplayer with rollback in Rust right afterwards. 

Useful for my game? Very likely not. Interesting? Absolutely, I am glad that I understand better how networking technology in multiplayer games works now. 

But the time for messing around has come to an end.

[Behold.]()

I have redone my entire basic game logic and UI in Rust and the Bevy engine (it used to be in pure JavaScript with the PIXI.js library). I'm a big fan of the minimap and the sidebars, but something seems off about the sliding animation when I move around. I will figure this out later.

Pros:

* The performance has received unfathomable improvements.
* I no longer have to bang my head against the brick wall that is scaling the screen to every possible resolution.
* The Entity Component System structure is amazing, I never want to do an object-oriented game ever again.
* I just write things and they work. I still have not used the debugger once.

Cons:

* UI support is quite sad. Don't tell anyone, but the "UI" in my game is actually just more game entities, except they follow the player around as it moves.
* I have a feeling implementing mouse support will be a terrible experience.

As for the game's design - I have once again drawn out my butcher's knife and have been cutting out the chaff. I've been so obsessed with being creative - to the point where I forgot to be reasonable.

As an example, even **the controls** had been turned into a game mechanic. You literally started with a spell which was composed of the editable blocks "When pressing 'W', on the tile north of the caster, move towards the targeted tile, a turn passes'". Let's list the problems with that:

* Removing the "a turn passes" component causes the player to freeze time forever, which is unfathomably OP
* Removing any of the other components basically induces a softlock
* Rebinding keys, a fundamental part of game design, was tied with a game mechanic (can't just do it in a quick menu, you need to use controls to rebind the controls)
* Quality-of-life like "click on a tile to move towards it" like in most modern roguelikes was simply impossible

I am dialing it down, keeping the spirit of "build your own abilities" but in such a way that can coexist with the accessibility standards of modern gaming. I have a rather tight concept - a spin on *the oldest ideas I ever had for this game*, which I'll talk about when it is actually getting implemented.