fn main() {
    println!("Outline of Accountable safety algorithm");

    // Step 0.
    // -------
    //
    // Detect blocks B and B' on two different branches finalized
    // Assume B' was finalized in a later round than B, r'> r.
    //
    // o-o-o-B
    //    \o-o-B'
    //
    // Step 1. start asking questions about B'
    // ---------------------------------------
    //
    // Q: Why the estimate did not include B when prevoting for B'
    // A: A set S of prevotes or a set S of precommits of the preceding round.
    //    In either case such that it is impossible for S to have a supermajority for B.
    //
    // (Repeat for each round back to round r+1.)
    //
    // Step 2. reach the round after which B was finalized
    // ---------------------------------------------------
    //
    // The reply for round r+1 will contain a set S of either prevotes or precommites
    // - If precommits: take union with precommits in commit msg for B to find equivocators.
    // - If prevotes: ask the precommitters for B.
    //
    // Step 3. instead ask the precommitters for B
    // -------------------------------------------
    //
    // Q: Ask all precommitters in the in commit msg for B, which prevotes have you seen?
    // A: A set T of prevotes with a supermajority for B.
    //    Take the union with S and find the equivocators.
}
