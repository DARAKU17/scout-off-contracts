//! Tests for the timelocked propose-then-activate fee config mechanism (#807).
//!
//! Covers:
//! - Proposing fee increases (delayed activation)
//! - Proposing fee decreases (immediate activation)
//! - Activating pending proposals after the delay
//! - Rejecting premature activations
//! - Verifying that business logic uses the active config, never the pending one
//! - Handling overlapping proposals

use scoutchain_scout_access::{
    FeeConfig, ScoutAccessContract, ScoutAccessContractClient, SubscriptionTier,
};
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token::StellarAssetClient,
    Address, Env, IntoVal, Symbol,
};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn default_fees() -> FeeConfig {
    FeeConfig {
        contact_fee_stroops: 100_000,
        basic_sub_stroops: 1_000_000,
        pro_sub_stroops: 3_000_000,
        elite_sub_stroops: 7_000_000,
        sub_duration_secs: 30 * 24 * 60 * 60,
        pro_contact_limit: 10,
        trial_offer_escrow_stroops: 500_000,
        trial_offer_expiry_secs: 3_600,
    }
}

struct Harness {
    env: Env,
    admin: Address,
    scout: Address,
    xlm: Address,
    contract_id: Address,
    client: ScoutAccessContractClient<'static>,
}

fn setup() -> Harness {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let admin = Address::generate(&env);
    let scout = Address::generate(&env);

    // Create XLM token
    let xlm = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    // Mint XLM to scout for fee payments
    let token_client = StellarAssetClient::new(&env, &xlm);
    token_client.mint(&scout, &1_000_000_000);

    // Deploy and initialize scout_access
    let contract_id = env.register_contract(None, ScoutAccessContract);
    let client = ScoutAccessContractClient::new(&env, &contract_id);
    client.initialize(&admin, &xlm, &default_fees());

    Harness {
        env,
        admin,
        scout,
        xlm,
        contract_id,
        client,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Proposing fee increases
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_propose_fee_config_with_increase() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase from 7M to 10M

    // Propose the increase
    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Verify the active config hasn't changed (still the old fees)
    let active_config = h.client.get_fee_config();
    assert_eq!(active_config.elite_sub_stroops, 7_000_000);
}

#[test]
fn test_propose_fee_config_with_decrease() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 5_000_000; // Decrease from 7M to 5M

    // Propose the decrease
    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Verify the config was immediately activated
    let active_config = h.client.get_fee_config();
    assert_eq!(active_config.elite_sub_stroops, 5_000_000);
}

#[test]
fn test_propose_fee_config_with_mixed_change() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase
    new_config.basic_sub_stroops = 500_000; // Decrease

    // Propose the mixed change
    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Since there's an increase, it should be pending (not activated)
    let active_config = h.client.get_fee_config();
    assert_eq!(active_config.elite_sub_stroops, 7_000_000); // Still old value
    assert_eq!(active_config.basic_sub_stroops, 1_000_000); // Still old value
}

#[test]
fn test_propose_fee_config_invalid_validation() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.basic_sub_stroops = 0; // Invalid: zero fee

    // Propose should reject due to validation
    let result = h.client.try_propose_fee_config(&new_config);
    assert!(result.is_err());
}

#[test]
fn test_propose_fee_config_requires_admin() {
    let h = setup();
    let non_admin = Address::generate(&h.env);
    h.env.mock_all_auths_allow_address(vec![non_admin.clone()]);

    let new_config = default_fees();

    // Propose should reject due to missing admin auth
    let result = h
        .client
        .with_address(&non_admin)
        .try_propose_fee_config(&new_config);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Activating pending proposals
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_activate_fee_config_after_delay() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase

    // Propose the increase at timestamp 1_000_000
    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Verify still not active
    let active = h.client.get_fee_config();
    assert_eq!(active.elite_sub_stroops, 7_000_000);

    // Advance time by 7 days + 1 second
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) + 1;
    });

    // Activate the proposal
    h.client
        .activate_fee_config()
        .expect("activate_fee_config should succeed");

    // Verify it's now active
    let active = h.client.get_fee_config();
    assert_eq!(active.elite_sub_stroops, 10_000_000);
}

#[test]
fn test_activate_fee_config_before_delay_fails() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Try to activate immediately (no time advanced)
    let result = h.client.try_activate_fee_config();
    assert!(result.is_err());
}

