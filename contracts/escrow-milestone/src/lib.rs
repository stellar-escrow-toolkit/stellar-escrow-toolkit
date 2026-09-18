#![no_std]

mod errors;
mod events;
pub mod types;

#[cfg(test)]
mod test;

use errors::MilestoneError;
use events::{
    emit_initialized, emit_milestone_approved, emit_milestone_disputed, emit_milestone_released,
    emit_milestone_resolved, emit_milestone_submitted,
};
use soroban_sdk::{
    contract, contractimpl, token, Address, BytesN, Env, Vec,
};
use types::{DataKey, Milestone, MilestoneEscrow, MilestoneInit, MilestoneStatus};

#[contract]
pub struct MilestoneEscrowContract;

#[contractimpl]
impl MilestoneEscrowContract {
    /// Initializes a milestone escrow agreement with pre-defined milestones.
    pub fn initialize(
        env: Env,
        client: Address,
        contractor: Address,
        arbiter: Option<Address>,
        token: Address,
        milestones: Vec<MilestoneInit>,
    ) -> Result<(), MilestoneError> {
        client.require_auth();

        if env.storage().instance().has(&DataKey::Config) {
            return Err(MilestoneError::AlreadyInitialized);
        }

        let milestone_count = milestones.len();
        if milestone_count == 0 {
            return Err(MilestoneError::InvalidMilestoneCount);
        }

        let mut total_amount: i128 = 0;
        for i in 0..milestone_count {
            let m_init = milestones.get(i).unwrap();
            if m_init.amount <= 0 {
                return Err(MilestoneError::AmountMismatch);
            }
            total_amount = total_amount
                .checked_add(m_init.amount)
                .ok_or(MilestoneError::AmountMismatch)?;

            let milestone = Milestone {
                id: m_init.id,
                description_hash: m_init.description_hash,
                amount: m_init.amount,
                status: MilestoneStatus::Pending,
                proof_hash: None,
                submitted_at: 0,
                approved_at: 0,
            };
            env.storage()
                .persistent()
                .set(&DataKey::Milestone(m_init.id), &milestone);
        }

        let config = MilestoneEscrow {
            client: client.clone(),
            contractor: contractor.clone(),
            arbiter,
            token,
            total_amount,
            released_amount: 0,
            is_funded: false,
            milestone_count,
        };

        env.storage().instance().set(&DataKey::Config, &config);

        emit_initialized(&env, &client, &contractor, total_amount, milestone_count);
        Ok(())
    }

    /// Deposits the full escrow sum into the milestone contract.
    pub fn deposit(env: Env, caller: Address) -> Result<(), MilestoneError> {
        caller.require_auth();

        let mut config: MilestoneEscrow = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)?;

