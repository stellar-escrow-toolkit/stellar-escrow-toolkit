#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, BytesN, Env, Vec,
};

fn create_token<'a>(env: &Env, admin: &Address) -> (TokenClient<'a>, StellarAssetClient<'a>) {
    let contract_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        TokenClient::new(env, &contract_address.address()),
        StellarAssetClient::new(env, &contract_address.address()),
    )
}

#[test]
fn test_milestone_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(MilestoneEscrowContract, ());
    let client = MilestoneEscrowContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let (token, token_admin_client) = create_token(&env, &token_admin);

    let client_addr = Address::generate(&env);
    let contractor = Address::generate(&env);
    let arbiter = Address::generate(&env);

    let mut milestones = Vec::new(&env);
    milestones.push_back(MilestoneInit {
        id: 1,
        description_hash: BytesN::from_array(&env, &[10u8; 32]),
        amount: 300_000,
    });
    milestones.push_back(MilestoneInit {
        id: 2,
        description_hash: BytesN::from_array(&env, &[20u8; 32]),
        amount: 700_000,
    });

    token_admin_client.mint(&client_addr, &1_000_000);

    client.initialize(
        &client_addr,
        &contractor,
        &Some(arbiter.clone()),
        &token.address,
        &milestones,
    );

    let config = client.get_config();
    assert_eq!(config.total_amount, 1_000_000);
    assert_eq!(config.is_funded, false);

    // Deposit funds
    client.deposit(&client_addr);
    assert_eq!(token.balance(&contract_id), 1_000_000);

    // Submit Milestone 1
    let proof1 = BytesN::from_array(&env, &[99u8; 32]);
    client.submit_milestone(&contractor, &1, &proof1);

    let m1 = client.get_milestone(&1);
    assert_eq!(m1.status, MilestoneStatus::Submitted);
    assert_eq!(m1.proof_hash, Some(proof1));

    // Client approves Milestone 1
    client.approve_and_release(&client_addr, &1);

    assert_eq!(token.balance(&contractor), 300_000);
    assert_eq!(token.balance(&contract_id), 700_000);

    let m1_released = client.get_milestone(&1);
    assert_eq!(m1_released.status, MilestoneStatus::Released);

    // Submit Milestone 2
    let proof2 = BytesN::from_array(&env, &[88u8; 32]);
    client.submit_milestone(&contractor, &2, &proof2);

    // Dispute Milestone 2
    client.dispute_milestone(&client_addr, &2);
    let m2_disputed = client.get_milestone(&2);
    assert_eq!(m2_disputed.status, MilestoneStatus::Disputed);

    // Arbiter resolves dispute in favor of contractor
    client.resolve_milestone(&arbiter, &2, &true);

    assert_eq!(token.balance(&contractor), 1_000_000);
    assert_eq!(token.balance(&contract_id), 0);
}
