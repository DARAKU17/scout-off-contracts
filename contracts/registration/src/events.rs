#![allow(deprecated, dead_code)]
use soroban_sdk::{Address, Env, Symbol};

pub const PLAYER_REGISTERED: &str = "player_registered";
pub const SCOUT_REGISTERED: &str = "scout_registered";
pub const PROFILE_UPDATED: &str = "profile_updated";
pub const PLAYER_DEREGISTERED: &str = "player_deregistered";
pub const PLAYER_LEVEL_SYNCED: &str = "player_level_synced";
pub const SCOUT_VERIFIED: &str = "scout_verified";

pub fn player_registered(env: &Env, player_id: u64, wallet: &Address) {
    env.events().publish(
        (Symbol::new(env, "player_registered"), wallet.clone()),
        player_id,
    );
}

pub fn scout_registered(env: &Env, scout_id: u64, wallet: &Address) {
    env.events().publish(
        (Symbol::new(env, "scout_registered"), wallet.clone()),
        scout_id,
    );
}

pub fn profile_updated(env: &Env, player_id: u64) {
    env.events()
        .publish((Symbol::new(env, "profile_updated"),), player_id);
}

pub fn player_deregistered(env: &Env, player_id: u64) {
    env.events()
        .publish((Symbol::new(env, "player_deregistered"),), player_id);
}

pub fn player_level_synced(env: &Env, player_id: u64) {
    env.events()
        .publish((Symbol::new(env, "player_level_synced"),), player_id);
}

pub fn scout_verified(env: &Env, scout_id: u64, wallet: &Address) {
    env.events().publish(
        (Symbol::new(env, "scout_verified"), wallet.clone()),
        scout_id,
    );
}
