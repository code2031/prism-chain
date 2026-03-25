use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Program ID
// ---------------------------------------------------------------------------

solana_program::declare_id!("PrismBurn1111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for the fee state PDA.
pub const FEE_STATE_SEED: &[u8] = b"prism_fee_state";
/// Seed for the treasury PDA.
pub const TREASURY_SEED: &[u8] = b"prism_treasury";
/// Seed for the staker reward pool PDA.
pub const STAKER_POOL_SEED: &[u8] = b"prism_staker_pool";

/// Basis-point denominator.
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Fee distribution schedule (basis points, must sum to 10_000).
pub const BURN_RATE_BPS: u64 = 5_000;       // 50% burned
pub const VALIDATOR_SHARE_BPS: u64 = 3_000;  // 30% to block-producing validator
pub const TREASURY_SHARE_BPS: u64 = 1_000;   // 10% to protocol treasury
pub const STAKER_SHARE_BPS: u64 = 1_000;     // 10% to staker reward pool

// ---------------------------------------------------------------------------
// Account state
// ---------------------------------------------------------------------------

/// Global fee-burn state that tracks cumulative statistics.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct FeeState {
    /// Whether this account has been initialized.
    pub is_initialized: bool,
    /// Authority that can update parameters.
    pub authority: Pubkey,
    /// Treasury account that receives the treasury share.
    pub treasury: Pubkey,
    /// Staker reward pool that receives the staker share.
    pub staker_pool: Pubkey,
    /// Total lamports burned since genesis.
    pub total_burned: u64,
    /// Total lamports distributed to validators.
    pub total_validator_rewards: u64,
    /// Total lamports sent to treasury.
    pub total_treasury_deposits: u64,
    /// Total lamports sent to staker pool.
    pub total_staker_rewards: u64,
    /// Total number of fee distribution events processed.
    pub total_events: u64,
    /// Current burn rate in basis points (default 5000 = 50%).
    pub burn_rate_bps: u64,
    /// Current validator share in basis points.
    pub validator_share_bps: u64,
    /// Current treasury share in basis points.
    pub treasury_share_bps: u64,
    /// Current staker share in basis points.
    pub staker_share_bps: u64,
    /// Slot of the last fee distribution event.
    pub last_event_slot: u64,
    /// Bump seed for the fee state PDA.
    pub bump: u8,
}

impl FeeState {
    pub const LEN: usize = 1 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}

