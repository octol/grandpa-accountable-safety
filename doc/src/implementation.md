# Implementation

The natural place to run the accountable safety protocol is on-chain. Only voters participate, since
they are guaranteed to be able to respond to all requests. Although anyone listening to the grandpa
protocol should in principle be able to take part.

## Comparison with proof-of-concept

In the proof-of-concept implementation we have the methods

```rust
fn start(
	block_not_included,
	Round_for_block_not_included,
	commit_for_block_not_included
)

// Ask the question why the estimate the previous round didn't include the earlier block
fn start_query_round(round, voters) -> Query

fn add_response(round, voter, query_response) -> Option<NextQuery>

// Ask what prevotes the voters know about
fn start_prevote_query(round, voters) -> PrevoteQuery

fn add_prevote_response(round, voter, query_response) -> Option<NextQuery>

fn equivocations_detected() -> Vec<EquivocationDetected>
```
where
```rust
struct Query {
	pub round: RoundNumber,
	pub receivers: Vec<VoterId>,
	pub block_not_included: BlockNumber,
}

enum NextQuery {
	AskAboutRound(Query),
	PrevotesForRound(PrevoteQuery),
}
```

### Input

Like the equivocation reporting API for GRANDPA, the API here would use unsigned extrinsics. To
avoid spam we do like with equivocation reporting that only block authors can submit. The
alternative to using unsigned extrinsics would be to use signed extrinsics, but this would mean the
need to maintain funded accounts. This might deter reporting issues.

```rust
sp_api::decl_runtime_apis! {
	pub trait GrandpaApi {
		// Initiate the accountable safety protocol. We can have multiple concurrently
		// running sessions so we need an instance tag to separate them.
		// Note: there is an assumption here that `commit_for_new_block` is for a later
		// round than the other
		fn start_accountable_safety_protocol(
			commit_for_new_block,
			commit_for_block_not_included,
		);

		// Ask the question why the estimate the previous round didn't
		// include the earlier block.
		fn start_query_round(round, voters, instance);

		// Each voter that are recipients for the queries add their responses.
		fn add_response(round, voter, query_response, instance);

		// Ask what prevotes the voters know about
		fn start_prevote_query(round, voters, instance);

		fn add_prevote_response(round, voter, query_response, instance);
	}
}
```

The methods `start_query_round` and `start_prevote_query` would be called to progress between
rounds.

### Output

Nodes will track the state of the accountable safety protocol by calling into the runtime when
importing blocks.

```rust
sp_api::decl_runtime_apis! {
	pub trait GrandpaApi {
		// Currently running accountable safety instances.
		fn active_accountable_safety_instances() -> Vec<AccountableSafetyId>;

		// Get the state of a running accountable instance
		fn acountable_safety_state(instance_id) -> Option<AccountableSafety>;
	}
}
```

In particular, they would need to keep track of if their response is needed. If requested to do so,
nodes would then log their responses using `add_response` and `add_prevote_response`.

*Note:* Instead of having to call into the runtime when importing blocks, an alternative would be to
use Digests.

## Storage

The equivalent of `AccountableSafety` struct will be stored on-chain. In the proof-of-concept this
is

```rust
struct AccountableSafety {
	block_not_included: BlockNumber,
	round_for_block_not_included: RoundNumber,
	commit_for_block_not_included: Commit,
	querying_rounds: BTreeMap<RoundNumber, QueryState>,
	prevote_queries: BTreeMap<RoundNumber, QueryState>,
}

struct QueryState {
	round: RoundNumber,
	voters: Vec<VoterId>,
	responses: BTreeMap<VoterId, QueryResponse>,
	equivocations: Vec<EquivocationDetected>,
}

enum QueryResponse {
	Prevotes(Vec<Prevote>),
	Precommits(Vec<Precommit>),
}
```

For a validator set of 1000 validators, how much would we then store?
