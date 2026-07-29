//! Adversarial tests for issue #811: idempotency and all-or-nothing revert
//! guarantee for `confirm_trial_offer`.
//!
//! These tests prove that a `ProgressCallFailed` error reverts the entire
//! transaction (no partial state committed) and that the new idempotency
//! nonce mechanism makes retries safe.

use scoutchain_scout_access::{ScoutAccessContract, ScoutAccessContractClient};
use scoutchain_progress::{ProgressContract, ProgressContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

#[test]
fn test_confirm_trial_offer_progress_call_failed_reverts_all_state() {
    let env = Env::default();
    env.mock_all_auths();

    let scout_access_id = env.register(ScoutAccessContract, ());
    let scout_access_client = ScoutAccessContractClient::new(&env, &scout_access_id);

    let progress_id = env.register(ProgressContract, ());
    let progress_client = ProgressContractClient::new(&env, &progress_id);

    let admin = Address::generate(&env);
    let scout = Address::generate(&env);
    let player = Address::generate(&env);

    let xlm = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let fees = scoutchain_scout_access::FeeConfig {
        contact_fee_stroops: 100_000,
        basic_sub_stroops: 1_000_000,
        pro_sub_stroops: 3_000_000,
        elite_sub_stroops: 10_000_000,
        sub_duration_secs: 2_592_000,
        pro_contact_limit: 10,
        trial_offer_escrow_stroops: 500_000,
        trial_offer_expiry_secs: 7_200,
    };

    scout_access_client.initialize(&admin, &xlm, &fees);
    scout_access_client.set_progress_contract(&progress_id);

    progress_client.initialize(&admin);

    // Scout logs a trial offer
    let _index = scout_access_client.log_trial_offer(
        &scout,
        &player,
        &String::from_str(&env, "trial details hash"),
    );

    // Player confirms — progress contract is initialized but not wired to
    // registration, so advance_level will fail. This simulates a misconfigured
    // deployment.
    let result = scout_access_client.try_confirm_trial_offer(
        &player,
        &1u64,
        &0u32,
    );

    // The call should fail with ProgressCallFailed
    assert!(result.is_err(), "confirm_trial_offer should fail when progress.advance_level fails");

    // Verify escrow was NOT cleaned up (transaction reverted)
    let escrow = env.storage().persistent().get::<
        scoutchain_scout_access::DataKey,
        scoutchain_scout_access::TrialEscrow,
    >(&scoutchain_scout_access::DataKey::TrialEscrow(1u64, 0u32));
    assert!(escrow.is_some(), "Escrow must still exist when transaction reverts on ProgressCallFailed");
}

#[test]
fn test_confirm_trial_offer_idempotency_nonce_prevents_replay() {
    let env = Env::default();
    env.mock_all_auths();

    let scout_access_id = env.register(ScoutAccessContract, ());
    let scout_access_client = ScoutAccessContractClient::new(&env, &scout_access_id);

    let admin = Address::generate(&env);
    let scout = Address::generate(&env);
    let player = Address::generate(&env);

    let xlm = env.register_stellar_asset_contract_v2(admin.clone()).address();
    let fees = scoutchain_scout_access::FeeConfig {
        contact_fee_stroops: 100_000,
        basic_sub_stroops: 1_000_000,
        pro_sub_stroops: 3_000_000,
        elite_sub_stroops: 10_000_000,
        sub_duration_secs: 2_592_000,
        pro_contact_limit: 10,
        trial_offer_escrow_stroops: 500_000,
        trial_offer_expiry_secs: 7_200,
    };

    scout_access_client.initialize(&admin, &xlm, &fees);

    // Scout logs a trial offer
    let _index = scout_access_client.log_trial_offer(
        &scout,
        &player,
        &String::from_str(&env, "trial details hash"),
    );

    // First confirmation with a nonce should succeed
    let nonce = String::from_str(&env, "confirm-nonce-1");
    let result = scout_access_client.try_confirm_trial_offer(
        &player,
        &1u64,
        &0u32,
        &Some(nonce.clone()),
    );
    assert!(result.is_ok(), "First confirm_trial_offer with nonce should succeed");

    // Second confirmation with the same nonce should return Ok(()) idempotently
    let result2 = scout_access_client.try_confirm_trial_offer(
        &player,
        &1u64,
        &0u32,
        &Some(nonce),
    );
    assert!(result2.is_ok(), "Retry with same nonce should succeed idempotently");
}
