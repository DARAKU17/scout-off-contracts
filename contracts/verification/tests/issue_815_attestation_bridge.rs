//! Tests for issue #815: validator-credential attestation bridge.
//!
//! Verifies that:
//! 1. A validly-signed attestation succeeds
//! 2. A tampered/invalid-signature attestation is rejected
//! 3. An untrusted issuer's attestation is rejected

use scoutchain_verification::{VerificationContract, VerificationContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_sdk::crypto::ed25519;

fn setup() -> (Env, VerificationContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register_contract(None, scoutchain_verification::VerificationContract);
    let client = scoutchain_verification::VerificationContractClient::new(&env, &id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn generate_keypair() -> (ed25519::Keypair, [u8; 32]) {
    let keypair = ed25519::Keypair::generate();
    let public_key = keypair.public().to_bytes();
    (keypair, public_key)
}

fn sign_attestation(
    env: &Env,
    keypair: &ed25519::Keypair,
    issuer_wallet: &Address,
    validator_wallet: &Address,
    credential_type: &str,
    expires_at: u64,
) -> scoutchain_verification::CredentialAttestation {
    use soroban_sdk::Vec;
    
    let mut message = Vec::new(env);
    for b in issuer_wallet.to_bytes() {
        message.push_back(b);
    }
    for b in validator_wallet.to_bytes() {
        message.push_back(b);
    }
    for b in credential_type.as_bytes() {
        message.push_back(b);
    }
    for b in expires_at.to_be_bytes() {
        message.push_back(*b);
    }
    
    let signature = keypair.sign(&message);
    
    scoutchain_verification::CredentialAttestation {
        issuer_wallet: issuer_wallet.clone(),
        validator_wallet: validator_wallet.clone(),
        credential_type: String::from_str(env, credential_type),
        expires_at,
        signature: signature.to_vec(),
    }
}

#[test]
fn test_valid_attestation_succeeds() {
    let (env, client, admin) = setup();
    
    let issuer = Address::generate(&env);
    let validator = Address::generate(&env);
    
    client.register_issuer(&issuer, &String::from_str(&env, "Test Federation"));
    
    let (keypair, _public_key) = generate_keypair();
    
    let attestation = sign_attestation(&env, &keypair, &issuer, &validator, "UEFA B License", 0);
    
    let result = client.try_register_validator_with_attestation(&validator, &attestation);
    assert!(result.is_ok(), "Valid attestation should succeed");
    
    let validators = client.get_validators();
    assert!(validators.contains(&validator), "Validator should be registered");
}

#[test]
fn test_invalid_signature_rejected() {
    let (env, client, admin) = setup();
    
    let issuer = Address::generate(&env);
    let validator = Address::generate(&env);
    
    client.register_issuer(&issuer, &String::from_str(&env, "Test Federation"));
    
    let (keypair, _public_key) = generate_keypair();
    
    let mut attestation = sign_attestation(&env, &keypair, &issuer, &validator, "UEFA B License", 0);
    
    attestation.signature = Vec::from_slice(&env, &[0u8; 64]);
    
    let result = client.try_register_validator_with_attestation(&validator, &attestation);
    assert_eq!(
        result,
        Err(Ok(scoutchain_verification::VerificationError::InvalidAttestation)),
        "Tampered signature should be rejected"
    );
}

#[test]
fn test_untrusted_issuer_rejected() {
    let (env, client, admin) = setup();
    
    let issuer = Address::generate(&env);
    let validator = Address::generate(&env);
    
    let (keypair, _public_key) = generate_keypair();
    
    let attestation = sign_attestation(&env, &keypair, &validator, "UEFA B License", 0);
    
    let result = client.try_register_validator_with_attestation(&validator, &attestation);
    assert_eq!(
        result,
        Err(Ok(scoutchain_verification::VerificationError::UntrustedIssuer)),
        "Untrusted issuer should be rejected"
    );
}

#[test]
fn test_legacy_register_validator_still_works() {
    let (env, client, admin) = setup();
    
    let validator = Address::generate(&env);
    
    let result = client.try_register_validator(&validator, &String::from_str(&env, "UEFA B License"));
    assert!(result.is_ok(), "Legacy register_validator should still work");
    
    let validators = client.get_validators();
    assert!(validators.contains(&validator), "Validator should be registered via legacy path");
}
