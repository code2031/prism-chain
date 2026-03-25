use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    instruction::Instruction,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for multisig account PDA.
pub const MULTISIG_SEED: &[u8] = b"multisig";

/// Seed for transaction proposal PDA.
pub const TRANSACTION_SEED: &[u8] = b"ms_transaction";

/// Maximum signers in a multisig.
pub const MAX_SIGNERS: usize = 11;

/// Minimum signers in a multisig.
pub const MIN_SIGNERS: usize = 2;

/// Maximum serialized instruction data length.
pub const MAX_INSTRUCTION_DATA_LEN: usize = 1024;

/// Maximum number of accounts in a single instruction.
pub const MAX_INSTRUCTION_ACCOUNTS: usize = 16;

/// Maximum pending transactions per multisig.
pub const MAX_PENDING_TRANSACTIONS: usize = 64;

/// Default time-lock duration (0 = no time-lock).
pub const DEFAULT_TIME_LOCK_SECS: i64 = 0;

/// Maximum time-lock duration: 30 days.
pub const MAX_TIME_LOCK_SECS: i64 = 30 * 24 * 60 * 60;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum MultisigError {
    #[error("Not enough signers (minimum 2)")]
    NotEnoughSigners,
    #[error("Too many signers (maximum 11)")]
    TooManySigners,
    #[error("Threshold exceeds number of signers")]
    ThresholdTooHigh,
    #[error("Threshold must be at least 1")]
    ThresholdTooLow,
    #[error("Not a signer of this multisig")]
    NotASigner,
    #[error("Already approved this transaction")]
    AlreadyApproved,
    #[error("Already rejected this transaction")]
    AlreadyRejected,
    #[error("Transaction threshold not met")]
    ThresholdNotMet,
    #[error("Transaction already executed")]
    AlreadyExecuted,
    #[error("Transaction already cancelled")]
    AlreadyCancelled,
    #[error("Time-lock not elapsed")]
    TimeLockNotElapsed,
    #[error("Time-lock duration too long")]
    TimeLockTooLong,
    #[error("Signer already exists in the multisig")]
    SignerAlreadyExists,
    #[error("Cannot remove signer below threshold")]
    CannotRemoveBelowThreshold,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Multisig already initialized")]
    AlreadyInitialized,
    #[error("Transaction is not pending")]
    TransactionNotPending,
    #[error("Instruction data too large")]
    InstructionDataTooLarge,
    #[error("Too many instruction accounts")]
    TooManyInstructionAccounts,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Transaction was rejected")]
    TransactionRejected,
}

impl From<MultisigError> for ProgramError {
    fn from(e: MultisigError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum TransactionStatus {
    /// Waiting for approvals.
    Pending,
    /// Threshold met, ready to execute (possibly time-locked).
    Approved,
    /// Successfully executed.
    Executed,
    /// Rejected by enough signers.
    Rejected,
    /// Cancelled by the proposer.
    Cancelled,
}

/// An account instruction to be executed by the multisig.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultisigAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

/// A serialized instruction to be executed by the multisig.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultisigInstructionData {
    /// Target program.
    pub program_id: Pubkey,
    /// Accounts needed by the instruction.
    pub accounts: Vec<MultisigAccountMeta>,
    /// Instruction data bytes.
    pub data: Vec<u8>,
}

/// The multisig account (M-of-N).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Multisig {
    pub is_initialized: bool,
    /// Required number of approvals.
    pub threshold: u8,
    /// List of authorized signer pubkeys.
    pub signers: Vec<Pubkey>,
    /// Running transaction counter.
    pub transaction_count: u64,
    /// Time-lock duration in seconds (0 = immediate execution).
    pub time_lock_seconds: i64,
    /// Nonce/label for human identification.
    pub label: [u8; 32],
    /// PDA bump.
    pub bump: u8,
}

impl Multisig {
    pub const MAX_SIZE: usize =
        1 + 1 + (4 + 32 * MAX_SIGNERS) + 8 + 8 + 32 + 1;
}

