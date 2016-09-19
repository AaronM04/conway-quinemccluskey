Quine-McCluskey circuit optimization applied to Conway's Game of Life
========================

I want to process a full machine word's worth of CGOL cell states (stored as
bits) to find the next generation. The naive approach would be to use full adder
circuits to add up all the neighbors and then add logic to apply the life rules
to the center cell and the calculated neighbor sum.

I'm hoping I can reduce the number of operations by computing a 2-bit-wide sum
of three of the neighbors and then a 3-bit-wide sum of the remaining five
neighbors, and then perform Quine-McCluskey optimization to find logic to get
the next cell state.

Further reading:
https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life
https://en.wikipedia.org/wiki/Quine%E2%80%93McCluskey_algorithm
https://en.wikipedia.org/wiki/Adder_%28electronics%29#Full_adder
