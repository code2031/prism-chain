use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for recovery config PDA.
pub const RECOVERY_CONFIG_SEED: &[u8] = b"recovery_config";

/// Seed for recovery request PDA.
pub const RECOVERY_REQUEST_SEED: &[u8] = b"recovery_request";

/// Seed for guardian approval PDA.
pub const GUARDIAN_APPROVAL_SEED: &[u8] = b"guardian_approval";

/// Minimum number of guardians.
pub const MIN_GUARDIANS: usize = 3;

/// Maximum number of guardians.
pub const MAX_GUARDIANS: usize = 5;

/// Delay before a recovery executes (owner can cancel within this window).
pub const RECOVERY_DELAY_SECS: i64 = 48 * 60 * 60; // 48 hours

/// Delay before a guardian replacement takes effect.
pub const GUARDIAN_REPLACEMENT_DELAY_SECS: i64 = 7 * 24 * 60 * 60; // 7 days

/// Maximum pending recovery requests per account.
pub const MAX_PENDING_REQUESTS: usize = 3;

/// Time after which an unfinished recovery request expires.
pub const RECOVERY_EXPIRY_SECS: i64 = 14 * 24 * 60 * 60; // 14 days

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum RecoveryError {
    #[error("Recovery config already initialized")]
    AlreadyInitialized,
    #[error("Recovery config not initialized")]
    NotInitialized,
    #[error("Too few guardians (minimum {MIN_GUARDIANS})")]
    TooFewGuardians,
    #[error("Too many guardians (maximum {MAX_GUARDIANS})")]
    TooManyGuardians,
    #[error("Not the account owner")]
    NotOwner,
    #[error("Not a guardian of this account")]
    NotGuardian,
    #[error("Recovery delay has not elapsed")]
    DelayNotElapsed,
    #[error("Recovery was cancelled by the owner")]
    RecoveryCancelled,
    #[error("Recovery request has expired")]
    RecoveryExpired,
    #[error("Already approved this recovery")]
    AlreadyApproved,
    #[error("Guardian approval threshold not met")]
    ThresholdNotMet,
    #[error("Guardian already exists")]
    GuardianAlreadyExists,
    #[error("Guardian replacement delay not elapsed")]
    ReplacementDelayNotElapsed,
    #[error("No active recovery request")]
    NoActiveRecovery,
    #[error("Recovery already executed")]
    AlreadyExecuted,
    #[error("Cannot set self as guardian")]
    SelfGuardian,
    #[error("Duplicate guardian in list")]
    DuplicateGuardian,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Invalid recovery request status")]
    InvalidStatus,
    #[error("Guardian replacement already pending")]
    ReplacementAlreadyPending,
    #[error("No pending guardian replacement")]
    NoPendingReplacement,
}

impl From<RecoveryError> for ProgramError {
    fn from(e: RecoveryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum RecoveryStatus {
    /// Recovery has been initiated and is collecting guardian approvals.
    Pending,
    /// Enough guardians approved; waiting for the 48-hour delay.
    Approved,
    /// Recovery was executed; new owner controls the account.
    Executed,
    /// Recovery was cancelled by the original owner.
    Cancelled,
    /// Recovery request expired without enough approvals.
    Expired,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum GuardianReplacementStatus {
    /// Replacement is pending (waiting for delay).
    Pending,
    /// Replacement was executed.
    Executed,
    /// Replacement was cancelled.
    Cancelled,
}

/// Recovery configuration for a single account.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RecoveryConfig {
    pub is_initialized: bool,
    /// The account owner who set up recovery.
    pub owner: Pubkey,
    /// List of guardian public keys (3-5).
    pub guardians: Vec<Pubkey>,
    /// Required number of guardian approvals (majority).
    pub threshold: u8,
    /// Running counter of recovery requests.
    pub request_count: u64,
    /// Pending guardian replacement (only one at a time).
    pub pending_replacement: Option<GuardianReplacement>,
    /// PDA bump.
    pub bump: u8,
}

impl RecoveryConfig {
    pub const MAX_SIZE: usize =
        1 + 32 + (4 + 32 * MAX_GUARDIANS) + 1 + 8 + (1 + 32 + 32 + 8 + 1) + 1;
}

/// A pending guardian replacement.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GuardianReplacement {
    /// The guardian being removed.
    pub old_guardian: Pubkey,
    /// The new guardian being added.
    pub new_guardian: Pubkey,
    /// When the replacement was initiated.
    pub initiated_at: i64,
    /// Status.
    pub status: GuardianReplacementStatus,
}

/// A recovery request initiated by a guardian.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RecoveryRequest {
    pub is_initialized: bool,
    /// The account being recovered.
    pub account_owner: Pubkey,
    /// Sequential request id.
    pub request_id: u64,
    /// The proposed new owner.
    pub new_owner: Pubkey,
    /// Which guardian initiated the recovery.
    pub initiator: Pubkey,
    /// Current status.
    pub status: RecoveryStatus,
    /// Guardians who have approved.
    pub approvals: Vec<Pubkey>,
    /// Required threshold (snapshot from config at creation time).
    pub threshold: u8,
    /// When the request was created.
    pub created_at: i64,
    /// When enough approvals were collected (for delay calculation).
    pub approved_at: i64,
    /// When the recovery was executed.
    pub executed_at: i64,
    /// PDA bump.
    pub bump: u8,
}