/// A proposed transaction awaiting approvals.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MultisigTransaction {
    pub is_initialized: bool,
    /// Which multisig this belongs to.
    pub multisig: Pubkey,
    /// Sequential transaction id.
    pub tx_id: u64,
    /// Who proposed it.
    pub proposer: Pubkey,
    /// The instruction to execute.
    pub instruction: MultisigInstructionData,
    /// Current status.
    pub status: TransactionStatus,
    /// Signers who approved.
    pub approvals: Vec<Pubkey>,
    /// Signers who rejected.
    pub rejections: Vec<Pubkey>,
    /// Timestamp when threshold was met (for time-lock).
    pub approved_at: i64,
    /// When the transaction was created.
    pub created_at: i64,
    /// When the transaction was executed (0 if not yet).
    pub executed_at: i64,
    /// PDA bump.
    pub bump: u8,
}

impl MultisigTransaction {
    pub const MAX_SIZE: usize = 1 + 32 + 8 + 32
        + (32 + (4 + 32 * MAX_INSTRUCTION_ACCOUNTS) + (4 + MAX_INSTRUCTION_DATA_LEN))
        + 1
        + (4 + 32 * MAX_SIGNERS)
        + (4 + 32 * MAX_SIGNERS)
        + 8 + 8 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum MultisigInstruction {
    /// Create a new M-of-N multisig account.
    ///
    /// Accounts:
    ///   0. `[writable]` Multisig PDA
    ///   1. `[signer]`   Payer / initial authority
    ///   2. `[]`         System program
    ///   3. `[]`         Rent sysvar
    CreateMultisig {
        threshold: u8,
        signers: Vec<Pubkey>,
        time_lock_seconds: i64,
        label: [u8; 32],
    },

    /// Propose a new transaction for the multisig to execute.
    ///
    /// Accounts:
    ///   0. `[]`         Multisig PDA
    ///   1. `[writable]` Transaction PDA
    ///   2. `[signer]`   Proposer (must be a signer of the multisig)
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    ProposeTransaction {
        instruction: MultisigInstructionData,
    },

    /// Approve a pending transaction.
    ///
    /// Accounts:
    ///   0. `[]`         Multisig PDA
    ///   1. `[writable]` Transaction PDA
    ///   2. `[signer]`   Approver (must be a signer)
    ///   3. `[]`         Clock sysvar
    ApproveTransaction,

    /// Reject a pending transaction.
    ///
    /// Accounts:
    ///   0. `[]`         Multisig PDA
    ///   1. `[writable]` Transaction PDA
    ///   2. `[signer]`   Rejector (must be a signer)
    ///   3. `[]`         Clock sysvar
    RejectTransaction,

    /// Execute a transaction that has met the threshold (and time-lock).
    ///
    /// Accounts:
    ///   0. `[]`         Multisig PDA
    ///   1. `[writable]` Transaction PDA
    ///   2. `[signer]`   Executor (any signer)
    ///   3. `[]`         Clock sysvar
    ///   (+ target program and accounts needed by the instruction)
    ExecuteTransaction,

    /// Cancel a pending transaction (proposer only).
    ///
    /// Accounts:
    ///   0. `[writable]` Transaction PDA
    ///   1. `[signer]`   Proposer
    CancelTransaction,

    /// Add a new signer to the multisig (requires existing threshold approval).
    /// This must be proposed and approved like any other transaction.
    ///
    /// Accounts:
    ///   0. `[writable]` Multisig PDA
    ///   1. `[signer]`   Multisig PDA (self-signed via CPI)
    AddSigner {
        new_signer: Pubkey,
    },

    /// Remove a signer from the multisig (requires existing threshold approval).
    ///
    /// Accounts:
    ///   0. `[writable]` Multisig PDA
    ///   1. `[signer]`   Multisig PDA (self-signed via CPI)
    RemoveSigner {
        signer_to_remove: Pubkey,
    },

    /// Change the approval threshold.
    ///
    /// Accounts:
    ///   0. `[writable]` Multisig PDA
    ///   1. `[signer]`   Multisig PDA (self-signed via CPI)
    ChangeThreshold {
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
    let instruction = MultisigInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MultisigInstruction::CreateMultisig {
            threshold,
            signers,
            time_lock_seconds,
            label,
        } => process_create_multisig(program_id, accounts, threshold, signers, time_lock_seconds, label),

        MultisigInstruction::ProposeTransaction { instruction } => {
            process_propose_transaction(program_id, accounts, instruction)
        }

        MultisigInstruction::ApproveTransaction => {
            process_approve_transaction(program_id, accounts)
        }

        MultisigInstruction::RejectTransaction => {
            process_reject_transaction(program_id, accounts)
        }

        MultisigInstruction::ExecuteTransaction => {
            process_execute_transaction(program_id, accounts)
        }

        MultisigInstruction::CancelTransaction => {
            process_cancel_transaction(program_id, accounts)
        }

        MultisigInstruction::AddSigner { new_signer } => {
            process_add_signer(program_id, accounts, new_signer)
        }

        MultisigInstruction::RemoveSigner { signer_to_remove } => {
            process_remove_signer(program_id, accounts, signer_to_remove)
        }

        MultisigInstruction::ChangeThreshold { new_threshold } => {
            process_change_threshold(program_id, accounts, new_threshold)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn is_signer_of(multisig: &Multisig, key: &Pubkey) -> bool {
    multisig.signers.contains(key)
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_create_multisig(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    threshold: u8,
    signers: Vec<Pubkey>,
    time_lock_seconds: i64,
    label: [u8; 32],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let payer_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate signer count.
    if signers.len() < MIN_SIGNERS {
        return Err(MultisigError::NotEnoughSigners.into());
    }
    if signers.len() > MAX_SIGNERS {
        return Err(MultisigError::TooManySigners.into());
    }

    // Validate threshold.
    if threshold < 1 {
        return Err(MultisigError::ThresholdTooLow.into());
    }
    if threshold as usize > signers.len() {
        return Err(MultisigError::ThresholdTooHigh.into());
    }

    // Validate time-lock.
    if time_lock_seconds > MAX_TIME_LOCK_SECS {
        return Err(MultisigError::TimeLockTooLong.into());
    }

    // Check for duplicate signers.
    let mut sorted = signers.clone();
    sorted.sort();
    for window in sorted.windows(2) {
        if window[0] == window[1] {
            return Err(MultisigError::SignerAlreadyExists.into());
        }
    }

    // Verify PDA.
    let (expected_pda, bump) =
        Pubkey::find_program_address(&[MULTISIG_SEED, &label], program_id);
    if multisig_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check not already initialized.
    if multisig_info.data_len() > 0 {
        let existing = Multisig::try_from_slice(&multisig_info.data.borrow());
        if let Ok(ms) = existing {
            if ms.is_initialized {
                return Err(MultisigError::AlreadyInitialized.into());
            }
        }
    }

    let multisig = Multisig {
        is_initialized: true,
        threshold,
        signers,
        transaction_count: 0,
        time_lock_seconds,
        label,
        bump,
    };

    let data = multisig.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    multisig_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Multisig created: {}-of-{}, time-lock={}s",
        threshold,
        multisig.signers.len(),
        time_lock_seconds
    );
    Ok(())
}

fn process_propose_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: MultisigInstructionData,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let transaction_info = next_account_info(account_iter)?;
    let proposer_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !proposer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate instruction size.
    if instruction.data.len() > MAX_INSTRUCTION_DATA_LEN {
        return Err(MultisigError::InstructionDataTooLarge.into());
    }
    if instruction.accounts.len() > MAX_INSTRUCTION_ACCOUNTS {
        return Err(MultisigError::TooManyInstructionAccounts.into());
    }

    let mut multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if !is_signer_of(&multisig, proposer_info.key) {
        return Err(MultisigError::NotASigner.into());
    }

    let tx_id = multisig.transaction_count;
    multisig.transaction_count = multisig
        .transaction_count
        .checked_add(1)
        .ok_or(MultisigError::Overflow)?;

    let clock = Clock::from_account_info(clock_sysvar)?;

    // Verify transaction PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[
            TRANSACTION_SEED,
            multisig_info.key.as_ref(),
            &tx_id.to_le_bytes(),
        ],
        program_id,
    );
    if transaction_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // The proposer's proposal counts as the first approval.
    let transaction = MultisigTransaction {
        is_initialized: true,
        multisig: *multisig_info.key,
        tx_id,
        proposer: *proposer_info.key,
        instruction,
        status: TransactionStatus::Pending,
        approvals: vec![*proposer_info.key],
        rejections: vec![],
        approved_at: 0,
        created_at: clock.unix_timestamp,
        executed_at: 0,
        bump,
    };

    // Serialize.
    let ms_data = multisig.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    multisig_info.data.borrow_mut()[..ms_data.len()].copy_from_slice(&ms_data);

    let tx_data = transaction.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    transaction_info.data.borrow_mut()[..tx_data.len()].copy_from_slice(&tx_data);

    msg!(
        "Transaction #{} proposed by {} (auto-approved as 1st approval)",
        tx_id,
        proposer_info.key
    );
    Ok(())
}

fn process_approve_transaction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let transaction_info = next_account_info(account_iter)?;
    let approver_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !approver_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if !is_signer_of(&multisig, approver_info.key) {
        return Err(MultisigError::NotASigner.into());
    }

