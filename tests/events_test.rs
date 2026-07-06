use scoutchain_verification::events::{validator_registered, VALIDATOR_REGISTERED};
use soroban_sdk::{testutils::Events, vec, Env, IntoVal, String, Symbol};

#[test]
fn test_validator_registered_emits_wallet_and_credentials() {
    let env = Env::default();
    let wallet = env.register_contract(None, MockContract);
    let credentials = String::from_str(&env, "UEFA Pro License — Coach Level A");

    validator_registered(&env, wallet.clone(), credentials.clone());

    let events = env.events().all();

    assert_eq!(events.len(), 1);

    let event = events.get(0).unwrap();

    let expected_topics = vec![
        &env,
        Symbol::new(&env, VALIDATOR_REGISTERED).into_val(&env),
        wallet.clone().into_val(&env),
    ];
    assert_eq!(event.0, expected_topics);

    let expected_data = (wallet.clone(), credentials.clone()).into_val(&env);
    assert_eq!(event.1, expected_data);
}

#[test]
fn test_validator_registered_credentials_persisted_correctly() {
    let env = Env::default();
    let wallet = env.register_contract(None, MockContract);
    let credentials = String::from_str(&env, "FIFA Certified Academy Director");

    validator_registered(&env, wallet.clone(), credentials.clone());

    let events = env.events().all();
    let event = events.get(0).unwrap();

    let data: (soroban_sdk::Address, soroban_sdk::String) =
        event.1.into_val(&env);

    assert_eq!(data.0, wallet);
    assert_eq!(data.1, credentials);
}

#[test]
fn test_validator_registered_with_empty_credentials() {
    let env = Env::default();
    let wallet = env.register_contract(None, MockContract);
    let credentials = String::from_str(&env, "");

    validator_registered(&env, wallet.clone(), credentials.clone());

    let events = env.events().all();
    let event = events.get(0).unwrap();

    let data: (soroban_sdk::Address, soroban_sdk::String) =
        event.1.into_val(&env);

    assert_eq!(data.0, wallet);
    assert_eq!(data.1, String::from_str(&env, ""));
}

#[test]
fn test_validator_registered_with_long_credentials() {
    let env = Env::default();
    let wallet = env.register_contract(None, MockContract);
    let long_cred = String::from_str(
        &env,
        "UEFA Pro License — Coach Level A — Specialization: Youth Development — \
         Certified: 2024-01-15 — Institution: Royal Spanish Football Federation — \
         License ID: ESP-2024-001337",
    );

    validator_registered(&env, wallet.clone(), long_cred.clone());

    let events = env.events().all();
    let event = events.get(0).unwrap();

    let data: (soroban_sdk::Address, soroban_sdk::String) =
        event.1.into_val(&env);

    assert_eq!(data.0, wallet);
    assert_eq!(data.1, long_cred);
}
