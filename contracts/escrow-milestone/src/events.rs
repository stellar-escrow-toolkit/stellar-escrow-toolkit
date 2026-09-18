use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub fn emit_initialized(env: &Env, client: &Address, contractor: &Address, total_amount: i128, count: u32) {
    let topics = (symbol_short!("ms_init"), client.clone(), contractor.clone());
    env.events().publish(topics, (total_amount, count));
}

pub fn emit_milestone_submitted(env: &Env, milestone_id: u32, proof: &BytesN<32>) {
    let topics = (Symbol::new(env, "ms_submit"), milestone_id);
    env.events().publish(topics, proof.clone());
}

pub fn emit_milestone_approved(env: &Env, milestone_id: u32, by: &Address) {
    let topics = (Symbol::new(env, "ms_apprv"), milestone_id);
    env.events().publish(topics, by.clone());
}

pub fn emit_milestone_released(env: &Env, milestone_id: u32, to: &Address, amount: i128) {
    let topics = (Symbol::new(env, "ms_rel"), milestone_id, to.clone());
    env.events().publish(topics, amount);
}

pub fn emit_milestone_disputed(env: &Env, milestone_id: u32, by: &Address) {
    let topics = (Symbol::new(env, "ms_disp"), milestone_id);
    env.events().publish(topics, by.clone());
}

pub fn emit_milestone_resolved(env: &Env, milestone_id: u32, approved: bool) {
    let topics = (Symbol::new(env, "ms_res"), milestone_id);
    env.events().publish(topics, approved);
}