    let mut transaction =
        MultisigTransaction::try_from_slice(&transaction_info.data.borrow())
            .map_err(|_| MultisigError::InvalidAccountData)?;

    if transaction.status != TransactionStatus::Pending {
        return Err(MultisigError::TransactionNotPending.into());
    }

    // Check not already approved by this signer.
    if transaction.approvals.contains(approver_info.key) {
        return Err(MultisigError::AlreadyApproved.into());
    }

    // Check not already rejected by this signer (can't approve after rejecting).
    if transaction.rejections.contains(approver_info.key) {
        return Err(MultisigError::AlreadyRejected.into());
    }

    transaction.approvals.push(*approver_info.key);

    // Check if threshold is now met.
    if transaction.approvals.len() >= multisig.threshold as usize {
        transaction.status = TransactionStatus::Approved;
        let clock = Clock::from_account_info(clock_sysvar)?;
        transaction.approved_at = clock.unix_timestamp;
        msg!(
            "Transaction #{} approved (threshold met: {}/{})",
            transaction.tx_id,
            transaction.approvals.len(),
            multisig.threshold
        );
    } else {
        msg!(
            "Transaction #{} approval: {}/{} needed",
            transaction.tx_id,
            transaction.approvals.len(),
            multisig.threshold
        );
    }

