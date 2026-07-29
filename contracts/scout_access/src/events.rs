use crate::types::SubscriptionTier;
use soroban_sdk::{Address, Env, Symbol};

pub fn scout_subscribed(env: &Env, scout: &Address, tier: &SubscriptionTier) {
    env.events().publish(
        (Symbol::new(env, "scout_subscribed"), scout.clone()),
        tier.clone(),
    );
}

pub fn player_contacted(env: &Env, player_id: u64, scout: &Address) {
    env.events().publish(
        (Symbol::new(env, "player_contacted"), scout.clone()),
        player_id,
    );
}

pub fn evidence_access_granted(env: &Env, player_id: u64, viewer: &Address, granted_at: u64) {
    env.events().publish(
        (
            Symbol::new(env, "evidence_access_granted"),
            player_id,
            viewer.clone(),
        ),
        granted_at,
    );
}

pub fn trial_offer_logged(env: &Env, player_id: u64, scout: &Address) {
    env.events().publish(
        (Symbol::new(env, "trial_offer_logged"), scout.clone()),
        player_id,
    );
}

pub fn fees_withdrawn(env: &Env, to: &Address, amount: i128) {
    env.events()
        .publish((Symbol::new(env, "fees_withdrawn"), to.clone()), amount);
}
