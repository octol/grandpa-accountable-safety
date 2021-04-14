use crate::voting::RoundNumber;

// State of the accountable safety protocol
pub enum AccountableSafety {
	Inactive,
	ProbeVoters(AccountableSafetyState),
}

struct AccountableSafetyState {
	current_round: RoundNumber,
}