    let data = transaction.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    transaction_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    Ok(())
}

fn process_reject_transaction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let transaction_info = next_account_info(account_iter)?;
    let rejector_info = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !rejector_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if !is_signer_of(&multisig, rejector_info.key) {
        return Err(MultisigError::NotASigner.into());
    }

    let mut transaction =
        MultisigTransaction::try_from_slice(&transaction_info.data.borrow())
            .map_err(|_| MultisigError::InvalidAccountData)?;

    if transaction.status != TransactionStatus::Pending {
        return Err(MultisigError::TransactionNotPending.into());
    }

    if transaction.rejections.contains(rejector_info.key) {
        return Err(MultisigError::AlreadyRejected.into());
    }

    if transaction.approvals.contains(rejector_info.key) {
        return Err(MultisigError::AlreadyApproved.into());
    }

    transaction.rejections.push(*rejector_info.key);

    // If enough signers reject (more than N - threshold), the transaction
    // can never reach threshold, so mark it as rejected.
    let max_possible_approvals = multisig.signers.len() - transaction.rejections.len();
    if max_possible_approvals < multisig.threshold as usize {
        transaction.status = TransactionStatus::Rejected;
        msg!(
            "Transaction #{} rejected (cannot reach threshold)",
            transaction.tx_id
        );
    } else {
        msg!(
            "Transaction #{} rejection: {} reject, {} approve, {} threshold",
            transaction.tx_id,
            transaction.rejections.len(),
            transaction.approvals.len(),
            multisig.threshold
        );
    }

    let data = transaction.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    transaction_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    Ok(())
}

