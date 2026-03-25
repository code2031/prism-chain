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

solana_program::declare_id!("PrismVest1111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for the vesting registry PDA.
pub const REGISTRY_SEED: &[u8] = b"prism_vesting_registry";
/// Seed prefix for individual vesting schedule PDAs.
pub const SCHEDULE_SEED: &[u8] = b"prism_vesting_schedule";
/// Seed for the token vault PDA that holds vested tokens.
pub const VAULT_SEED: &[u8] = b"prism_vesting_vault";

/// Maximum number of beneficiaries per vesting registry.
pub const MAX_BENEFICIARIES: usize = 256;
/// Seconds per 30-day month (used for cliff/duration arithmetic).
pub const SECONDS_PER_MONTH: i64 = 2_592_000;
/// Seconds per quarter (3 months).
pub const SECONDS_PER_QUARTER: i64 = 7_776_000;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Release schedule type.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum ReleaseSchedule {
    /// Tokens vest continuously every second after the cliff.
    Linear,
    /// Tokens vest in quarterly chunks after the cliff.
    Quarterly,
}

/// Status of a vesting schedule.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum VestingStatus {
    /// Actively vesting.
    Active,
    /// Fully vested and all tokens claimed.
    Completed,
    /// Revoked by authority — unvested portion returned.
    Revoked,
}

// ---------------------------------------------------------------------------
// Account state
// ---------------------------------------------------------------------------

/// Global vesting registry.  One per program deployment.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VestingRegistry {
    /// Whether initialized.
    pub is_initialized: bool,
    /// Authority that can create/revoke vesting schedules.
    pub authority: Pubkey,
    /// Vault holding all unvested tokens.
    pub vault: Pubkey,
    /// SPL Token mint for the vested token.
    pub token_mint: Pubkey,
    /// Total tokens deposited across all schedules.
    pub total_deposited: u64,
    /// Total tokens claimed by all beneficiaries.
    pub total_claimed: u64,
    /// Total tokens returned via revocations.
    pub total_revoked: u64,
    /// Number of active vesting schedules.
    pub active_schedules: u32,
    /// Counter used to derive unique schedule PDAs.
    pub schedule_counter: u64,
    /// Bump for the registry PDA.
    pub bump: u8,
}

impl VestingRegistry {
    pub const LEN: usize = 1 + 32 + 32 + 32 + 8 + 8 + 8 + 4 + 8 + 1;
}

/// Individual vesting schedule for a single beneficiary.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct VestingSchedule {
    /// Whether initialized.
    pub is_initialized: bool,
    /// The registry this schedule belongs to.
    pub registry: Pubkey,
    /// Beneficiary who can claim vested tokens.
    pub beneficiary: Pubkey,
    /// Total tokens allocated to this schedule.
    pub total_amount: u64,
    /// Tokens already claimed by the beneficiary.
    pub claimed_amount: u64,
    /// Unix timestamp when vesting starts (tokens are deposited).
    pub start_timestamp: i64,
    /// Cliff duration in seconds.  No tokens vest before start + cliff.
    pub cliff_seconds: i64,
    /// Total vesting duration in seconds (from start, including cliff).
    pub duration_seconds: i64,
    /// Release schedule type.
    pub schedule: ReleaseSchedule,
    /// Current status.
    pub status: VestingStatus,
    /// Whether this schedule is revocable by the authority.
    pub revocable: bool,
    /// Unique index for PDA derivation.
    pub index: u64,
    /// Bump for this schedule's PDA.
    pub bump: u8,
}

impl VestingSchedule {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum VestingInstruction {
    /// Initialize the vesting registry.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Authority (payer)
    ///   1. `[writable]`         Registry PDA
    ///   2. `[]`                 Vault account (token account owned by registry PDA)
    ///   3. `[]`                 Token mint
    ///   4. `[]`                 System program
    InitializeRegistry,

    /// Create a new vesting schedule for a beneficiary.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Authority
    ///   1. `[writable]`         Registry PDA
    ///   2. `[writable]`         Schedule PDA (derived from index)
    ///   3. `[]`                 Beneficiary (read-only, does not sign)
    ///   4. `[]`                 System program
    CreateSchedule {
        beneficiary: Pubkey,
        total_amount: u64,
        cliff_months: u32,
        duration_months: u32,
        schedule: ReleaseSchedule,
        revocable: bool,
    },

