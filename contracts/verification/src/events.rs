use soroban_sdk::{Address, Env, String, Symbol};

pub fn milestone_approved(
    env: &Env,
    player_id: u64,
    validator: &Address,
    milestone_index: u32,
    description: &String,
    evidence_hash: &String,
) {
    env.events().publish(
        (
            Symbol::new(env, "milestone_approved"),
            validator.clone(),
            milestone_index,
        ),
        (player_id, description.clone(), evidence_hash.clone()),
    );
}

pub fn validator_registered(env: &Env, wallet: &Address) {
    env.events()
        .publish((Symbol::new(env, "validator_registered"),), wallet.clone());
}

pub fn validator_revoked(env: &Env, wallet: &Address) {
    env.events()
        .publish((Symbol::new(env, "validator_revoked"),), wallet.clone());
}

pub fn milestone_disputed(
    env: &Env,
    player_id: u64,
    milestone_index: u32,
    filed_by: &Address,
    jury_required: bool,
) {
    env.events().publish(
        (
            Symbol::new(env, "milestone_disputed"),
            player_id,
            milestone_index,
        ),
        (filed_by.clone(), jury_required),
    );
}

pub fn dispute_vote_cast(
    env: &Env,
    player_id: u64,
    milestone_index: u32,
    validator: &Address,
    upheld: bool,
) {
    env.events().publish(
        (
            Symbol::new(env, "dispute_vote_cast"),
            player_id,
            milestone_index,
        ),
        (validator.clone(), upheld),
    );
}

pub fn dispute_resolved(env: &Env, player_id: u64, milestone_index: u32, upheld: bool) {
    env.events().publish(
        (
            Symbol::new(env, "dispute_resolved"),
            player_id,
            milestone_index,
        ),
        upheld,
    );
}

pub fn dispute_tallied(
    env: &Env,
    player_id: u64,
    milestone_index: u32,
    upheld: bool,
    votes_for: u32,
    votes_against: u32,
) {
    env.events().publish(
        (
            Symbol::new(env, "dispute_tallied"),
            player_id,
            milestone_index,
        ),
        (upheld, votes_for, votes_against),
    );
}
