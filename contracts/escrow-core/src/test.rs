#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, Client as TokenClient},
    Address, BytesN, Env,
};

fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(env, &contract_address.address()),
        StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_happy_path_deposit_and_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token(&env, &token_admin);

    let initiator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let amount = 10_000_000i128;
    token_admin_client.mint(&initiator, &amount);

    let deadline = env.ledger().timestamp() + 3600;
    let engagement_id = BytesN::from_array(&env, &[1u8; 32]);

    client.initialize(
        &1,
        &initiator,
        &beneficiary,
        &Some(arbiter.clone()),
        &token.address,
        &amount,
        &deadline,
        &engagement_id,
    );

    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Created);
    assert_eq!(state.amount, amount);
    assert_eq!(state.funded_amount, 0);

    // Deposit
    client.deposit(&initiator);
    assert_eq!(token.balance(&contract_id), amount);
    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Funded);
    assert_eq!(state.funded_amount, amount);

    // Release to beneficiary
    client.release(&initiator);
    assert_eq!(token.balance(&beneficiary), amount);
    assert_eq!(token.balance(&contract_id), 0);

    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Completed);
    assert_eq!(state.funded_amount, 0);
}

#[test]
fn test_timelock_refund() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token(&env, &token_admin);

    let initiator = Address::generate(&env);
    let beneficiary = Address::generate(&env);

    let amount = 5_000_000i128;
    token_admin_client.mint(&initiator, &amount);

    let deadline = env.ledger().timestamp() + 1000;
    let engagement_id = BytesN::from_array(&env, &[2u8; 32]);

    client.initialize(
        &2,
        &initiator,
        &beneficiary,
        &None,
        &token.address,
        &amount,
        &deadline,
        &engagement_id,
    );

    client.deposit(&initiator);

    // Try refunding before deadline -> fails
    let err = client.try_refund(&initiator);
    assert!(err.is_err());

    // Advance time past deadline
    env.ledger().set_timestamp(deadline + 10);

    // Initiator refunds
    client.refund(&initiator);
    assert_eq!(token.balance(&initiator), amount);
    assert_eq!(token.balance(&contract_id), 0);

    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Refunded);
}

#[test]
fn test_dispute_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(EscrowContract, ());
    let client = EscrowContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token(&env, &token_admin);

    let initiator = Address::generate(&env);
    let beneficiary = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let amount = 100i128;
    token_admin_client.mint(&initiator, &amount);

    let deadline = env.ledger().timestamp() + 5000;
    let engagement_id = BytesN::from_array(&env, &[3u8; 32]);

    client.initialize(
        &3,
        &initiator,
        &beneficiary,
        &Some(arbiter.clone()),
        &token.address,
        &amount,
        &deadline,
        &engagement_id,
    );

    client.deposit(&initiator);

    // Beneficiary raises dispute
    client.dispute(&beneficiary);
    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Disputed);

    // Arbiter resolves dispute 70% to beneficiary, 30% to initiator
    client.resolve_dispute(&arbiter, &70, &30);

    assert_eq!(token.balance(&beneficiary), 70);
    assert_eq!(token.balance(&initiator), 30);
    assert_eq!(token.balance(&contract_id), 0);

    let state = client.get_escrow();
    assert_eq!(state.status, EscrowStatus::Completed);
}