    /// Claim vested tokens.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Beneficiary
    ///   1. `[writable]`         Schedule PDA
    ///   2. `[writable]`         Registry PDA
    ///   3. `[writable]`         Vault token account
    ///   4. `[writable]`         Beneficiary token account
    ///   5. `[]`                 Vault authority PDA
    ///   6. `[]`                 SPL Token program
    ///   7. `[]`                 Clock sysvar
    Claim,

    /// Revoke a vesting schedule (authority only, schedule must be revocable).
    /// Unvested tokens are returned to the vault; already-vested tokens remain
    /// claimable by the beneficiary.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Authority
    ///   1. `[writable]`         Schedule PDA
    ///   2. `[writable]`         Registry PDA
    ///   3. `[]`                 Clock sysvar
    Revoke,

    /// Query vesting status for a schedule.  Logs current vested/claimable
    /// amounts.  No state mutation.
    ///
    /// Accounts:
    ///   0. `[]` Schedule PDA
    ///   1. `[]` Clock sysvar
    QuerySchedule,

    /// Transfer authority to a new key.
    ///
    /// Accounts:
    ///   0. `[signer]`   Current authority
    ///   1. `[writable]` Registry PDA
    TransferAuthority {
        new_authority: Pubkey,
    },
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum VestingError {
    #[error("Registry already initialized")]
    AlreadyInitialized,
    #[error("Registry not initialized")]
    NotInitialized,
    #[error("Schedule already initialized")]
    ScheduleAlreadyInitialized,
    #[error("Schedule not initialized")]
    ScheduleNotInitialized,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Nothing to claim")]
    NothingToClaim,
    #[error("Schedule is not revocable")]
    NotRevocable,
    #[error("Schedule already revoked")]
    AlreadyRevoked,
    #[error("Schedule already completed")]
    AlreadyCompleted,
    #[error("Cliff period has not ended")]
    CliffNotReached,
    #[error("Invalid duration — must be greater than cliff")]
    InvalidDuration,
    #[error("Zero amount")]
    ZeroAmount,
    #[error("Max beneficiaries reached")]
    MaxBeneficiaries,
}

impl From<VestingError> for ProgramError {
    fn from(e: VestingError) -> Self {
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
    let instruction = VestingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        VestingInstruction::InitializeRegistry => {
            process_initialize_registry(program_id, accounts)
        }
        VestingInstruction::CreateSchedule {
            beneficiary,
            total_amount,
            cliff_months,
            duration_months,
            schedule,
            revocable,
        } => process_create_schedule(
            program_id,
            accounts,
            beneficiary,
            total_amount,
            cliff_months,
            duration_months,
            schedule,
            revocable,
        ),
        VestingInstruction::Claim => process_claim(program_id, accounts),
        VestingInstruction::Revoke => process_revoke(program_id, accounts),
        VestingInstruction::QuerySchedule => process_query_schedule(program_id, accounts),
        VestingInstruction::TransferAuthority { new_authority } => {
            process_transfer_authority(program_id, accounts, new_authority)
        }
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_registry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let vault = next_account_info(account_iter)?;
    let token_mint = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (registry_pda, bump) = Pubkey::find_program_address(&[REGISTRY_SEED], program_id);
    if registry_pda != *registry_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::get()?;
    let space = VestingRegistry::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            registry_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            registry_account.clone(),
            system_program.clone(),
        ],
        &[&[REGISTRY_SEED, &[bump]]],
    )?;

