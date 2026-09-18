use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub fn emit_initialized(env: &Env, threshold: u32, signers_count: u32) {
    let topics = (symbol_short!("msig_init"), threshold);
    env.events().publish(topics, signers_count);
}

pub fn emit_proposal_created(env: &Env, proposal_id: u32, proposer: &Address) {
    let topics = (Symbol::new(env, "prop_new"), proposal_id);
    env.events().publish(topics, proposer.clone());
}

pub fn emit_approved(env: &Env, proposal_id: u32, signer: &Address) {
    let topics = (Symbol::new(env, "approved"), proposal_id);
    env.events().publish(topics, signer.clone());
}

pub fn emit_executed(env: &Env, proposal_id: u32) {
    let topics = (Symbol::new(env, "executed"), proposal_id);
    env.events().publish(topics, true);
}