#[test]
fn test_activate_fee_config_almost_but_not_quite_ready() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000;

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Advance time by 7 days - 1 second
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) - 1;
    });

    // Should still fail
    let result = h.client.try_activate_fee_config();
    assert!(result.is_err());
}

#[test]
fn test_activate_fee_config_with_no_pending() {
    let h = setup();

    // Try to activate when no proposal exists
    let result = h.client.try_activate_fee_config();
    assert!(result.is_err());
}

#[test]
fn test_activate_fee_config_requires_admin() {
    let h = setup();
    let non_admin = Address::generate(&h.env);

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000;

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Advance time
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) + 1;
    });

    // Try to activate as non-admin
    h.env.mock_all_auths_allow_address(vec![non_admin.clone()]);
    let result = h
        .client
        .with_address(&non_admin)
        .try_activate_fee_config();
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Subscribe uses active config during proposal window
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_subscribe_during_proposal_window_uses_old_fee() {
    let h = setup();

    // Propose an increase
    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase from 7M to 10M

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Subscribe while proposal is pending
    h.client
        .subscribe(&h.scout, &SubscriptionTier::Elite)
        .expect("subscribe should succeed");

    // The scout should have been charged the OLD elite_sub_stroops (7M)
    let accumulated = h.client.get_accumulated_fees();
    assert_eq!(accumulated, 7_000_000);
}

#[test]
fn test_subscribe_after_activation_uses_new_fee() {
    let h = setup();

    // Propose an increase
    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000;

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Advance time by 7 days + 1 second
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) + 1;
    });

    // Activate
    h.client
        .activate_fee_config()
        .expect("activate_fee_config should succeed");

    // Subscribe after activation
    h.client
        .subscribe(&h.scout, &SubscriptionTier::Elite)
        .expect("subscribe should succeed");

    // The scout should be charged the NEW elite_sub_stroops (10M)
    let accumulated = h.client.get_accumulated_fees();
    assert_eq!(accumulated, 10_000_000);
}

#[test]
fn test_pay_to_contact_during_proposal_window_uses_old_fee() {
    let h = setup();

    // First subscribe the scout
    h.client
        .subscribe(&h.scout, &SubscriptionTier::Pro)
        .expect("subscribe should succeed");

    // Contact a player to establish eligibility
    let player_id = 1u64;
    h.client
        .pay_to_contact(&h.scout, &player_id)
        .expect("pay_to_contact should succeed");

    let accumulated_before_proposal = h.client.get_accumulated_fees();

    // Propose a fee increase
    let mut new_config = default_fees();
    new_config.contact_fee_stroops = 500_000; // Increase from 100k

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    // Contact another player while proposal is pending
    let player_id_2 = 2u64;
    h.client
        .pay_to_contact(&h.scout, &player_id_2)
        .expect("pay_to_contact should succeed");

    // The second contact should have been charged the OLD contact_fee_stroops (100k)
    let accumulated_after = h.client.get_accumulated_fees();
    assert_eq!(accumulated_after - accumulated_before_proposal, 100_000);
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Overlapping proposals
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_propose_fee_config_with_existing_pending_proposal() {
    let h = setup();

    let mut config1 = default_fees();
    config1.elite_sub_stroops = 10_000_000;

    let mut config2 = default_fees();
    config2.elite_sub_stroops = 15_000_000;

    // Propose the first increase
    h.client
        .propose_fee_config(&config1)
        .expect("first proposal should succeed");

    // Try to propose a second increase while first is pending
    let result = h.client.try_propose_fee_config(&config2);
    assert!(result.is_err()); // Should reject due to existing pending proposal
}

#[test]
fn test_propose_after_activate_allows_new_proposal() {
    let h = setup();

    let mut config1 = default_fees();
    config1.elite_sub_stroops = 10_000_000;

    h.client
        .propose_fee_config(&config1)
        .expect("first proposal should succeed");

    // Advance time and activate
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) + 1;
    });

    h.client
        .activate_fee_config()
        .expect("activate should succeed");

    // Now propose a new config
    let mut config2 = default_fees();
    config2.elite_sub_stroops = 15_000_000;

    h.client
        .propose_fee_config(&config2)
        .expect("second proposal should succeed after first is activated");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: update_fee_config's delay-bypass is distinguishable in the event