    let registry = VestingRegistry {
        is_initialized: true,
        authority: *authority.key,
        vault: *vault.key,
        token_mint: *token_mint.key,
        total_deposited: 0,
        total_claimed: 0,
        total_revoked: 0,
        active_schedules: 0,
        schedule_counter: 0,
        bump,
    };

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Vesting registry initialized for mint {}", token_mint.key);
    Ok(())
}

fn process_create_schedule(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    beneficiary: Pubkey,
    total_amount: u64,
    cliff_months: u32,
    duration_months: u32,
    schedule: ReleaseSchedule,
    revocable: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let _beneficiary_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut registry = VestingRegistry::try_from_slice(&registry_account.data.borrow())?;
    if !registry.is_initialized {
        return Err(VestingError::NotInitialized.into());
    }
    if registry.authority != *authority.key {
        return Err(VestingError::Unauthorized.into());
    }
    if total_amount == 0 {
        return Err(VestingError::ZeroAmount.into());
    }
    if registry.active_schedules as usize >= MAX_BENEFICIARIES {
        return Err(VestingError::MaxBeneficiaries.into());
    }

    let cliff_seconds = (cliff_months as i64)
        .checked_mul(SECONDS_PER_MONTH)
        .ok_or(VestingError::Overflow)?;
    let duration_seconds = (duration_months as i64)
        .checked_mul(SECONDS_PER_MONTH)
        .ok_or(VestingError::Overflow)?;

    if duration_seconds <= cliff_seconds && duration_months > 0 && cliff_months > 0 {
        return Err(VestingError::InvalidDuration.into());
    }

    let index = registry.schedule_counter;
    let index_bytes = index.to_le_bytes();

    let (schedule_pda, schedule_bump) = Pubkey::find_program_address(
        &[SCHEDULE_SEED, registry_account.key.as_ref(), &index_bytes],
        program_id,
    );
    if schedule_pda != *schedule_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    let rent = Rent::get()?;
    let space = VestingSchedule::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            schedule_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            schedule_account.clone(),
            system_program.clone(),
        ],
        &[&[
            SCHEDULE_SEED,
            registry_account.key.as_ref(),
            &index_bytes,
            &[schedule_bump],
        ]],
    )?;

    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    let vesting_schedule = VestingSchedule {
        is_initialized: true,
        registry: *registry_account.key,
        beneficiary,
        total_amount,
        claimed_amount: 0,
        start_timestamp: now,
        cliff_seconds,
        duration_seconds,
        schedule,
        status: VestingStatus::Active,
        revocable,
        index,
        bump: schedule_bump,
    };

    vesting_schedule.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    registry.total_deposited = registry
        .total_deposited
        .checked_add(total_amount)
        .ok_or(VestingError::Overflow)?;
    registry.active_schedules = registry
        .active_schedules
        .checked_add(1)
        .ok_or(VestingError::Overflow)?;
    registry.schedule_counter = registry
        .schedule_counter
        .checked_add(1)
        .ok_or(VestingError::Overflow)?;

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!(
        "Vesting schedule #{} created: {} tokens for {} | cliff: {} months, duration: {} months, schedule: {:?}, revocable: {}",
        index,
        total_amount,
        beneficiary,
        cliff_months,
        duration_months,
        schedule,
        revocable,
    );
    Ok(())
}