fn process_execute_transaction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let transaction_info = next_account_info(account_iter)?;
    let executor_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !executor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if !is_signer_of(&multisig, executor_info.key) {
        return Err(MultisigError::NotASigner.into());
    }

    let mut transaction =
        MultisigTransaction::try_from_slice(&transaction_info.data.borrow())
            .map_err(|_| MultisigError::InvalidAccountData)?;

    match transaction.status {
        TransactionStatus::Approved => {}
        TransactionStatus::Executed => {
            return Err(MultisigError::AlreadyExecuted.into());
        }
        TransactionStatus::Rejected => {
            return Err(MultisigError::TransactionRejected.into());
        }
        TransactionStatus::Cancelled => {
            return Err(MultisigError::AlreadyCancelled.into());
        }
        TransactionStatus::Pending => {
            return Err(MultisigError::ThresholdNotMet.into());
        }
    }

    // Check time-lock.
    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    if multisig.time_lock_seconds > 0 {
        let unlock_time = transaction
            .approved_at
            .checked_add(multisig.time_lock_seconds)
            .ok_or(MultisigError::Overflow)?;
        if now < unlock_time {
            msg!(
                "Time-lock: {} seconds remaining",
                unlock_time - now
            );
            return Err(MultisigError::TimeLockNotElapsed.into());
        }
    }

    // Build the instruction for CPI.
    let ix_accounts: Vec<_> = transaction
        .instruction
        .accounts
        .iter()
        .map(|meta| solana_program::instruction::AccountMeta {
            pubkey: meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect();

    let ix = Instruction {
        program_id: transaction.instruction.program_id,
        accounts: ix_accounts,
        data: transaction.instruction.data.clone(),
    };

    // Collect remaining accounts for the CPI.
    let remaining_accounts: Vec<AccountInfo> = account_iter.cloned().collect();

    // Execute via CPI with the multisig PDA as signer.
    let signer_seeds: &[&[u8]] = &[
        MULTISIG_SEED,
        &multisig.label,
        &[multisig.bump],
    ];

    invoke_signed(&ix, &remaining_accounts, &[signer_seeds])?;

    transaction.status = TransactionStatus::Executed;
    transaction.executed_at = now;

    let data = transaction.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    transaction_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Transaction #{} executed", transaction.tx_id);
    Ok(())
}

fn process_cancel_transaction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let transaction_info = next_account_info(account_iter)?;
    let proposer_info = next_account_info(account_iter)?;

    if !proposer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut transaction =
        MultisigTransaction::try_from_slice(&transaction_info.data.borrow())
            .map_err(|_| MultisigError::InvalidAccountData)?;

    if transaction.proposer != *proposer_info.key {
        return Err(MultisigError::InvalidAuthority.into());
    }

    if transaction.status != TransactionStatus::Pending {
        return Err(MultisigError::TransactionNotPending.into());
    }

    transaction.status = TransactionStatus::Cancelled;

    let data = transaction.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    transaction_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Transaction #{} cancelled by proposer", transaction.tx_id);
    Ok(())
}

fn process_add_signer(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_signer: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;

    // This instruction should only be callable via the multisig PDA itself
    // (through the execute_transaction flow), so authority must be the PDA.
    if !authority_info.is_signer || authority_info.key != multisig_info.key {
        return Err(MultisigError::InvalidAuthority.into());
    }

    let mut multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if multisig.signers.contains(&new_signer) {
        return Err(MultisigError::SignerAlreadyExists.into());
    }

    if multisig.signers.len() >= MAX_SIGNERS {
        return Err(MultisigError::TooManySigners.into());
    }

    multisig.signers.push(new_signer);

    let data = multisig.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    multisig_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Signer added: {} (now {}-of-{})",
        new_signer,
        multisig.threshold,
        multisig.signers.len()
    );
    Ok(())
}

fn process_remove_signer(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    signer_to_remove: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;

    if !authority_info.is_signer || authority_info.key != multisig_info.key {
        return Err(MultisigError::InvalidAuthority.into());
    }

    let mut multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    let pos = multisig
        .signers
        .iter()
        .position(|s| s == &signer_to_remove)
        .ok_or(MultisigError::NotASigner)?;

    // Cannot remove if it would drop below threshold.
    if multisig.signers.len() - 1 < multisig.threshold as usize {
        return Err(MultisigError::CannotRemoveBelowThreshold.into());
    }

    // Cannot drop below minimum signers.
    if multisig.signers.len() - 1 < MIN_SIGNERS {
        return Err(MultisigError::NotEnoughSigners.into());
    }

    multisig.signers.remove(pos);

    let data = multisig.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    multisig_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Signer removed: {} (now {}-of-{})",
        signer_to_remove,
        multisig.threshold,
        multisig.signers.len()
    );
    Ok(())
}

