use soroban_sdk::{symbol_short, Address, Env, Symbol};

pub fn emit_initialized(env: &Env, id: u64, initiator: &Address, beneficiary: &Address, amount: i128) {
    let topics = (symbol_short!("init"), id, initiator.clone());
    env.events().publish(topics, (beneficiary.clone(), amount));
}

pub fn emit_deposited(env: &Env, id: u64, from: &Address, amount: i128) {
    let topics = (symbol_short!("deposit"), id, from.clone());
    env.events().publish(topics, amount);
}

pub fn emit_released(env: &Env, id: u64, to: &Address, amount: i128) {
    let topics = (symbol_short!("release"), id, to.clone());
    env.events().publish(topics, amount);
}

pub fn emit_refunded(env: &Env, id: u64, to: &Address, amount: i128) {
    let topics = (symbol_short!("refund"), id, to.clone());
    env.events().publish(topics, amount);
}

pub fn emit_disputed(env: &Env, id: u64, by: &Address) {
    let topics = (Symbol::new(env, "dispute"), id);
    env.events().publish(topics, by.clone());
}

pub fn emit_dispute_resolved(
    env: &Env,
    id: u64,
    beneficiary_amount: i128,
    initiator_amount: i128,
) {
    let topics = (Symbol::new(env, "resolved"), id);
    env.events().publish(topics, (beneficiary_amount, initiator_amount));
}