        if config.is_funded {
            return Err(MilestoneError::AlreadyFunded);
        }

        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&caller, &env.current_contract_address(), &config.total_amount);

        config.is_funded = true;
        env.storage().instance().set(&DataKey::Config, &config);

        Ok(())
    }

    /// Submits deliverables / proof of completion for a milestone.
    pub fn submit_milestone(
        env: Env,
        caller: Address,
        milestone_id: u32,
        proof_hash: BytesN<32>,
    ) -> Result<(), MilestoneError> {
        caller.require_auth();

        let config: MilestoneEscrow = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)?;

        if !config.is_funded {
            return Err(MilestoneError::NotFunded);
        }

        if caller != config.contractor {
            return Err(MilestoneError::Unauthorized);
        }

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .ok_or(MilestoneError::MilestoneNotFound)?;

        if milestone.status != MilestoneStatus::Pending {
            return Err(MilestoneError::InvalidMilestoneStatus);
        }

        milestone.status = MilestoneStatus::Submitted;
        milestone.proof_hash = Some(proof_hash.clone());
        milestone.submitted_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&DataKey::Milestone(milestone_id), &milestone);

        emit_milestone_submitted(&env, milestone_id, &proof_hash);
        Ok(())
    }

    /// Approves and releases payment for an individual milestone.
    pub fn approve_and_release(
        env: Env,
        caller: Address,
        milestone_id: u32,
    ) -> Result<(), MilestoneError> {
        caller.require_auth();

        let mut config: MilestoneEscrow = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)?;

        if !config.is_funded {
            return Err(MilestoneError::NotFunded);
        }

        let is_client = caller == config.client;
        let is_arbiter = config.arbiter.as_ref().map_or(false, |a| *a == caller);

        if !is_client && !is_arbiter {
            return Err(MilestoneError::Unauthorized);
        }

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .ok_or(MilestoneError::MilestoneNotFound)?;

        if milestone.status != MilestoneStatus::Submitted {
            return Err(MilestoneError::InvalidMilestoneStatus);
        }

        let payout = milestone.amount;
        let token_client = token::Client::new(&env, &config.token);
        token_client.transfer(&env.current_contract_address(), &config.contractor, &payout);

        milestone.status = MilestoneStatus::Released;
        milestone.approved_at = env.ledger().timestamp();

        config.released_amount += payout;

        env.storage()
            .persistent()
            .set(&DataKey::Milestone(milestone_id), &milestone);
        env.storage().instance().set(&DataKey::Config, &config);

        emit_milestone_approved(&env, milestone_id, &caller);
        emit_milestone_released(&env, milestone_id, &config.contractor, payout);

        Ok(())
    }

    /// Raises a dispute for a milestone.
    pub fn dispute_milestone(
        env: Env,
        caller: Address,
        milestone_id: u32,
    ) -> Result<(), MilestoneError> {
        caller.require_auth();

        let config: MilestoneEscrow = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)?;

        if !config.is_funded {
            return Err(MilestoneError::NotFunded);
        }

        if config.arbiter.is_none() {
            return Err(MilestoneError::ArbiterRequired);
        }

        if caller != config.client && caller != config.contractor {
            return Err(MilestoneError::Unauthorized);
        }

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .ok_or(MilestoneError::MilestoneNotFound)?;

        if milestone.status != MilestoneStatus::Submitted {
            return Err(MilestoneError::InvalidMilestoneStatus);
        }

        milestone.status = MilestoneStatus::Disputed;
        env.storage()
            .persistent()
            .set(&DataKey::Milestone(milestone_id), &milestone);

        emit_milestone_disputed(&env, milestone_id, &caller);
        Ok(())
    }

    /// Resolves a disputed milestone by either approving payout or refunding milestone to client.
    pub fn resolve_milestone(
        env: Env,
        caller: Address,
        milestone_id: u32,
        release_to_contractor: bool,
    ) -> Result<(), MilestoneError> {
        caller.require_auth();

        let mut config: MilestoneEscrow = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)?;

        match &config.arbiter {
            Some(arbiter) if *arbiter == caller => {}
            _ => return Err(MilestoneError::Unauthorized),
        }

        let mut milestone: Milestone = env
            .storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .ok_or(MilestoneError::MilestoneNotFound)?;

        if milestone.status != MilestoneStatus::Disputed {
            return Err(MilestoneError::InvalidMilestoneStatus);
        }

        let token_client = token::Client::new(&env, &config.token);
        let payout = milestone.amount;

        if release_to_contractor {
            token_client.transfer(&env.current_contract_address(), &config.contractor, &payout);
            milestone.status = MilestoneStatus::Released;
            config.released_amount += payout;
        } else {
            token_client.transfer(&env.current_contract_address(), &config.client, &payout);
            milestone.status = MilestoneStatus::Cancelled;
        }

        env.storage()
            .persistent()
            .set(&DataKey::Milestone(milestone_id), &milestone);
        env.storage().instance().set(&DataKey::Config, &config);

        emit_milestone_resolved(&env, milestone_id, release_to_contractor);
        Ok(())
    }

    /// Reads milestone configuration.
    pub fn get_config(env: Env) -> Result<MilestoneEscrow, MilestoneError> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MilestoneError::NotInitialized)
    }

    /// Reads specific milestone status.
    pub fn get_milestone(env: Env, milestone_id: u32) -> Result<Milestone, MilestoneError> {
        env.storage()
            .persistent()
            .get(&DataKey::Milestone(milestone_id))
            .ok_or(MilestoneError::MilestoneNotFound)
    }
}
