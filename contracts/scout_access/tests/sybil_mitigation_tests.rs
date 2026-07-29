//! Tests for Sybil-resistant Pro-tier gating (#808).
//!
//! Covers:
//! - Unverified scouts cannot subscribe to Pro tier
//! - Verified scouts can subscribe to Pro tier
//! - Unverified scouts can still get Basic and Elite tiers
//! - Admin can verify scouts
//! - Registration contract wiring

use scoutchain_registration::{RegistrationContract, RegistrationContractClient};
use scoutchain_scout_access::{
    FeeConfig, ScoutAccessContract, ScoutAccessContractClient, SubscriptionTier,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
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
    scout1: Address,
    scout2: Address,
    xlm: Address,
    registration_client: RegistrationContractClient<'static>,
    scout_access_client: ScoutAccessContractClient<'static>,
}

fn setup() -> Harness {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = 1_000_000);

    let admin = Address::generate(&env);
    let scout1 = Address::generate(&env);
    let scout2 = Address::generate(&env);

    // Create XLM token
    let xlm = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();

    // Mint XLM to scouts
    let token_client = StellarAssetClient::new(&env, &xlm);
    token_client.mint(&scout1, &1_000_000_000);
    token_client.mint(&scout2, &1_000_000_000);

    // Deploy registration contract
    let reg_id = env.register_contract(None, RegistrationContract);
    let registration_client = RegistrationContractClient::new(&env, &reg_id);
    registration_client.initialize(&admin);

    // Register scouts in registration contract
    let _scout1_id = registration_client
        .register_scout(&scout1, &"North America".into())
        .expect("scout1 registration should succeed");

    let _scout2_id = registration_client
        .register_scout(&scout2, &"Europe".into())
        .expect("scout2 registration should succeed");

    // Deploy scout_access contract
    let sa_id = env.register_contract(None, ScoutAccessContract);
    let scout_access_client = ScoutAccessContractClient::new(&env, &sa_id);
    scout_access_client.initialize(&admin, &xlm, &default_fees());

    // Wire registration contract into scout_access
    scout_access_client
        .set_registration_contract(&reg_id)
        .expect("wiring should succeed");

    Harness {
        env,
        admin,
        scout1,
        scout2,
        xlm,
        registration_client,
        scout_access_client,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Pro-tier gating
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unverified_scout_cannot_subscribe_pro() {
    let h = setup();

    // scout1 is unverified by default
    let result = h
        .scout_access_client
        .try_subscribe(&h.scout1, &SubscriptionTier::Pro);
    assert!(result.is_err(), "Unverified scout should not be able to subscribe to Pro");
}

#[test]
fn test_verified_scout_can_subscribe_pro() {
    let h = setup();

    // Get scout1's ID and verify them
    let scout1_profile = h
        .registration_client
        .get_scout_by_wallet(&h.scout1)
        .expect("should find scout1");
    let scout1_id = scout1_profile.scout_id;

    h.registration_client
        .verify_scout(&scout1_id)
        .expect("verify should succeed");

    // Now scout1 should be able to subscribe to Pro
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Pro)
        .expect("verified scout should be able to subscribe to Pro");
}

#[test]
fn test_unverified_scout_can_subscribe_basic() {
    let h = setup();

    // Basic tier should work for unverified scouts
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Basic)
        .expect("unverified scout should be able to subscribe to Basic");
}

#[test]
fn test_unverified_scout_can_subscribe_elite() {
    let h = setup();

    // Elite tier should work for unverified scouts (no gating on Elite)
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Elite)
        .expect("unverified scout should be able to subscribe to Elite");
}

#[test]
fn test_multiple_scouts_independent_verification() {
    let h = setup();

    // Get scout profiles
    let scout1_profile = h
        .registration_client
        .get_scout_by_wallet(&h.scout1)
        .expect("should find scout1");
    let scout2_profile = h
        .registration_client
        .get_scout_by_wallet(&h.scout2)
        .expect("should find scout2");

    // Verify only scout1
    h.registration_client
        .verify_scout(&scout1_profile.scout_id)
        .expect("verify scout1");

    // scout1 can subscribe to Pro
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Pro)
        .expect("verified scout1 should subscribe to Pro");

    // scout2 (still unverified) cannot subscribe to Pro
    let result = h
        .scout_access_client
        .try_subscribe(&h.scout2, &SubscriptionTier::Pro);
    assert!(result.is_err(), "unverified scout2 should not subscribe to Pro");

    // scout2 can still get Elite
    h.scout_access_client
        .subscribe(&h.scout2, &SubscriptionTier::Elite)
        .expect("unverified scout2 should subscribe to Elite");
}