// stream from activate_fee_config's delay-respecting activation (#1055)
// ─────────────────────────────────────────────────────────────────────────────

/// `update_fee_config` bypasses the 7-day propose/activate delay by design
/// (see docs/FEE_CONFIG_PROPOSAL_DESIGN.md, "Coexist"). It must flag that
/// bypass in the event stream: alongside the existing `fee_config_updated`
/// event, it must also emit `fee_config_delay_bypassed` — an additive event
/// that leaves `fee_config_updated`'s own topics/data unchanged for existing
/// consumers.
#[test]
fn test_update_fee_config_emits_delay_bypassed_event() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 9_000_000;

    h.client
        .update_fee_config(&new_config)
        .expect("update_fee_config should succeed");

    let events = h.env.events().all().filter_by_contract(&h.contract_id);

    let has_topic = |name: &str| {
        events.iter().any(|(_, topics, _)| {
            topics
                .get(0)
                .map(|t| t == Symbol::new(&h.env, name).into_val(&h.env))
                .unwrap_or(false)
        })
    };

    assert!(
        has_topic("fee_config_updated"),
        "update_fee_config must still emit fee_config_updated"
    );
    assert!(
        has_topic("fee_config_delay_bypassed"),
        "update_fee_config must additionally emit fee_config_delay_bypassed"
    );
}

/// `activate_fee_config`, by contrast, respects the full 7-day delay and must
/// emit only `fee_config_updated` — never `fee_config_delay_bypassed`. This
/// is the actual distinguishing signal an indexer/auditor relies on: a
/// `fee_config_updated` event with no accompanying `fee_config_delay_bypassed`
/// (and no accompanying `fee_config_proposed`, which would instead indicate
/// propose_fee_config's own immediate-decrease shortcut) means the change
/// went through the full delay.
#[test]
fn test_activate_fee_config_does_not_emit_delay_bypassed_event() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 10_000_000; // Increase — requires the delay

    h.client
        .propose_fee_config(&new_config)
        .expect("propose_fee_config should succeed");

    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (7 * 24 * 60 * 60) + 1;
    });

    h.client
        .activate_fee_config()
        .expect("activate_fee_config should succeed");

    // Only the activate_fee_config invocation's own events are relevant here;
    // `events().all()` reflects the most recent invocation.
    let events = h.env.events().all().filter_by_contract(&h.contract_id);

    let has_topic = |name: &str| {
        events.iter().any(|(_, topics, _)| {
            topics
                .get(0)
                .map(|t| t == Symbol::new(&h.env, name).into_val(&h.env))
                .unwrap_or(false)
        })
    };

    assert!(
        !has_topic("fee_config_delay_bypassed"),
        "activate_fee_config must never emit fee_config_delay_bypassed"
    );
    assert!(
        has_topic("fee_config_updated"),
        "activate_fee_config must still emit fee_config_updated"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Backwards compatibility with update_fee_config
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_update_fee_config_still_works() {
    let h = setup();

    let mut new_config = default_fees();
    new_config.elite_sub_stroops = 12_000_000; // Arbitrary new value

    // Old update_fee_config should still work
    h.client
        .update_fee_config(&new_config)
        .expect("update_fee_config should still work");

    // Should be immediately active
    let active = h.client.get_fee_config();
    assert_eq!(active.elite_sub_stroops, 12_000_000);
}

#[test]
fn test_update_fee_config_and_propose_coexist() {
    let h = setup();

    // Use old update_fee_config
    let mut config1 = default_fees();
    config1.elite_sub_stroops = 9_000_000;

    h.client
        .update_fee_config(&config1)
        .expect("update_fee_config should work");

    // Verify it's active
    let active = h.client.get_fee_config();
    assert_eq!(active.elite_sub_stroops, 9_000_000);

    // Now use new propose_fee_config to propose an increase
    let mut config2 = default_fees();
    config2.elite_sub_stroops = 12_000_000;

    h.client
        .propose_fee_config(&config2)
        .expect("propose_fee_config should work after update_fee_config");

    // Old config should still be active
    let active = h.client.get_fee_config();
    assert_eq!(active.elite_sub_stroops, 9_000_000);
}
