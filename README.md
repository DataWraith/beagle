# Beagle

Beagle is a [Vindinium] bot written in Rust. The name was chosen for the HMS Beagle, because
I originally wanted to incorporate a genetic algorithm into the bot, but that plan was scrapped
quickly.

It is my first Rust project, and the code was written in a trial-and-error fashion without
up-front design, so it's definitively not a very clean code base. I'd go so far as to call
it sloppy. In order to run it, you currently need the nightly Rust compiler (v. 1.12).

I have embedded the bot's key out of sheer laziness, so feel free to
ruin the (not exactly great) rank it has on the server by running it yourself.

The bot currently can't cope with the larger maps due to the way pathfinding works and simply
crashes. Apropos crashing, it does that a lot, usually with a stack overflow I couldn't pin down
yet.

## Inner workings

The bot is based on [Iterative Deepening](https://chessprogramming.wikispaces.com/Iterative+Deepening)
[Best Reply Search](https://project.dke.maastrichtuniversity.nl/games/files/articles/BestReplySearch.pdf)
(PDF) with the [MTD(f)] driver and uses [Zobrist Hashing].

The search trawls through between 20k and 100k nodes per turn, reaching a depth of about 10
moves (depending on map size and complexity), but still does stupid things occasionally;
I haven't figured out why yet -- probably a bug somewhere.

I wonder how Mini-Me plays so well (also using BRS), given that it only searches for a fraction
of the time Beagle takes. Apropos Mini-Me, the evaluation function is basically the MineGold heuristic
discussed [here](https://www.reddit.com/r/vindinium/comments/2kgsx4/a_chat_with_the_creator_of_the_best_performing/)
with the addition of considering the hero's life and a penalty for standing next to opponents.

Due to the way pathfinding is implemented, the bot also usually can't see that it could go to a
tavern that is not the nearest one.

## TODO

* [ ] Experiment with Principal Variation Search vs. MTD(f)
* [ ] Experiment with Paranoid instead of Best Reply Search
* [ ] Experiment with Quiescence Search
* [ ] Team play
* [X] Unmake moves instead of copying state around
* [ ] Better Move Ordering
* [ ] Ignore opponent moves if the opponent is on the other side of the map
* [ ] Try a simple BFS instead for finding mines (might be faster)

[Vindinium]: http://vindinium.org
[MTD(f)]: https://people.csail.mit.edu/plaat/mtdf.html
[Zobrist Hashing]: https://en.wikipedia.org/wiki/Zobrist_hashing