#[test]
fn test_verified_scout_can_renew_pro() {
    let h = setup();

    // Get scout1's ID and verify
    let scout1_profile = h
        .registration_client
        .get_scout_by_wallet(&h.scout1)
        .expect("should find scout1");

    h.registration_client
        .verify_scout(&scout1_profile.scout_id)
        .expect("verify should succeed");

    // Subscribe to Pro
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Pro)
        .expect("first subscription should succeed");

    // Advance time to expire subscription
    h.env.ledger().with_mut(|l| {
        l.timestamp = 1_000_000 + (35 * 24 * 60 * 60);
    });

    // Renew subscription (should still work since scout remains verified)
    h.scout_access_client
        .subscribe(&h.scout1, &SubscriptionTier::Pro)
        .expect("renewal should succeed for verified scout");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Registration contract wiring
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_set_registration_contract_requires_admin() {
    let h = setup();
    let non_admin = Address::generate(&h.env);
    let random_contract = Address::generate(&h.env);

    h.env
        .mock_all_auths_allow_address(vec![non_admin.clone()]);

    let result = h
        .scout_access_client
        .with_address(&non_admin)
        .try_set_registration_contract(&random_contract);
    assert!(result.is_err(), "non-admin should not be able to wire registration contract");
}

#[test]
fn test_registration_contract_graceful_degradation() {
    let h = setup();
    let scout3 = Address::generate(&h.env);

    // Mint XLM to scout3
    let token_client = StellarAssetClient::new(&h.env, &h.xlm);
    token_client.mint(&scout3, &1_000_000_000);

    // Do NOT wire registration contract; create a new scout_access instance
    let sa_id = h.env.register_contract(None, ScoutAccessContract);
    let sa_client = ScoutAccessContractClient::new(&h.env, &sa_id);
    sa_client.initialize(&h.admin, &h.xlm, &default_fees());

    // Without registration contract wired, Pro subscriptions should be allowed (graceful degradation)
    sa_client
        .subscribe(&scout3, &SubscriptionTier::Pro)
        .expect("Pro subscription should be allowed when registration contract is not wired");
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests: Sybil attack scenarios
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_multi_wallet_sybil_attempt_blocked() {
    let h = setup();

    // Attacker registers 3 scout wallets
    let attacker1 = Address::generate(&h.env);
    let attacker2 = Address::generate(&h.env);
    let attacker3 = Address::generate(&h.env);

    let token_client = StellarAssetClient::new(&h.env, &h.xlm);
    token_client.mint(&attacker1, &1_000_000_000);
    token_client.mint(&attacker2, &1_000_000_000);
    token_client.mint(&attacker3, &1_000_000_000);

    // Register all 3 in registration contract
    let _id1 = h
        .registration_client
        .register_scout(&attacker1, &"Region1".into())
        .expect("registration should succeed");
    let _id2 = h
        .registration_client
        .register_scout(&attacker2, &"Region2".into())
        .expect("registration should succeed");
    let _id3 = h
        .registration_client
        .register_scout(&attacker3, &"Region3".into())
        .expect("registration should succeed");

    // All 3 are unverified by default
    let result1 = h
        .scout_access_client
        .try_subscribe(&attacker1, &SubscriptionTier::Pro);
    let result2 = h
        .scout_access_client
        .try_subscribe(&attacker2, &SubscriptionTier::Pro);
    let result3 = h
        .scout_access_client
        .try_subscribe(&attacker3, &SubscriptionTier::Pro);

    assert!(
        result1.is_err() && result2.is_err() && result3.is_err(),
        "All unverified wallets should be blocked from Pro tier"
    );

    // Admin can verify one or more, but attacker must convince admin N times
    let attacker1_profile = h
        .registration_client
        .get_scout_by_wallet(&attacker1)
        .expect("should find");
    h.registration_client
        .verify_scout(&attacker1_profile.scout_id)
        .expect("verify");

    // Only attacker1 can now subscribe to Pro
    h.scout_access_client
        .subscribe(&attacker1, &SubscriptionTier::Pro)
        .expect("verified attacker1 should subscribe");

    // attacker2 and attacker3 still blocked
    let result2 = h
        .scout_access_client
        .try_subscribe(&attacker2, &SubscriptionTier::Pro);
    let result3 = h
        .scout_access_client
        .try_subscribe(&attacker3, &SubscriptionTier::Pro);

    assert!(
        result2.is_err() && result3.is_err(),
        "Unverified wallets should remain blocked"
    );
}

#[test]
fn test_attacker_can_pay_for_elite_instead() {
    let h = setup();

    let attacker = Address::generate(&h.env);
    let token_client = StellarAssetClient::new(&h.env, &h.xlm);
    token_client.mint(&attacker, &1_000_000_000);

    h.registration_client
        .register_scout(&attacker, &"AttackerRegion".into())
        .expect("registration should succeed");

    // Elite tier always works (no verification needed)
    h.scout_access_client
        .subscribe(&attacker, &SubscriptionTier::Elite)
        .expect("Elite should be available to any scout");

    // This demonstrates that Elite (0.7 XLM unlimited) is still cheaper than
    // 3 Pro wallets (0.9 XLM for 30 contacts), so the mitigation raises friction
    // but doesn't completely prevent an attacker who wants unlimited contacts.
}
