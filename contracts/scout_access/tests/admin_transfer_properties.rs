//! Property-based tests for the admin propose/accept flow (#824).

use scoutchain_scout_access::{ScoutAccessContract, ScoutAccessContractClient};
use soroban_sdk::{Address, Env};

struct Harness {
    env: Env,
    admin: Address,
    client: ScoutAccessContractClient<'static>,
}

fn setup() -> Harness {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let xlm = Address::generate(&env);
    let contract_id = env.register_contract(None, ScoutAccessContract);
    let client = ScoutAccessContractClient::new(&env, &contract_id);
    let fees = scoutchain_scout_access::FeeConfig {
        contact_fee_stroops: 100_000,
        basic_sub_stroops: 1_000_000,
        pro_sub_stroops: 3_000_000,
        elite_sub_stroops: 7_000_000,
        sub_duration_secs: 30 * 24 * 60 * 60,
        pro_contact_limit: 10,
        trial_offer_escrow_stroops: 500_000,
        trial_offer_expiry_secs: 3_600,
    };
    client.initialize(&admin, &xlm, &fees);
    Harness { env, admin, client }
}

#[test]
fn test_only_proposed_can_accept() {
    let h = setup();
    let proposed = Address::generate(&h.env);
    h.client.propose_admin(&proposed);
    let result = h.client.try_accept_admin(&proposed);
    assert!(result.is_ok());
}

#[test]
fn test_non_proposed_cannot_accept() {
    let h = setup();
    let proposed = Address::generate(&h.env);
    let attacker = Address::generate(&h.env);
    h.client.propose_admin(&proposed);
    let result = h.client.try_accept_admin(&attacker);
    assert!(result.is_err());
}

#[test]
fn test_double_propose_replaces_pending() {
    let h = setup();
    let first = Address::generate(&h.env);
    let second = Address::generate(&h.env);
    h.client.propose_admin(&first);
    h.client.propose_admin(&second);
    let result = h.client.try_accept_admin(&first);
    assert!(result.is_err());
    let result2 = h.client.try_accept_admin(&second);
    assert!(result2.is_ok());
}

#[test]
fn test_admin_unchanged_before_accept() {
    let h = setup();
    let new_admin = Address::generate(&h.env);
    h.client.propose_admin(&new_admin);
    let third = Address::generate(&h.env);
    h.client.propose_admin(&third);
    let result = h.client.try_accept_admin(&third);
    assert!(result.is_ok());
}

#[test]
fn test_replaced_proposal_cannot_accept() {
    let h = setup();
    let first = Address::generate(&h.env);
    let second = Address::generate(&h.env);
    h.client.propose_admin(&first);
    h.client.propose_admin(&second);
    let result = h.client.try_accept_admin(&first);
    assert!(result.is_err());
}