/// Emitted via `msg!` for off-chain indexers.  Structured as a log line that
/// can be parsed by the explorer or analytics pipelines.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BurnEvent {
    /// Slot in which the burn occurred.
    pub slot: u64,
    /// Total fee input.
    pub fee_amount: u64,
    /// Amount burned (destroyed).
    pub burned: u64,
    /// Amount sent to the block-producing validator.
    pub validator_share: u64,
    /// Amount sent to the treasury.
    pub treasury_share: u64,
    /// Amount sent to the staker reward pool.
    pub staker_share: u64,
    /// Cumulative burn total after this event.
    pub cumulative_burned: u64,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum FeeBurnInstruction {
    /// Initialize the global fee state.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Authority (payer)
    ///   1. `[writable]`         Fee state PDA
    ///   2. `[]`                 Treasury account
    ///   3. `[]`                 Staker pool account
    ///   4. `[]`                 System program
    Initialize,

    /// Distribute a collected transaction fee according to the burn schedule.
    ///
    /// This is called by the runtime (or a crank) after collecting fees for a
    /// block.  The `fee_amount` is the total fee to be split.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Fee payer (runtime / crank)
    ///   1. `[writable]`         Fee state PDA
    ///   2. `[writable]`         Validator reward account
    ///   3. `[writable]`         Treasury PDA
    ///   4. `[writable]`         Staker pool PDA
    ///   5. `[]`                 Clock sysvar
    ///   6. `[]`                 System program
    DistributeFee {
        fee_amount: u64,
    },

    /// Update fee distribution parameters (authority only).
    ///
    /// Accounts:
    ///   0. `[signer]`   Authority
    ///   1. `[writable]` Fee state PDA
    UpdateParams {
        burn_rate_bps: u64,
        validator_share_bps: u64,
        treasury_share_bps: u64,
        staker_share_bps: u64,
    },

    /// Query cumulative burn statistics.  This is a no-op instruction whose
    /// side effect is logging the current state for off-chain consumers.
    ///
    /// Accounts:
    ///   0. `[]` Fee state PDA
    QueryStats,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum FeeBurnError {
    #[error("Fee state already initialized")]
    AlreadyInitialized,
    #[error("Fee state not initialized")]
    NotInitialized,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Fee shares must sum to 10000 bps")]
    InvalidFeeShares,
    #[error("Zero fee amount")]
    ZeroFeeAmount,
}

impl From<FeeBurnError> for ProgramError {
    fn from(e: FeeBurnError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = FeeBurnInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        FeeBurnInstruction::Initialize => process_initialize(program_id, accounts),
        FeeBurnInstruction::DistributeFee { fee_amount } => {
            process_distribute_fee(program_id, accounts, fee_amount)
        }
        FeeBurnInstruction::UpdateParams {
            burn_rate_bps,
            validator_share_bps,
            treasury_share_bps,
            staker_share_bps,
        } => process_update_params(
            program_id,
            accounts,
            burn_rate_bps,
            validator_share_bps,
            treasury_share_bps,
            staker_share_bps,
        ),
        FeeBurnInstruction::QueryStats => process_query_stats(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let fee_state_account = next_account_info(account_iter)?;
    let treasury = next_account_info(account_iter)?;
    let staker_pool = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (fee_state_pda, bump) = Pubkey::find_program_address(&[FEE_STATE_SEED], program_id);
    if fee_state_pda != *fee_state_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::get()?;
    let space = FeeState::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            fee_state_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            fee_state_account.clone(),
            system_program.clone(),
        ],
        &[&[FEE_STATE_SEED, &[bump]]],
    )?;

    let state = FeeState {
        is_initialized: true,
        authority: *authority.key,
        treasury: *treasury.key,
        staker_pool: *staker_pool.key,
        total_burned: 0,
        total_validator_rewards: 0,
        total_treasury_deposits: 0,
        total_staker_rewards: 0,
        total_events: 0,
        burn_rate_bps: BURN_RATE_BPS,
        validator_share_bps: VALIDATOR_SHARE_BPS,
        treasury_share_bps: TREASURY_SHARE_BPS,
        staker_share_bps: STAKER_SHARE_BPS,
        last_event_slot: 0,
        bump,
    };

    state.serialize(&mut &mut fee_state_account.data.borrow_mut()[..])?;

    msg!(
        "Fee burn program initialized. Burn: {}%, Validator: {}%, Treasury: {}%, Staker: {}%",
        BURN_RATE_BPS / 100,
        VALIDATOR_SHARE_BPS / 100,
        TREASURY_SHARE_BPS / 100,
        STAKER_SHARE_BPS / 100,
    );
    Ok(())
}

fn process_distribute_fee(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    fee_amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let fee_payer = next_account_info(account_iter)?;
    let fee_state_account = next_account_info(account_iter)?;
    let validator_account = next_account_info(account_iter)?;
    let treasury_account = next_account_info(account_iter)?;
    let staker_pool_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !fee_payer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if fee_amount == 0 {
        return Err(FeeBurnError::ZeroFeeAmount.into());
    }

    let mut state = FeeState::try_from_slice(&fee_state_account.data.borrow())?;
    if !state.is_initialized {
        return Err(FeeBurnError::NotInitialized.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    // Calculate each share.
    let burn_amount = fee_amount
        .checked_mul(state.burn_rate_bps)
        .ok_or(FeeBurnError::Overflow)?
        / BPS_DENOMINATOR;

    let validator_amount = fee_amount
        .checked_mul(state.validator_share_bps)
        .ok_or(FeeBurnError::Overflow)?
        / BPS_DENOMINATOR;

    let treasury_amount = fee_amount
        .checked_mul(state.treasury_share_bps)
        .ok_or(FeeBurnError::Overflow)?
        / BPS_DENOMINATOR;

    // Staker share gets any remainder from rounding to ensure no dust is lost.
    let staker_amount = fee_amount
        .checked_sub(burn_amount)
        .ok_or(FeeBurnError::Overflow)?
        .checked_sub(validator_amount)
        .ok_or(FeeBurnError::Overflow)?
        .checked_sub(treasury_amount)
        .ok_or(FeeBurnError::Overflow)?;

    // ---- Burn: reduce fee payer lamports, they vanish from total supply ----
    // In a real runtime integration the burn would happen by not crediting the
    // burned portion to anyone.  Here we simulate by decrementing the payer.
    **fee_payer.try_borrow_mut_lamports()? -= burn_amount;

    // ---- Validator share ----
    **fee_payer.try_borrow_mut_lamports()? -= validator_amount;
    **validator_account.try_borrow_mut_lamports()? += validator_amount;

    // ---- Treasury share ----
    **fee_payer.try_borrow_mut_lamports()? -= treasury_amount;
    **treasury_account.try_borrow_mut_lamports()? += treasury_amount;

    // ---- Staker pool share ----
    **fee_payer.try_borrow_mut_lamports()? -= staker_amount;
    **staker_pool_account.try_borrow_mut_lamports()? += staker_amount;

    // ---- Update cumulative stats ----
    state.total_burned = state
        .total_burned
        .checked_add(burn_amount)
        .ok_or(FeeBurnError::Overflow)?;
    state.total_validator_rewards = state
        .total_validator_rewards
        .checked_add(validator_amount)
        .ok_or(FeeBurnError::Overflow)?;
    state.total_treasury_deposits = state
        .total_treasury_deposits
        .checked_add(treasury_amount)
        .ok_or(FeeBurnError::Overflow)?;
    state.total_staker_rewards = state
        .total_staker_rewards
        .checked_add(staker_amount)
        .ok_or(FeeBurnError::Overflow)?;
    state.total_events = state
        .total_events
        .checked_add(1)
        .ok_or(FeeBurnError::Overflow)?;
    state.last_event_slot = clock.slot;

    state.serialize(&mut &mut fee_state_account.data.borrow_mut()[..])?;

    // ---- Emit burn event for indexers ----
    let event = BurnEvent {
        slot: clock.slot,
        fee_amount,
        burned: burn_amount,
        validator_share: validator_amount,
        treasury_share: treasury_amount,
        staker_share: staker_amount,
        cumulative_burned: state.total_burned,
    };

    msg!("BURN_EVENT:{}", bs58_encode_event(&event));
    msg!(
        "Fee distributed: {} total | {} burned | {} validator | {} treasury | {} stakers",
        fee_amount,
        burn_amount,
        validator_amount,
        treasury_amount,
        staker_amount,
    );
    msg!(
        "Cumulative burned: {} lamports ({:.4} PRISM)",
        state.total_burned,
        state.total_burned as f64 / 1_000_000_000.0,
    );

    Ok(())
}

fn process_update_params(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    burn_rate_bps: u64,
    validator_share_bps: u64,
    treasury_share_bps: u64,
    staker_share_bps: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let fee_state_account = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut state = FeeState::try_from_slice(&fee_state_account.data.borrow())?;
    if !state.is_initialized {
        return Err(FeeBurnError::NotInitialized.into());
    }
    if state.authority != *authority.key {
        return Err(FeeBurnError::Unauthorized.into());
    }

    // Validate that shares sum to 100%.
    let total = burn_rate_bps
        .checked_add(validator_share_bps)
        .ok_or(FeeBurnError::Overflow)?
        .checked_add(treasury_share_bps)
        .ok_or(FeeBurnError::Overflow)?
        .checked_add(staker_share_bps)
        .ok_or(FeeBurnError::Overflow)?;

    if total != BPS_DENOMINATOR {
        msg!("Fee shares sum to {} bps, expected {}", total, BPS_DENOMINATOR);
        return Err(FeeBurnError::InvalidFeeShares.into());
    }

    state.burn_rate_bps = burn_rate_bps;
    state.validator_share_bps = validator_share_bps;
    state.treasury_share_bps = treasury_share_bps;
    state.staker_share_bps = staker_share_bps;

    state.serialize(&mut &mut fee_state_account.data.borrow_mut()[..])?;

    msg!(
        "Fee params updated. Burn: {}%, Validator: {}%, Treasury: {}%, Staker: {}%",
        burn_rate_bps / 100,
        validator_share_bps / 100,
        treasury_share_bps / 100,
        staker_share_bps / 100,
    );
    Ok(())
}

fn process_query_stats(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let fee_state_account = next_account_info(account_iter)?;

    let state = FeeState::try_from_slice(&fee_state_account.data.borrow())?;
    if !state.is_initialized {
        return Err(FeeBurnError::NotInitialized.into());
    }

    msg!("=== Prism Fee Burn Statistics ===");
    msg!("Total events processed: {}", state.total_events);
    msg!(
        "Total burned:            {} lamports ({:.4} PRISM)",
        state.total_burned,
        state.total_burned as f64 / 1_000_000_000.0,
    );
    msg!(
        "Total validator rewards: {} lamports ({:.4} PRISM)",
        state.total_validator_rewards,
        state.total_validator_rewards as f64 / 1_000_000_000.0,
    );
    msg!(
        "Total treasury deposits: {} lamports ({:.4} PRISM)",
        state.total_treasury_deposits,
        state.total_treasury_deposits as f64 / 1_000_000_000.0,
    );
    msg!(
        "Total staker rewards:    {} lamports ({:.4} PRISM)",
        state.total_staker_rewards,
        state.total_staker_rewards as f64 / 1_000_000_000.0,
    );
    msg!("Current rates: burn={}bps validator={}bps treasury={}bps staker={}bps",
        state.burn_rate_bps,
        state.validator_share_bps,
        state.treasury_share_bps,
        state.staker_share_bps,
    );
    msg!("Last event slot: {}", state.last_event_slot);

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Simple base58-ish encoding of the burn event for structured log parsing.
/// In production, this would use a proper event framework (e.g., Anchor events).
fn bs58_encode_event(event: &BurnEvent) -> String {
    format!(
        "slot={},fee={},burned={},validator={},treasury={},staker={},cumulative={}",
        event.slot,
        event.fee_amount,
        event.burned,
        event.validator_share,
        event.treasury_share,
        event.staker_share,
        event.cumulative_burned,
    )
}
