#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Vec,
};

fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(env, &contract_address.address()),
        StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_multisig_2_of_3_release() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MultisigEscrowContract, ());
    let client = MultisigEscrowContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token(&env, &token_admin);

    let s1 = Address::generate(&env);
    let s2 = Address::generate(&env);
    let s3 = Address::generate(&env);
    let recipient = Address::generate(&env);

    let mut signers = Vec::new(&env);
    signers.push_back(s1.clone());
    signers.push_back(s2.clone());
    signers.push_back(s3.clone());

    client.initialize(&signers, &2, &token.address);

    // Fund the multisig contract
    token_admin_client.mint(&contract_id, &500_000);

    // Signer 1 submits proposal to release 500k to recipient
    let prop_id = client.submit_proposal(
        &s1,
        &MultisigAction::Release {
            to: recipient.clone(),
            amount: 500_000,
        },
    );

    // Try executing with only 1 approval -> fails
    let err = client.try_execute_proposal(&s1, &prop_id);
    assert!(err.is_err());

    // Signer 2 approves
    client.approve_proposal(&s2, &prop_id);

    // Now execute proposal
    client.execute_proposal(&s1, &prop_id);

    assert_eq!(token.balance(&recipient), 500_000);
    assert_eq!(token.balance(&contract_id), 0);

    let prop = client.get_proposal(&prop_id);
    assert_eq!(prop.executed, true);
    assert_eq!(prop.approvals_count, 2);
}
