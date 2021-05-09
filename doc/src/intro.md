# Introduction

Accountable Safety for GRANDPA is a synchronous interactive protocol for
tracking down and proving after the fact when participants misbehave. The idea
is that even is more than 1/3 of participants misbehave and finalize
conflicting forks, they will not get away with and will get their stake
slashed.

In the [GRANDPA
paper](https://github.com/w3f/consensus/blob/master/pdf/grandpa.pdf) there is a
proof by construction for showing that if two blocks B and B' for which valid
commit messages were sent, but do not lay on the same chain, then there are at
least f + 1 Byzantine voters. The proof itself then provides the procedure for
tracking down this set of misbehaving voters.

## Definitions

We refer to the GRANDPA paper for in-depth material, it is still useful to
restate a some of the more important definitions here.

### GHOST Function

The function $g(S)$ takes the set of votes and returns the block B with the
highest block number such that S has a supermajority for B.

### Estimate

$E_{r,v}$ is voter v's estimate of what might have been finalized in round
r, given by the last block in the chain with head $g(V_{r,v})$ for which it
is possible for $C_{r,v}$ to have a supermajority.

### Completable

Denote descendent blocks by $B < B'$, where block B' is a descendent of
block B. Then if either $E_{r,v} < g(V_{r,v})$ or it is impossible for
$C_{r,v}$ to have a supermajority for any children of $g(V_{r,v})$,
then we say that v sees that round r as completable. In other words, when
$E_{r,v}$ contains everything that could have been finalized in round r.

$E_{r,v}$ having supermajority means that $E_{r,v} < g(V_{r,v})$.