impl RecoveryRequest {
    pub const MAX_SIZE: usize =
        1 + 32 + 8 + 32 + 32 + 1 + (4 + 32 * MAX_GUARDIANS) + 1 + 8 + 8 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum RecoveryInstruction {
    /// Set up social recovery for an account (set 3-5 guardians).
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[signer]`   Account owner
    ///   2. `[]`         System program
    ///   3. `[]`         Rent sysvar
    InitializeRecovery {
        guardians: Vec<Pubkey>,
    },

    /// A guardian initiates a recovery request for an account.
    ///
    /// Accounts:
    ///   0. `[]`         Recovery config PDA
    ///   1. `[writable]` Recovery request PDA
    ///   2. `[signer]`   Guardian (initiator)
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    InitiateRecovery {
        new_owner: Pubkey,
    },

    /// A guardian approves an active recovery request.
    ///
    /// Accounts:
    ///   0. `[]`         Recovery config PDA
    ///   1. `[writable]` Recovery request PDA
    ///   2. `[signer]`   Guardian (approver)
    ///   3. `[]`         Clock sysvar
    ApproveRecovery,

    /// Execute a recovery after the delay period has elapsed.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[writable]` Recovery request PDA
    ///   2. `[signer]`   Executor (any guardian or the proposed new owner)
    ///   3. `[]`         Clock sysvar
    ExecuteRecovery,

    /// Owner cancels an active recovery request.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery request PDA
    ///   1. `[signer]`   Account owner
    CancelRecovery,

    /// Owner initiates replacement of a single guardian.
    /// There is a 7-day delay before the replacement takes effect.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[signer]`   Account owner
    ///   2. `[]`         Clock sysvar
    InitiateGuardianReplacement {
        old_guardian: Pubkey,
        new_guardian: Pubkey,
    },

    /// Finalize a guardian replacement after the delay period.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[signer]`   Account owner
    ///   2. `[]`         Clock sysvar
    FinalizeGuardianReplacement,

    /// Cancel a pending guardian replacement.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[signer]`   Account owner
    CancelGuardianReplacement,

    /// Update the recovery threshold (owner only).
    /// The new threshold must be a valid majority of the current guardian count.
    ///
    /// Accounts:
    ///   0. `[writable]` Recovery config PDA
    ///   1. `[signer]`   Account owner
    UpdateThreshold {
        new_threshold: u8,
    },
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
    let instruction = RecoveryInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        RecoveryInstruction::InitializeRecovery { guardians } => {
            process_initialize_recovery(program_id, accounts, guardians)
        }
        RecoveryInstruction::InitiateRecovery { new_owner } => {
            process_initiate_recovery(program_id, accounts, new_owner)
        }
        RecoveryInstruction::ApproveRecovery => {
            process_approve_recovery(program_id, accounts)
        }
        RecoveryInstruction::ExecuteRecovery => {
            process_execute_recovery(program_id, accounts)
        }
        RecoveryInstruction::CancelRecovery => {
            process_cancel_recovery(program_id, accounts)
        }
        RecoveryInstruction::InitiateGuardianReplacement {
            old_guardian,
            new_guardian,
        } => process_initiate_guardian_replacement(program_id, accounts, old_guardian, new_guardian),
        RecoveryInstruction::FinalizeGuardianReplacement => {
            process_finalize_guardian_replacement(program_id, accounts)
        }
        RecoveryInstruction::CancelGuardianReplacement => {
            process_cancel_guardian_replacement(program_id, accounts)
        }
        RecoveryInstruction::UpdateThreshold { new_threshold } => {
            process_update_threshold(program_id, accounts, new_threshold)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate the majority threshold for a given guardian count.
/// For 3 guardians: 2. For 4: 3. For 5: 3.
fn majority_threshold(guardian_count: usize) -> u8 {
    ((guardian_count / 2) + 1) as u8
}

fn is_guardian(config: &RecoveryConfig, key: &Pubkey) -> bool {
    config.guardians.contains(key)
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_recovery(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    guardians: Vec<Pubkey>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate guardian count.
    if guardians.len() < MIN_GUARDIANS {
        return Err(RecoveryError::TooFewGuardians.into());
    }
    if guardians.len() > MAX_GUARDIANS {
        return Err(RecoveryError::TooManyGuardians.into());
    }

    // No self-guardianship.
    for g in &guardians {
        if g == owner_info.key {
            return Err(RecoveryError::SelfGuardian.into());
        }
    }

    // No duplicate guardians.
    let mut sorted = guardians.clone();
    sorted.sort();
    for window in sorted.windows(2) {
        if window[0] == window[1] {
            return Err(RecoveryError::DuplicateGuardian.into());
        }
    }

    // Verify config PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[RECOVERY_CONFIG_SEED, owner_info.key.as_ref()],
        program_id,
    );
    if config_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check not already initialized.
    if config_info.data_len() > 0 {
        let existing = RecoveryConfig::try_from_slice(&config_info.data.borrow());
        if let Ok(cfg) = existing {
            if cfg.is_initialized {
                return Err(RecoveryError::AlreadyInitialized.into());
            }
        }
    }

    let threshold = majority_threshold(guardians.len());

    let config = RecoveryConfig {
        is_initialized: true,
        owner: *owner_info.key,
        guardians,
        threshold,
        request_count: 0,
        pending_replacement: None,
        bump,
    };

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Social recovery initialized: {}-of-{} guardians for {}",
        threshold,
        config.guardians.len(),
        owner_info.key
    );
    Ok(())
}

fn process_initiate_recovery(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_owner: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let request_info = next_account_info(account_iter)?;
    let guardian_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !guardian_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if !config.is_initialized {
        return Err(RecoveryError::NotInitialized.into());
    }

    if !is_guardian(&config, guardian_info.key) {
        return Err(RecoveryError::NotGuardian.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let request_id = config.request_count;
    config.request_count = config
        .request_count
        .checked_add(1)
        .ok_or(RecoveryError::Overflow)?;

    // Verify request PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[
            RECOVERY_REQUEST_SEED,
            config.owner.as_ref(),
            &request_id.to_le_bytes(),
        ],
        program_id,
    );
    if request_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // The initiating guardian's initiation counts as the first approval.
    let request = RecoveryRequest {
        is_initialized: true,
        account_owner: config.owner,
        request_id,
        new_owner,
        initiator: *guardian_info.key,
        status: RecoveryStatus::Pending,
        approvals: vec![*guardian_info.key],
        threshold: config.threshold,
        created_at: now,
        approved_at: 0,
        executed_at: 0,
        bump,
    };

    // Serialize.
    let cfg_data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..cfg_data.len()].copy_from_slice(&cfg_data);

    let req_data = request.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    request_info.data.borrow_mut()[..req_data.len()].copy_from_slice(&req_data);

    msg!(
        "Recovery #{} initiated by guardian {} for account {} (new owner: {})",
        request_id,
        guardian_info.key,
        config.owner,
        new_owner
    );
    Ok(())
}

fn process_approve_recovery(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let request_info = next_account_info(account_iter)?;
    let guardian_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !guardian_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if !is_guardian(&config, guardian_info.key) {
        return Err(RecoveryError::NotGuardian.into());
    }

    let mut request = RecoveryRequest::try_from_slice(&request_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if request.status != RecoveryStatus::Pending {
        return Err(RecoveryError::InvalidStatus.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Check if request has expired.
    let expiry = request
        .created_at
        .checked_add(RECOVERY_EXPIRY_SECS)
        .ok_or(RecoveryError::Overflow)?;
    if now > expiry {
        request.status = RecoveryStatus::Expired;
        let data = request.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
        request_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);
        return Err(RecoveryError::RecoveryExpired.into());
    }

    // Check not already approved by this guardian.
    if request.approvals.contains(guardian_info.key) {
        return Err(RecoveryError::AlreadyApproved.into());
    }

    request.approvals.push(*guardian_info.key);

    // Check if threshold is met.
    if request.approvals.len() >= request.threshold as usize {
        request.status = RecoveryStatus::Approved;
        request.approved_at = now;
        msg!(
            "Recovery #{} fully approved ({}/{}). 48-hour delay started.",
            request.request_id,
            request.approvals.len(),
            request.threshold
        );
    } else {
        msg!(
            "Recovery #{} approval: {}/{} needed",
            request.request_id,
            request.approvals.len(),
            request.threshold
        );
    }

    let data = request.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    request_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    Ok(())
}

fn process_execute_recovery(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let request_info = next_account_info(account_iter)?;
    let executor_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !executor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    let mut request = RecoveryRequest::try_from_slice(&request_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    match request.status {
        RecoveryStatus::Approved => {}
        RecoveryStatus::Executed => {
            return Err(RecoveryError::AlreadyExecuted.into());
        }
        RecoveryStatus::Cancelled => {
            return Err(RecoveryError::RecoveryCancelled.into());
        }
        RecoveryStatus::Expired => {
            return Err(RecoveryError::RecoveryExpired.into());
        }
        RecoveryStatus::Pending => {
            return Err(RecoveryError::ThresholdNotMet.into());
        }
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Check 48-hour delay.
    let execute_after = request
        .approved_at
        .checked_add(RECOVERY_DELAY_SECS)
        .ok_or(RecoveryError::Overflow)?;

    if now < execute_after {
        msg!(
            "Recovery delay: {} seconds remaining",
            execute_after - now
        );
        return Err(RecoveryError::DelayNotElapsed.into());
    }

    // Transfer ownership.
    let old_owner = config.owner;
    config.owner = request.new_owner;
    request.status = RecoveryStatus::Executed;
    request.executed_at = now;

    // Serialize.
    let cfg_data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..cfg_data.len()].copy_from_slice(&cfg_data);

    let req_data = request.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    request_info.data.borrow_mut()[..req_data.len()].copy_from_slice(&req_data);

    msg!(
        "Recovery #{} executed: ownership transferred from {} to {}",
        request.request_id,
        old_owner,
        request.new_owner
    );
    Ok(())
}

fn process_cancel_recovery(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let request_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut request = RecoveryRequest::try_from_slice(&request_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if request.account_owner != *owner_info.key {
        return Err(RecoveryError::NotOwner.into());
    }

    match request.status {
        RecoveryStatus::Pending | RecoveryStatus::Approved => {}
        RecoveryStatus::Executed => {
            return Err(RecoveryError::AlreadyExecuted.into());
        }
        _ => {
            return Err(RecoveryError::InvalidStatus.into());
        }
    }

    request.status = RecoveryStatus::Cancelled;

    let data = request.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    request_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Recovery #{} cancelled by owner {}",
        request.request_id,
        owner_info.key
    );
    Ok(())
}

fn process_initiate_guardian_replacement(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    old_guardian: Pubkey,
    new_guardian: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if config.owner != *owner_info.key {
        return Err(RecoveryError::NotOwner.into());
    }

    // Verify old guardian exists.
    if !config.guardians.contains(&old_guardian) {
        return Err(RecoveryError::NotGuardian.into());
    }

    // Verify new guardian is not already a guardian.
    if config.guardians.contains(&new_guardian) {
        return Err(RecoveryError::GuardianAlreadyExists.into());
    }

    // No self-guardian.
    if new_guardian == *owner_info.key {
        return Err(RecoveryError::SelfGuardian.into());
    }

    // Only one replacement at a time.
    if let Some(ref pending) = config.pending_replacement {
        if pending.status == GuardianReplacementStatus::Pending {
            return Err(RecoveryError::ReplacementAlreadyPending.into());
        }
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    config.pending_replacement = Some(GuardianReplacement {
        old_guardian,
        new_guardian,
        initiated_at: clock.unix_timestamp,
        status: GuardianReplacementStatus::Pending,
    });

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Guardian replacement initiated: {} -> {} (7-day delay)",
        old_guardian,
        new_guardian
    );
    Ok(())
}

fn process_finalize_guardian_replacement(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if config.owner != *owner_info.key {
        return Err(RecoveryError::NotOwner.into());
    }

    let replacement = config
        .pending_replacement
        .as_ref()
        .ok_or(RecoveryError::NoPendingReplacement)?;

    if replacement.status != GuardianReplacementStatus::Pending {
        return Err(RecoveryError::NoPendingReplacement.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let execute_after = replacement
        .initiated_at
        .checked_add(GUARDIAN_REPLACEMENT_DELAY_SECS)
        .ok_or(RecoveryError::Overflow)?;

    if now < execute_after {
        msg!(
            "Guardian replacement delay: {} seconds remaining",
            execute_after - now
        );
        return Err(RecoveryError::ReplacementDelayNotElapsed.into());
    }

    // Perform the swap.
    let old_guardian = replacement.old_guardian;
    let new_guardian = replacement.new_guardian;

    let pos = config
        .guardians
        .iter()
        .position(|g| g == &old_guardian)
        .ok_or(RecoveryError::NotGuardian)?;

    config.guardians[pos] = new_guardian;
    config.pending_replacement = Some(GuardianReplacement {
        old_guardian,
        new_guardian,
        initiated_at: replacement.initiated_at,
        status: GuardianReplacementStatus::Executed,
    });

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Guardian replaced: {} -> {}",
        old_guardian,
        new_guardian
    );
    Ok(())
}

fn process_cancel_guardian_replacement(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if config.owner != *owner_info.key {
        return Err(RecoveryError::NotOwner.into());
    }

    let replacement = config
        .pending_replacement
        .as_ref()
        .ok_or(RecoveryError::NoPendingReplacement)?;

    if replacement.status != GuardianReplacementStatus::Pending {
        return Err(RecoveryError::NoPendingReplacement.into());
    }

    config.pending_replacement = Some(GuardianReplacement {
        old_guardian: replacement.old_guardian,
        new_guardian: replacement.new_guardian,
        initiated_at: replacement.initiated_at,
        status: GuardianReplacementStatus::Cancelled,
    });

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Guardian replacement cancelled");
    Ok(())
}

fn process_update_threshold(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_threshold: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let config_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut config = RecoveryConfig::try_from_slice(&config_info.data.borrow())
        .map_err(|_| RecoveryError::InvalidAccountData)?;

    if config.owner != *owner_info.key {
        return Err(RecoveryError::NotOwner.into());
    }

    // Threshold must be at least majority and at most the total guardian count.
    let min_threshold = majority_threshold(config.guardians.len());
    if new_threshold < min_threshold {
        msg!(
            "Threshold {} is below minimum majority {}",
            new_threshold,
            min_threshold
        );
        return Err(ProgramError::InvalidArgument);
    }
    if new_threshold as usize > config.guardians.len() {
        return Err(ProgramError::InvalidArgument);
    }

    let old_threshold = config.threshold;
    config.threshold = new_threshold;

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    config_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Recovery threshold updated: {} -> {} (of {} guardians)",
        old_threshold,
        new_threshold,
        config.guardians.len()
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majority_threshold() {
        assert_eq!(majority_threshold(3), 2); // 2-of-3
        assert_eq!(majority_threshold(4), 3); // 3-of-4
        assert_eq!(majority_threshold(5), 3); // 3-of-5
    }

    #[test]
    fn test_recovery_config_serialization() {
        let config = RecoveryConfig {
            is_initialized: true,
            owner: Pubkey::new_unique(),
            guardians: vec![
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            ],
            threshold: 2,
            request_count: 0,
            pending_replacement: None,
            bump: 255,
        };
        let data = config.try_to_vec().unwrap();
        let decoded = RecoveryConfig::try_from_slice(&data).unwrap();
        assert_eq!(decoded.guardians.len(), 3);
        assert_eq!(decoded.threshold, 2);
        assert!(decoded.pending_replacement.is_none());
    }

    #[test]
    fn test_recovery_request_serialization() {
        let request = RecoveryRequest {
            is_initialized: true,
            account_owner: Pubkey::new_unique(),
            request_id: 1,
            new_owner: Pubkey::new_unique(),
            initiator: Pubkey::new_unique(),
            status: RecoveryStatus::Pending,
            approvals: vec![Pubkey::new_unique()],
            threshold: 2,
            created_at: 1_700_000_000,
            approved_at: 0,
            executed_at: 0,
            bump: 254,
        };
        let data = request.try_to_vec().unwrap();
        let decoded = RecoveryRequest::try_from_slice(&data).unwrap();
        assert_eq!(decoded.request_id, 1);
        assert_eq!(decoded.status, RecoveryStatus::Pending);
        assert_eq!(decoded.approvals.len(), 1);
    }

    #[test]
    fn test_guardian_replacement_serialization() {
        let replacement = GuardianReplacement {
            old_guardian: Pubkey::new_unique(),
            new_guardian: Pubkey::new_unique(),
            initiated_at: 1_700_000_000,
            status: GuardianReplacementStatus::Pending,
        };
        let data = replacement.try_to_vec().unwrap();
        let decoded = GuardianReplacement::try_from_slice(&data).unwrap();
        assert_eq!(decoded.status, GuardianReplacementStatus::Pending);
    }

    #[test]
    fn test_recovery_delay_constants() {
        assert_eq!(RECOVERY_DELAY_SECS, 172_800); // 48 hours
        assert_eq!(GUARDIAN_REPLACEMENT_DELAY_SECS, 604_800); // 7 days
        assert_eq!(RECOVERY_EXPIRY_SECS, 1_209_600); // 14 days
    }

    #[test]
    fn test_guardian_validation() {
        let owner = Pubkey::new_unique();
        let g1 = Pubkey::new_unique();
        let g2 = Pubkey::new_unique();
        let g3 = Pubkey::new_unique();

        let config = RecoveryConfig {
            is_initialized: true,
            owner,
            guardians: vec![g1, g2, g3],
            threshold: 2,
            request_count: 0,
            pending_replacement: None,
            bump: 255,
        };

        assert!(is_guardian(&config, &g1));
        assert!(is_guardian(&config, &g2));
        assert!(is_guardian(&config, &g3));
        assert!(!is_guardian(&config, &owner));
        assert!(!is_guardian(&config, &Pubkey::new_unique()));
    }

    #[test]
    fn test_config_with_pending_replacement() {
        let config = RecoveryConfig {
            is_initialized: true,
            owner: Pubkey::new_unique(),
            guardians: vec![
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            ],
            threshold: 2,
            request_count: 3,
            pending_replacement: Some(GuardianReplacement {
                old_guardian: Pubkey::new_unique(),
                new_guardian: Pubkey::new_unique(),
                initiated_at: 1_700_000_000,
                status: GuardianReplacementStatus::Pending,
            }),
            bump: 254,
        };
        let data = config.try_to_vec().unwrap();
        let decoded = RecoveryConfig::try_from_slice(&data).unwrap();
        assert!(decoded.pending_replacement.is_some());
        let rep = decoded.pending_replacement.unwrap();
        assert_eq!(rep.status, GuardianReplacementStatus::Pending);
    }

    #[test]
    fn test_recovery_status_transitions() {
        // Valid: Pending -> Approved -> Executed
        let s1 = RecoveryStatus::Pending;
        let s2 = RecoveryStatus::Approved;
        let s3 = RecoveryStatus::Executed;
        assert_ne!(s1, s2);
        assert_ne!(s2, s3);

        // Valid: Pending -> Cancelled
        let s4 = RecoveryStatus::Cancelled;
        assert_ne!(s1, s4);

        // Valid: Pending -> Expired
        let s5 = RecoveryStatus::Expired;
        assert_ne!(s1, s5);
    }

    #[test]
    fn test_duplicate_guardian_detection() {
        let g1 = Pubkey::new_unique();
        let g2 = Pubkey::new_unique();
        let mut guardians = vec![g1, g2, g1]; // duplicate
        guardians.sort();
        let has_dup = guardians.windows(2).any(|w| w[0] == w[1]);
        assert!(has_dup);
    }
}
