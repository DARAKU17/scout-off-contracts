use soroban_sdk::{Address, Env, String, Symbol};

pub const MILESTONE_APPROVED: &str = "milestone_approved";
pub const VALIDATOR_REGISTERED: &str = "validator_registered";
pub const VALIDATOR_REVOKED: &str = "validator_revoked";
pub const VALIDATOR_REVOKED_FOR_CAUSE: &str = "validator_revoked_for_cause";
pub const CONTRACT_PAUSED: &str = "contract_paused";
pub const CONTRACT_UNPAUSED: &str = "contract_unpaused";
pub const APPROVE_MILESTONE_PAUSED: &str = "approve_milestone_paused";
pub const APPROVE_MILESTONE_UNPAUSED: &str = "approve_milestone_unpaused";
pub const CONTRACT_INITIALIZED: &str = "contract_initialized";
pub const PROGRESS_CONTRACT_UPDATED: &str = "progress_contract_updated";
pub const DISPUTE_RESOLVED: &str = "dispute_resolved";
pub const ADMIN_TRANSFER_PROPOSED: &str = "admin_transfer_proposed";
pub const ADMIN_TRANSFERRED: &str = "admin_transferred";
pub const ISSUER_REGISTERED: &str = "issuer_registered";
pub const ISSUER_REVOKED: &str = "issuer_revoked";

/// topics: (event_name, old_admin)  data: new_admin
pub fn admin_transfer_proposed(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (
            Symbol::new(env, ADMIN_TRANSFER_PROPOSED),
            old_admin.clone(),
        ),
        new_admin.clone(),
    );
}

/// topics: (event_name, old_admin)  data: new_admin
pub fn admin_transferred(env: &Env, old_admin: &Address, new_admin: &Address) {
    env.events().publish(
        (Symbol::new(env, ADMIN_TRANSFERRED), old_admin.clone()),
        new_admin.clone(),
    );
}

/// topics: (event_name, validator)  data: (player_id, description, evidence_hash)
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
        ),
        (player_id, milestone_index, description.clone(), evidence_hash.clone()),
    );
}

/// topics: (event_name, wallet)  data: credentials
pub fn validator_registered(env: &Env, wallet: &Address, credentials: &String) {
    env.events().publish(
        (Symbol::new(env, "validator_registered"), wallet.clone()),
        credentials.clone(),
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
