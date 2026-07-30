//! Tests for issue #816: pre-authorized migration ticket protocol.
//!
//! Verifies that:
//! 1. A valid pre-authorization is successfully redeemed once
//! 2. A replay/reuse of the same authorization against a second new contract is rejected

use scoutchain_registration::{RegistrationContract, RegistrationContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::crypto::{ed25519, sha256};

fn setup() -> (Env, RegistrationContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, scoutchain_registration::RegistrationContract);
    let client = scoutchain_registration::RegistrationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn generate_keypair() -> ed25519::Keypair {
    ed25519::Keypair::generate()
}

fn sign_migration_player(
    env: &Env,
    keypair: &ed25519::Keypair,
    wallet: &Address,
    player_id: u64,
    new_contract_hint: &Address,
    nonce: u64,
) -> scoutchain_registration::MigrationAuthorization {
    use soroban_sdk::Vec;
    
    let vitals = scoutchain_registration::PlayerVitals {
        age: 20,
        position: String::from_str(env, "Forward"),
        region: String::from_str(env, "West Africa"),
        nationality: String::from_str(env, "Ghana"),
    };
    let ipfs_hashes = Vec::from_slice(env, &[String::from_str(env, "QmCID1")]);
    
    let mut hasher = sha256::Hasher::new(env);
    for b in wallet.to_bytes() {
        hasher.update(&[b]);
    }
    for b in vitals.age.to_be_bytes() {
        hasher.update(&[b]);
    }
    for b in vitals.position.as_bytes() {
        hasher.update(&[b]);
    }
    for b in vitals.region.as_bytes() {
        hasher.update(&[b]);
    }
    for b in vitals.nationality.as_bytes() {
        hasher.update(&[b]);
    }
    for hash in ipfs_hashes.iter() {
        for b in hash.as_bytes() {
            hasher.update(&[b]);
        }
    }
    for b in player_id.to_be_bytes() {
        hasher.update(&[b]);
    }
    let profile_data_hash = hasher.finalize().to_vec();
    
    let mut message = Vec::new(env);
    for b in wallet.to_bytes() {
        message.push_back(b);
    }
    message.push_back(0u8);
    for b in &profile_data_hash {
        message.push_back(*b);
    }
    for b in new_contract_hint.to_bytes() {
        message.push_back(*b);
    }
    for b in nonce.to_be_bytes() {
        message.push_back(*b);
    }
    message.push_back(0u8);
    for b in 0u64.to_be_bytes() {
        message.push_back(*b);
    }
    
    let signature = keypair.sign(&message);
    
    scoutchain_registration::MigrationAuthorization {
        wallet: wallet.clone(),
        role: scoutchain_registration::MigrationRole::Player,
        profile_data_hash,
        new_contract_hint: new_contract_hint.clone(),
        nonce,
        expires_at: 0,
        signature: signature.to_vec(),
    }
}

#[test]
fn test_valid_migration_authorization_redeemed_once() {
    let (env, client, _admin) = setup();
    
    let wallet = Address::generate(&env);
    let new_contract_hint = Address::generate(&env);
    let keypair = generate_keypair();
    
    let authorization = sign_migration_player(&env, &keypair, &wallet, 1, &new_contract_hint, 1);
    
    let vitals = scoutchain_registration::PlayerVitals {
        age: 20,
        position: String::from_str(&env, "Forward"),
        region: String::from_str(&env, "West Africa"),
        nationality: String::from_str(&env, "Ghana"),
    };
    let ipfs_hashes = Vec::from_slice(&env, &[String::from_str(&env, "QmCID1")]);
    
    let result = client.try_redeem_migration_player(
        &wallet,
        vitals,
        ipfs_hashes,
        scoutchain_shared_types::ProgressLevel::Unverified,
        1,
        1700000000,
        1700000000,
        authorization,
    );
    
    assert!(result.is_ok(), "Valid migration authorization should be redeemed");
}

#[test]
fn test_replay_same_nonce_rejected() {
    let (env, client, _admin) = setup();
    
    let wallet = Address::generate(&env);
    let new_contract_hint = Address::generate(&env);
    let keypair = generate_keypair();
    
    let authorization = sign_migration_player(&env, &keypair, &wallet, 1, &new_contract_hint, 1);
    
    let vitals = scoutchain_registration::PlayerVitals {
        age: 20,
        position: String::from_str(&env, "Forward"),
        region: String::from_str(&env, "West Africa"),
        nationality: String::from_str(&env, "Ghana"),
    };
    let ipfs_hashes = Vec::from_slice(&env, &[String::from_str(&env, "QmCID1")]);
    
    let result1 = client.try_redeem_migration_player(
        &wallet,
        vitals.clone(),
        ipfs_hashes.clone(),
        scoutchain_shared_types::ProgressLevel::Unverified,
        1,
        1700000000,
        1700000000,
        authorization.clone(),
    );
    assert!(result1.is_ok(), "First redemption should succeed");
    
    let result2 = client.try_redeem_migration_player(
        &wallet,
        vitals,
        ipfs_hashes,
        scoutchain_shared_types::ProgressLevel::Unverified,
        2,
        1700000000,
        1700000000,
        authorization,
    );
    
    assert_eq!(
        result2,
        Err(Ok(scoutchain_registration::ScoutChainError::InvalidInput)),
        "Replay with same nonce should be rejected"
    );
}