fn process_claim(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let beneficiary = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let vault_token_account = next_account_info(account_iter)?;
    let beneficiary_token_account = next_account_info(account_iter)?;
    let vault_authority = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !beneficiary.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut schedule = VestingSchedule::try_from_slice(&schedule_account.data.borrow())?;
    if !schedule.is_initialized {
        return Err(VestingError::ScheduleNotInitialized.into());
    }
    if schedule.beneficiary != *beneficiary.key {
        return Err(VestingError::Unauthorized.into());
    }
    if schedule.status == VestingStatus::Completed {
        return Err(VestingError::AlreadyCompleted.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let vested = calculate_vested_amount(&schedule, now)?;
    let claimable = vested
        .checked_sub(schedule.claimed_amount)
        .ok_or(VestingError::Overflow)?;

    if claimable == 0 {
        return Err(VestingError::NothingToClaim.into());
    }

    // Derive vault authority PDA.
    let (vault_auth_pda, vault_auth_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, registry_account.key.as_ref()], program_id);
    if vault_auth_pda != *vault_authority.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // CPI: transfer tokens from vault to beneficiary.
    let transfer_ix = spl_token_transfer(
        token_program.key,
        vault_token_account.key,
        beneficiary_token_account.key,
        &vault_auth_pda,
        claimable,
    )?;

    invoke_signed(
        &transfer_ix,
        &[
            vault_token_account.clone(),
            beneficiary_token_account.clone(),
            vault_authority.clone(),
            token_program.clone(),
        ],
        &[&[VAULT_SEED, registry_account.key.as_ref(), &[vault_auth_bump]]],
    )?;

    schedule.claimed_amount = schedule
        .claimed_amount
        .checked_add(claimable)
        .ok_or(VestingError::Overflow)?;

    // Check if fully vested.
    if schedule.claimed_amount >= schedule.total_amount {
        schedule.status = VestingStatus::Completed;
    }

    schedule.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    // Update registry.
    let mut registry = VestingRegistry::try_from_slice(&registry_account.data.borrow())?;
    registry.total_claimed = registry
        .total_claimed
        .checked_add(claimable)
        .ok_or(VestingError::Overflow)?;

    if schedule.status == VestingStatus::Completed {
        registry.active_schedules = registry.active_schedules.saturating_sub(1);
    }

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!(
        "Claimed {} tokens (total claimed: {}/{}) for beneficiary {}",
        claimable,
        schedule.claimed_amount,
        schedule.total_amount,
        beneficiary.key,
    );
    Ok(())
}

fn process_revoke(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let registry = VestingRegistry::try_from_slice(&registry_account.data.borrow())?;
    if !registry.is_initialized {
        return Err(VestingError::NotInitialized.into());
    }
    if registry.authority != *authority.key {
        return Err(VestingError::Unauthorized.into());
    }

    let mut schedule = VestingSchedule::try_from_slice(&schedule_account.data.borrow())?;
    if !schedule.is_initialized {
        return Err(VestingError::ScheduleNotInitialized.into());
    }
    if !schedule.revocable {
        return Err(VestingError::NotRevocable.into());
    }
    if schedule.status == VestingStatus::Revoked {
        return Err(VestingError::AlreadyRevoked.into());
    }
    if schedule.status == VestingStatus::Completed {
        return Err(VestingError::AlreadyCompleted.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Calculate what has already vested — those tokens remain claimable.
    let vested = calculate_vested_amount(&schedule, now)?;

    // The unvested portion goes back to the vault (already there, just update accounting).
    let unvested = schedule
        .total_amount
        .checked_sub(vested)
        .ok_or(VestingError::Overflow)?;

    // Shrink total_amount to only what has vested.
    schedule.total_amount = vested;
    schedule.status = VestingStatus::Revoked;

    schedule.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    // Update registry.
    let mut registry = VestingRegistry::try_from_slice(&registry_account.data.borrow())?;
    registry.total_revoked = registry
        .total_revoked
        .checked_add(unvested)
        .ok_or(VestingError::Overflow)?;
    registry.active_schedules = registry.active_schedules.saturating_sub(1);

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!(
        "Vesting schedule #{} revoked. Vested: {}, Returned to vault: {}",
        schedule.index,
        vested,
        unvested,
    );
    Ok(())
}

fn process_query_schedule(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let schedule_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    let schedule = VestingSchedule::try_from_slice(&schedule_account.data.borrow())?;
    if !schedule.is_initialized {
        return Err(VestingError::ScheduleNotInitialized.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let vested = calculate_vested_amount(&schedule, now)?;
    let claimable = vested.saturating_sub(schedule.claimed_amount);
    let remaining = schedule.total_amount.saturating_sub(vested);

    let cliff_end = schedule
        .start_timestamp
        .checked_add(schedule.cliff_seconds)
        .unwrap_or(i64::MAX);
    let vesting_end = schedule
        .start_timestamp
        .checked_add(schedule.duration_seconds)
        .unwrap_or(i64::MAX);

    msg!("=== Vesting Schedule #{} ===", schedule.index);
    msg!("Beneficiary:     {}", schedule.beneficiary);
    msg!("Status:          {:?}", schedule.status);
    msg!("Schedule type:   {:?}", schedule.schedule);
    msg!("Revocable:       {}", schedule.revocable);
    msg!("Total amount:    {}", schedule.total_amount);
    msg!("Vested:          {}", vested);
    msg!("Claimed:         {}", schedule.claimed_amount);
    msg!("Claimable now:   {}", claimable);
    msg!("Remaining:       {}", remaining);
    msg!("Start:           {}", schedule.start_timestamp);
    msg!("Cliff ends:      {}", cliff_end);
    msg!("Fully vested:    {}", vesting_end);

    if now < cliff_end {
        let days_until_cliff = (cliff_end - now) / 86_400;
        msg!("Days until cliff: {}", days_until_cliff);
    }

    Ok(())
}

fn process_transfer_authority(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_authority: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let current_authority = next_account_info(account_iter)?;
    let registry_account = next_account_info(account_iter)?;

    if !current_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut registry = VestingRegistry::try_from_slice(&registry_account.data.borrow())?;
    if !registry.is_initialized {
        return Err(VestingError::NotInitialized.into());
    }
    if registry.authority != *current_authority.key {
        return Err(VestingError::Unauthorized.into());
    }

    let old = registry.authority;
    registry.authority = new_authority;

    registry.serialize(&mut &mut registry_account.data.borrow_mut()[..])?;

    msg!("Authority transferred from {} to {}", old, new_authority);
    Ok(())
}

// ---------------------------------------------------------------------------
// Vesting calculation
// ---------------------------------------------------------------------------

/// Calculate the total number of tokens that have vested as of `now`.
fn calculate_vested_amount(
    schedule: &VestingSchedule,
    now: i64,
) -> Result<u64, ProgramError> {
    // If revoked, the total_amount has already been adjusted to the vested amount.
    if schedule.status == VestingStatus::Revoked {
        return Ok(schedule.total_amount);
    }

    let cliff_end = schedule
        .start_timestamp
        .checked_add(schedule.cliff_seconds)
        .ok_or(VestingError::Overflow)?;

    // Before cliff: nothing has vested.
    if now < cliff_end {
        return Ok(0);
    }

    let vesting_end = schedule
        .start_timestamp
        .checked_add(schedule.duration_seconds)
        .ok_or(VestingError::Overflow)?;

    // After full duration: everything has vested.
    if now >= vesting_end {
        return Ok(schedule.total_amount);
    }

    // Duration of the vesting window (after cliff).
    let vesting_window = schedule
        .duration_seconds
        .checked_sub(schedule.cliff_seconds)
        .ok_or(VestingError::Overflow)?;

    if vesting_window <= 0 {
        // Edge case: cliff == duration means everything vests at cliff.
        return Ok(schedule.total_amount);
    }

    let elapsed_after_cliff = now
        .checked_sub(cliff_end)
        .ok_or(VestingError::Overflow)?;

    match schedule.schedule {
        ReleaseSchedule::Linear => {
            // Continuous linear vesting.
            let vested = (schedule.total_amount as u128)
                .checked_mul(elapsed_after_cliff as u128)
                .ok_or(VestingError::Overflow)?
                / vesting_window as u128;
            Ok(vested as u64)
        }
        ReleaseSchedule::Quarterly => {
            // Tokens vest in quarterly chunks.
            let total_quarters = vesting_window / SECONDS_PER_QUARTER;
            if total_quarters <= 0 {
                return Ok(schedule.total_amount);
            }
            let elapsed_quarters = elapsed_after_cliff / SECONDS_PER_QUARTER;
            let quarters_completed = std::cmp::min(elapsed_quarters, total_quarters);

            let vested = (schedule.total_amount as u128)
                .checked_mul(quarters_completed as u128)
                .ok_or(VestingError::Overflow)?
                / total_quarters as u128;
            Ok(vested as u64)
        }
    }
}

// ---------------------------------------------------------------------------
// SPL Token CPI helper
// ---------------------------------------------------------------------------

/// Build an SPL Token `Transfer` instruction manually.
fn spl_token_transfer(
    token_program_id: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let data = {
        let mut buf = Vec::with_capacity(9);
        buf.push(3); // Transfer instruction index
        buf.extend_from_slice(&amount.to_le_bytes());
        buf
    };
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*source, false),
            solana_program::instruction::AccountMeta::new(*destination, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}
