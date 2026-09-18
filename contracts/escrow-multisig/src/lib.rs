#![no_std]

mod errors;
mod events;
pub mod types;

#[cfg(test)]
mod test;

use errors::MultisigError;
use events::{emit_approved, emit_executed, emit_initialized, emit_proposal_created};
use soroban_sdk::{
    contract, contractimpl, token, Address, Env, Vec,
};
use types::{DataKey, MultisigAction, MultisigConfig, Proposal};

#[contract]
pub struct MultisigEscrowContract;

#[contractimpl]
impl MultisigEscrowContract {
    /// Initializes an M-of-N multisig arbiter / escrow vault.
    pub fn initialize(
        env: Env,
        signers: Vec<Address>,
        threshold: u32,
        token: Address,
    ) -> Result<(), MultisigError> {
        if env.storage().instance().has(&DataKey::Config) {
            return Err(MultisigError::AlreadyInitialized);
        }

        let count = signers.len();
        if count == 0 {
            return Err(MultisigError::SignersEmpty);
        }

        if threshold == 0 || threshold > count {
            return Err(MultisigError::InvalidThreshold);
        }

        let config = MultisigConfig {
            signers,
            threshold,
            token,
        };

        env.storage().instance().set(&DataKey::Config, &config);
        env.storage().instance().set(&DataKey::ProposalCount, &0u32);

        emit_initialized(&env, threshold, count);
        Ok(())
    }

    /// Submits a new release, refund, or threshold action proposal.
    pub fn submit_proposal(
        env: Env,
        caller: Address,
        action: MultisigAction,
    ) -> Result<u32, MultisigError> {
        caller.require_auth();

        let config: MultisigConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MultisigError::NotInitialized)?;

        if !Self::is_signer(&config.signers, &caller) {
            return Err(MultisigError::NotSigner);
        }

        let mut count: u32 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);

        count += 1;

        let proposal = Proposal {
            id: count,
            action,
            approvals_count: 1,
            executed: false,
            created_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(count), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::Approved(count, caller.clone()), &true);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &count);

        emit_proposal_created(&env, count, &caller);
        emit_approved(&env, count, &caller);

        Ok(count)
    }

    /// Approves an existing proposal.
    pub fn approve_proposal(
        env: Env,
        signer: Address,
        proposal_id: u32,
    ) -> Result<(), MultisigError> {
        signer.require_auth();

        let config: MultisigConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MultisigError::NotInitialized)?;

        if !Self::is_signer(&config.signers, &signer) {
            return Err(MultisigError::NotSigner);
        }

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(MultisigError::ProposalNotFound)?;

        if proposal.executed {
            return Err(MultisigError::AlreadyExecuted);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Approved(proposal_id, signer.clone()))
        {
            return Err(MultisigError::AlreadyApproved);
        }

        proposal.approvals_count += 1;
        env.storage()
            .persistent()
            .set(&DataKey::Approved(proposal_id, signer.clone()), &true);
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        emit_approved(&env, proposal_id, &signer);
        Ok(())
    }

    /// Executes a proposal that has reached the quorum threshold.
    pub fn execute_proposal(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), MultisigError> {
        caller.require_auth();

        let mut config: MultisigConfig = env
            .storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MultisigError::NotInitialized)?;

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(MultisigError::ProposalNotFound)?;

        if proposal.executed {
            return Err(MultisigError::AlreadyExecuted);
        }

        if proposal.approvals_count < config.threshold {
            return Err(MultisigError::ThresholdNotMet);
        }

        match &proposal.action {
            MultisigAction::Release { to, amount } => {
                if *amount <= 0 {
                    return Err(MultisigError::InvalidAmount);
                }
                let token_client = token::Client::new(&env, &config.token);
                token_client.transfer(&env.current_contract_address(), to, amount);
            }
            MultisigAction::Refund { to, amount } => {
                if *amount <= 0 {
                    return Err(MultisigError::InvalidAmount);
                }
                let token_client = token::Client::new(&env, &config.token);
                token_client.transfer(&env.current_contract_address(), to, amount);
            }
            MultisigAction::UpdateThreshold { new_threshold } => {
                if *new_threshold == 0 || *new_threshold > config.signers.len() {
                    return Err(MultisigError::InvalidThreshold);
                }
                config.threshold = *new_threshold;
                env.storage().instance().set(&DataKey::Config, &config);
            }
        }

        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        emit_executed(&env, proposal_id);
        Ok(())
    }

    /// Gets a proposal by ID.
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, MultisigError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .ok_or(MultisigError::ProposalNotFound)
    }

    /// Gets configuration.
    pub fn get_config(env: Env) -> Result<MultisigConfig, MultisigError> {
        env.storage()
            .instance()
            .get(&DataKey::Config)
            .ok_or(MultisigError::NotInitialized)
    }

    fn is_signer(signers: &Vec<Address>, target: &Address) -> bool {
        for i in 0..signers.len() {
            if signers.get(i).unwrap() == *target {
                return true;
            }
        }
        false
    }
}
