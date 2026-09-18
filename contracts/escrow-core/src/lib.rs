#![no_std]

mod errors;
mod events;
pub mod types;

#[cfg(test)]
mod test;

use errors::EscrowError;
use events::{
    emit_deposited, emit_dispute_resolved, emit_disputed, emit_initialized, emit_refunded,
    emit_released,
};
use soroban_sdk::{
    contract, contractimpl, token, Address, BytesN, Env,
};
use types::{DataKey, Escrow, EscrowStatus};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Initializes a new escrow contract instance.
    pub fn initialize(
        env: Env,
        id: u64,
        initiator: Address,
        beneficiary: Address,
        arbiter: Option<Address>,
        token: Address,
        amount: i128,
        deadline: u64,
        engagement_id: BytesN<32>,
    ) -> Result<(), EscrowError> {
        initiator.require_auth();

        if env.storage().instance().has(&DataKey::Initialized) {
            return Err(EscrowError::AlreadyInitialized);
        }

        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let current_time = env.ledger().timestamp();
        if deadline <= current_time {
            return Err(EscrowError::InvalidDeadline);
        }

        let escrow = Escrow {
            id,
            initiator: initiator.clone(),
            beneficiary: beneficiary.clone(),
            arbiter,
            token,
            amount,
            funded_amount: 0,
            deadline,
            status: EscrowStatus::Created,
            created_at: current_time,
            engagement_id,
        };

        env.storage().instance().set(&DataKey::Escrow, &escrow);
        env.storage().instance().set(&DataKey::Initialized, &true);

        emit_initialized(&env, id, &initiator, &beneficiary, amount);
        Ok(())
    }

    /// Deposits funds into the escrow.
    pub fn deposit(env: Env, from: Address) -> Result<(), EscrowError> {
        from.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)?;

        if escrow.status != EscrowStatus::Created {
            return Err(EscrowError::InvalidStatus);
        }

        let client = token::Client::new(&env, &escrow.token);
        client.transfer(&from, &env.current_contract_address(), &escrow.amount);

        escrow.funded_amount = escrow.amount;
        escrow.status = EscrowStatus::Funded;

        env.storage().instance().set(&DataKey::Escrow, &escrow);

        emit_deposited(&env, escrow.id, &from, escrow.amount);
        Ok(())
    }

    /// Releases escrowed funds to beneficiary.
    /// Can be invoked by initiator (happy path) or arbiter.
    pub fn release(env: Env, caller: Address) -> Result<(), EscrowError> {
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)?;

        if escrow.status != EscrowStatus::Funded {
            return Err(EscrowError::EscrowNotFunded);
        }

        let is_initiator = caller == escrow.initiator;
        let is_arbiter = escrow.arbiter.as_ref().map_or(false, |a| *a == caller);

        if !is_initiator && !is_arbiter {
            return Err(EscrowError::Unauthorized);
        }

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.beneficiary,
            &escrow.funded_amount,
        );

        let payout = escrow.funded_amount;
        escrow.funded_amount = 0;
        escrow.status = EscrowStatus::Completed;

        env.storage().instance().set(&DataKey::Escrow, &escrow);

        emit_released(&env, escrow.id, &escrow.beneficiary, payout);
        Ok(())
    }

    /// Refunds escrowed funds to initiator.
    /// Can be invoked by:
    /// 1. Beneficiary or arbiter at any time before completion.
    /// 2. Initiator if deadline has passed without dispute.
    pub fn refund(env: Env, caller: Address) -> Result<(), EscrowError> {
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)?;

        if escrow.status != EscrowStatus::Funded {
            return Err(EscrowError::EscrowNotFunded);
        }

        let current_time = env.ledger().timestamp();
        let is_beneficiary = caller == escrow.beneficiary;
        let is_arbiter = escrow.arbiter.as_ref().map_or(false, |a| *a == caller);
        let is_initiator_after_deadline =
            caller == escrow.initiator && current_time >= escrow.deadline;

        if !is_beneficiary && !is_arbiter && !is_initiator_after_deadline {
            if caller == escrow.initiator && current_time < escrow.deadline {
                return Err(EscrowError::DeadlineNotReached);
            }
            return Err(EscrowError::Unauthorized);
        }

        let token_client = token::Client::new(&env, &escrow.token);
        token_client.transfer(
            &env.current_contract_address(),
            &escrow.initiator,
            &escrow.funded_amount,
        );

        let refund_amount = escrow.funded_amount;
        escrow.funded_amount = 0;
        escrow.status = EscrowStatus::Refunded;

        env.storage().instance().set(&DataKey::Escrow, &escrow);

        emit_refunded(&env, escrow.id, &escrow.initiator, refund_amount);
        Ok(())
    }

    /// Initiates a dispute. Callable by initiator or beneficiary when funded.
    pub fn dispute(env: Env, caller: Address) -> Result<(), EscrowError> {
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)?;

        if escrow.status != EscrowStatus::Funded {
            return Err(EscrowError::EscrowNotFunded);
        }

        if escrow.arbiter.is_none() {
            return Err(EscrowError::ArbiterRequired);
        }

        if caller != escrow.initiator && caller != escrow.beneficiary {
            return Err(EscrowError::Unauthorized);
        }

        escrow.status = EscrowStatus::Disputed;
        env.storage().instance().set(&DataKey::Escrow, &escrow);

        emit_disputed(&env, escrow.id, &caller);
        Ok(())
    }

    /// Resolves an ongoing dispute by splitting or allocating funds.
    /// Callable solely by the designated arbiter.
    pub fn resolve_dispute(
        env: Env,
        caller: Address,
        beneficiary_amount: i128,
        initiator_amount: i128,
    ) -> Result<(), EscrowError> {
        caller.require_auth();

        let mut escrow: Escrow = env
            .storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)?;

        if escrow.status != EscrowStatus::Disputed {
            return Err(EscrowError::InvalidStatus);
        }

        match &escrow.arbiter {
            Some(arbiter) if *arbiter == caller => {}
            _ => return Err(EscrowError::Unauthorized),
        }

        if beneficiary_amount < 0 || initiator_amount < 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if beneficiary_amount + initiator_amount != escrow.funded_amount {
            return Err(EscrowError::DisputeResolutionMismatch);
        }

        let token_client = token::Client::new(&env, &escrow.token);

        if beneficiary_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.beneficiary,
                &beneficiary_amount,
            );
        }

        if initiator_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &escrow.initiator,
                &initiator_amount,
            );
        }

        escrow.funded_amount = 0;
        escrow.status = EscrowStatus::Completed;

        env.storage().instance().set(&DataKey::Escrow, &escrow);

        emit_dispute_resolved(
            &env,
            escrow.id,
            beneficiary_amount,
            initiator_amount,
        );
        Ok(())
    }

    /// Fetches the current state of the escrow.
    pub fn get_escrow(env: Env) -> Result<Escrow, EscrowError> {
        env.storage()
            .instance()
            .get(&DataKey::Escrow)
            .ok_or(EscrowError::NotInitialized)
    }
}