fn process_change_threshold(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_threshold: u8,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let multisig_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;

    if !authority_info.is_signer || authority_info.key != multisig_info.key {
        return Err(MultisigError::InvalidAuthority.into());
    }

    let mut multisig = Multisig::try_from_slice(&multisig_info.data.borrow())
        .map_err(|_| MultisigError::InvalidAccountData)?;

    if new_threshold < 1 {
        return Err(MultisigError::ThresholdTooLow.into());
    }
    if new_threshold as usize > multisig.signers.len() {
        return Err(MultisigError::ThresholdTooHigh.into());
    }

    let old_threshold = multisig.threshold;
    multisig.threshold = new_threshold;

    let data = multisig.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    multisig_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Threshold changed: {} -> {} (of {} signers)",
        old_threshold,
        new_threshold,
        multisig.signers.len()
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
    fn test_multisig_serialization() {
        let ms = Multisig {
            is_initialized: true,
            threshold: 2,
            signers: vec![
                Pubkey::new_unique(),
                Pubkey::new_unique(),
                Pubkey::new_unique(),
            ],
            transaction_count: 5,
            time_lock_seconds: 3600,
            label: [0xAA; 32],
            bump: 255,
        };
        let data = ms.try_to_vec().unwrap();
        let decoded = Multisig::try_from_slice(&data).unwrap();
        assert_eq!(decoded.threshold, 2);
        assert_eq!(decoded.signers.len(), 3);
        assert_eq!(decoded.transaction_count, 5);
        assert_eq!(decoded.time_lock_seconds, 3600);
    }

    #[test]
    fn test_transaction_serialization() {
        let tx = MultisigTransaction {
            is_initialized: true,
            multisig: Pubkey::new_unique(),
            tx_id: 42,
            proposer: Pubkey::new_unique(),
            instruction: MultisigInstructionData {
                program_id: Pubkey::new_unique(),
                accounts: vec![MultisigAccountMeta {
                    pubkey: Pubkey::new_unique(),
                    is_signer: true,
                    is_writable: true,
                }],
                data: vec![1, 2, 3, 4],
            },
            status: TransactionStatus::Pending,
            approvals: vec![Pubkey::new_unique()],
            rejections: vec![],
            approved_at: 0,
            created_at: 1_700_000_000,
            executed_at: 0,
            bump: 254,
        };
        let data = tx.try_to_vec().unwrap();
        let decoded = MultisigTransaction::try_from_slice(&data).unwrap();
        assert_eq!(decoded.tx_id, 42);
        assert_eq!(decoded.status, TransactionStatus::Pending);
        assert_eq!(decoded.instruction.data, vec![1, 2, 3, 4]);
        assert_eq!(decoded.approvals.len(), 1);
    }

    #[test]
    fn test_is_signer_of() {
        let signer1 = Pubkey::new_unique();
        let signer2 = Pubkey::new_unique();
        let non_signer = Pubkey::new_unique();

        let ms = Multisig {
            is_initialized: true,
            threshold: 2,
            signers: vec![signer1, signer2],
            transaction_count: 0,
            time_lock_seconds: 0,
            label: [0; 32],
            bump: 255,
        };

        assert!(is_signer_of(&ms, &signer1));
        assert!(is_signer_of(&ms, &signer2));
        assert!(!is_signer_of(&ms, &non_signer));
    }

    #[test]
    fn test_rejection_math() {
        // 3-of-5 multisig. If 3 signers reject, threshold can never be reached.
        let threshold: usize = 3;
        let total_signers: usize = 5;
        let rejections: usize = 3;
        let max_possible_approvals = total_signers - rejections;
        assert!(max_possible_approvals < threshold);
    }

    #[test]
    fn test_duplicate_signer_detection() {
        let signer = Pubkey::new_unique();
        let mut signers = vec![signer, Pubkey::new_unique(), signer];
        signers.sort();
        let has_duplicates = signers.windows(2).any(|w| w[0] == w[1]);
        assert!(has_duplicates);
    }

    #[test]
    fn test_time_lock_validation() {
        assert!(DEFAULT_TIME_LOCK_SECS == 0);
        assert!(MAX_TIME_LOCK_SECS == 2_592_000); // 30 days in seconds
        assert!(MAX_TIME_LOCK_SECS > 0);
    }

    #[test]
    fn test_instruction_data_serialization() {
        let ix = MultisigInstructionData {
            program_id: Pubkey::new_unique(),
            accounts: vec![
                MultisigAccountMeta {
                    pubkey: Pubkey::new_unique(),
                    is_signer: false,
                    is_writable: true,
                },
                MultisigAccountMeta {
                    pubkey: Pubkey::new_unique(),
                    is_signer: true,
                    is_writable: false,
                },
            ],
            data: vec![0xFF; 100],
        };
        let data = ix.try_to_vec().unwrap();
        let decoded = MultisigInstructionData::try_from_slice(&data).unwrap();
        assert_eq!(decoded.accounts.len(), 2);
        assert_eq!(decoded.data.len(), 100);
        assert!(decoded.accounts[0].is_writable);
        assert!(decoded.accounts[1].is_signer);
    }
}
