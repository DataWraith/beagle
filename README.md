# Beagle

Beagle is a [Vindinium] bot written in Rust. The name was chosen for the HMS
Beagle, because I originally wanted to incorporate a genetic algorithm into the
bot, but that plan was scrapped quickly.

It is my first Rust project, and the code was written in a trial-and-error
fashion without up-front design, so it's definitively not a very clean code
base. In order to run it, you currently need the nightly Rust compiler.

## Inner workings

The bot is based on
[Iterative Deepening](https://chessprogramming.wikispaces.com/Iterative+Deepening)
[Best Reply Search](https://project.dke.maastrichtuniversity.nl/games/files/articles/BestReplySearch.pdf)
(PDF) with the [MTD(f)] driver (MTD(f)-α-β variant) and uses [Zobrist Hashing].

The search trawls through between 20k and 100k nodes per turn, reaching a depth
of about 10 to 13 moves (depending on map size and complexity).

The evaluation function takes into account the predicted amount of gold at the
end of the game as well as the predicted gain/loss in Elo points. That leads
nicely to cooperative behavior when more than one instance of the bot is
playing.

Due to the way pathfinding is implemented, the bot also usually can't see that
it could go to a tavern that is not the nearest one.

## TODO

* [ ] Experiment with Quiescence Search
* [X] Team play
* [X] Unmake moves instead of copying state around
* [ ] Better Move Ordering
* [X] Try a simple BFS for finding mines (might be faster)

[Vindinium]: http://vindinium.org
[MTD(f)]: https://people.csail.mit.edu/plaat/mtdf.html
[Zobrist Hashing]: https://en.wikipedia.org/wiki/Zobrist_hashing
