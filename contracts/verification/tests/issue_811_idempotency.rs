//! Adversarial tests for issue #811: idempotency and all-or-nothing revert
//! guarantee for `approve_milestone`.
//!
//! These tests prove that a `ProgressCallFailed` error reverts the entire
//! transaction (no partial state committed) and that the new idempotency
//! nonce mechanism makes retries safe.

use scoutchain_verification::{VerificationContract, VerificationContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

const VALID_CID: &str = "QmPK1s3pNYLi9ERiq3BDxKa4XosgWwFRQUydHUtz4YgpqB";

#[test]
fn test_approve_milestone_progress_call_failed_reverts_all_state() {
    let env = Env::default();
    env.mock_all_auths();

    let verification_id = env.register(VerificationContract, ());
    let verification_client = VerificationContractClient::new(&env, &verification_id);

    let progress_id = env.register(scoutchain_progress::ProgressContract, ());
    let progress_client = scoutchain_progress::ProgressContractClient::new(&env, &progress_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let player_id = 1u64;

    verification_client.initialize(&admin);
    verification_client.set_progress_contract(&progress_id);
    verification_client.register_validator(&validator, &String::from_str(&env, "UEFA B License"));

    progress_client.initialize(&admin);

    // Call approve_milestone — the progress contract is initialized but
    // has no registration contract linked, so advance_level will fail with
    // a cross-contract error. This simulates a misconfigured deployment.
    let result = verification_client.try_approve_milestone(
        &validator,
        &player_id,
        &String::from_str(&env, "hat-trick"),
        &String::from_str(&env, VALID_CID),
    );

    // The call should fail with ProgressCallFailed
    assert!(result.is_err(), "approve_milestone should fail when progress.advance_level fails");

    // Verify NO partial state was persisted:
    // 1. Milestone counter should still be 0
    let counter: u32 = env.storage()
        .persistent()
        .get(&scoutchain_verification::DataKey::MilestoneCounter(player_id))
        .unwrap_or(0);
    assert_eq!(counter, 0, "Milestone counter must not be incremented on reverted ProgressCallFailed");

    // 2. No milestone should exist
    let milestone = env.storage()
        .persistent()
        .get::<scoutchain_verification::DataKey, scoutchain_verification::Milestone>(
            &scoutchain_verification::DataKey::Milestone(player_id, 1)
        );
    assert!(milestone.is_none(), "Milestone must not be persisted on reverted ProgressCallFailed");

    // 3. Evidence hash must not be marked as used
    let evidence_used = env.storage().persistent().has(
        &scoutchain_verification::DataKey::EvidenceUsed(String::from_str(&env, VALID_CID))
    );
    assert!(!evidence_used, "Evidence hash must not be marked as used on reverted ProgressCallFailed");
}

#[test]
fn test_approve_milestone_idempotency_nonce_prevents_duplicate_on_retry() {
    let env = Env::default();
    env.mock_all_auths();

    let verification_id = env.register(VerificationContract, ());
    let verification_client = VerificationContractClient::new(&env, &verification_id);

    let admin = Address::generate(&env);
    let validator = Address::generate(&env);
    let player_id = 1u64;

    verification_client.initialize(&admin);

    // First call with a nonce should succeed
    let nonce = String::from_str(&env, "retry-token-1");
    let result = verification_client.try_approve_milestone(
        &validator,
        &player_id,
        &String::from_str(&env, "hat-trick"),
        &String::from_str(&env, VALID_CID),
        &Some(nonce.clone()),
    );
    assert!(result.is_ok(), "First approve_milestone with nonce should succeed");

    // Second call with the same nonce should return the cached index
    let result2 = verification_client.try_approve_milestone(
        &validator,
        &player_id,
        &String::from_str(&env, "different description"),
        &String::from_str(&env, "QmDifferentCID"),
        &Some(nonce),
    );
    assert!(result2.is_ok(), "Retry with same nonce should succeed idempotently");
    assert_eq!(result2.unwrap(), result.unwrap(), "Retry should return the same milestone index");
}
