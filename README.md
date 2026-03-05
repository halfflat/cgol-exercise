Task for our internal learning rust group is to implement Conway's Game of Life, loosely along the lines of the description given in the
[Curtin HPC course OpenMP assignment](https://pawsey.atlassian.net/wiki/download/attachments/51923133/HPC_Course_OpenMP_Assignment.pdf?api=v2).

Initial effort produces terminal 'graphics' (Unicode block characters) from the evolution of a hard-coded initial state.

Further goals:

* Run-time configuration through command line argument parsing.
* Read/write state, and then perhaps checkpointing.
* Configurable display options.
* Benchmarking mode.
* Speed optimation though multithreading and vectorization/other low level optimisations.
* Distributed parallelism.
