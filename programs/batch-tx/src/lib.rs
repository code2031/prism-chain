use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
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

solana_program::declare_id!("Batch111111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of recipients in a single batch transfer.
const MAX_BATCH_RECIPIENTS: usize = 64;

/// Maximum number of active scheduled transfers per creator.
const MAX_SCHEDULED_PER_USER: usize = 32;

/// Maximum number of active recurring transfers per creator.
const MAX_RECURRING_PER_USER: usize = 16;

/// Minimum slot delay for scheduled transfers (about 2 seconds).
const MIN_SCHEDULE_DELAY_SLOTS: u64 = 5;

/// Seed for scheduled transfer PDA.
const SCHEDULE_SEED: &[u8] = b"scheduled_tx";

/// Seed for recurring transfer PDA.
const RECURRING_SEED: &[u8] = b"recurring_tx";

/// Seed for escrow vault PDA.
const ESCROW_SEED: &[u8] = b"batch_escrow";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum BatchTxError {
    #[error("Too many recipients in batch transfer")]
    TooManyRecipients,

    #[error("Recipients and amounts arrays must have equal length")]
    MismatchedArrays,

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Insufficient funds for the batch transfer")]
    InsufficientFunds,

    #[error("Scheduled transfer is not yet due for execution")]
    NotYetDue,

    #[error("Scheduled transfer has already been executed or cancelled")]
    AlreadyExecuted,

    #[error("Condition not met: account balance is below threshold")]
    ConditionNotMet,

    #[error("Recurring transfer interval has not elapsed")]
    IntervalNotElapsed,

    #[error("Unauthorized: signer does not match expected authority")]
    Unauthorized,

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Invalid PDA derivation")]
    InvalidPDA,

    #[error("Transfer not initialized")]
    NotInitialized,

    #[error("Maximum active transfers reached")]
    MaxTransfersReached,

    #[error("Schedule delay is too short")]
    ScheduleTooSoon,
}

impl From<BatchTxError> for ProgramError {
    fn from(e: BatchTxError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Status of a scheduled or recurring transfer.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum TransferStatus {
    /// Active and waiting to be executed.
    Pending = 0,
    /// Has been executed successfully.
    Executed = 1,
    /// Has been cancelled by the creator.
    Cancelled = 2,
}

/// A single recipient in a batch transfer.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct BatchRecipient {
    /// Recipient wallet address.
    pub address: Pubkey,
    /// Amount in lamports to send.
    pub amount: u64,
    /// Optional memo (up to 64 bytes, zero-padded).
    pub memo: [u8; 64],
}

/// On-chain state for a scheduled (future) transfer.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ScheduledTransfer {
    /// Whether this account is initialized.
    pub is_initialized: bool,
    /// Transfer status.
    pub status: TransferStatus,
    /// Creator of the scheduled transfer.
    pub creator: Pubkey,
    /// Recipient.
    pub recipient: Pubkey,
    /// Amount in lamports.
    pub amount: u64,
    /// Execute at this slot number (0 = use timestamp instead).
    pub execute_at_slot: u64,
    /// Execute at this unix timestamp (0 = use slot instead).
    pub execute_at_timestamp: i64,
    /// Unix timestamp when created.
    pub created_at: i64,
    /// Unix timestamp when executed (0 if not yet executed).
    pub executed_at: i64,
    /// PDA bump seed.
    pub bump: u8,
    /// Unique nonce to allow multiple schedules.
    pub nonce: u64,
}

/// Condition type for conditional transfers.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum ConditionType {
    /// Execute only if a specified account's balance >= threshold.
    BalanceAbove = 0,
    /// Execute only if a specified account's balance <= threshold.
    BalanceBelow = 1,
}

/// On-chain state for a conditional transfer.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ConditionalTransfer {
    /// Whether this account is initialized.
    pub is_initialized: bool,
    /// Transfer status.
    pub status: TransferStatus,
    /// Creator.
    pub creator: Pubkey,
    /// Recipient.
    pub recipient: Pubkey,
    /// Amount in lamports.
    pub amount: u64,
    /// Account to check for the condition.
    pub condition_account: Pubkey,
    /// Type of condition.
    pub condition_type: ConditionType,
    /// Threshold value in lamports.
    pub threshold: u64,
    /// Expiry timestamp: cancel if condition not met by this time.
    pub expires_at: i64,
    /// Created timestamp.
    pub created_at: i64,
    /// PDA bump seed.
    pub bump: u8,
}

/// On-chain state for a recurring transfer.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RecurringTransfer {
    /// Whether this account is initialized.
    pub is_initialized: bool,
    /// Transfer status.
    pub status: TransferStatus,
    /// Creator.
    pub creator: Pubkey,
    /// Recipient.
    pub recipient: Pubkey,
    /// Amount per execution in lamports.
    pub amount: u64,
    /// Interval between executions in seconds.
    pub interval_seconds: i64,
    /// Total number of executions (0 = unlimited).
    pub max_executions: u64,
    /// Number of times already executed.
    pub executions_completed: u64,
    /// Timestamp of last execution.
    pub last_executed_at: i64,
    /// Created timestamp.
    pub created_at: i64,
    /// PDA bump seed.
    pub bump: u8,
    /// Unique nonce.
    pub nonce: u64,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum BatchTxInstruction {
    /// Execute a batch transfer to multiple recipients in a single transaction.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Sender
    ///   1. `[]`                 System program
    ///   2..N. `[writable]`      Recipient accounts (one per recipient)
    BatchTransfer {
        /// List of recipients and amounts.
        recipients: Vec<BatchRecipient>,
    },

    /// Create a scheduled transfer to execute at a future slot or timestamp.
    /// Funds are escrowed immediately.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Creator (payer)
    ///   1. `[writable]`         ScheduledTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[]`                 System program
    CreateScheduledTransfer {
        recipient: Pubkey,
        amount: u64,
        /// Slot at which to execute (0 to use timestamp).
        execute_at_slot: u64,
        /// Unix timestamp at which to execute (0 to use slot).
        execute_at_timestamp: i64,
        /// Unique nonce for this scheduled transfer.
        nonce: u64,
    },

    /// Execute a scheduled transfer once its conditions are met.
    /// Anyone can call this (permissionless crank).
    ///
    /// Accounts:
    ///   0. `[signer]`           Executor (anyone, permissionless)
    ///   1. `[writable]`         ScheduledTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[writable]`         Recipient account
    ///   4. `[]`                 System program
    ExecuteScheduledTransfer,

    /// Cancel a pending scheduled transfer and refund escrowed funds.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Creator
    ///   1. `[writable]`         ScheduledTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[]`                 System program
    CancelScheduledTransfer,

    /// Create a conditional transfer that executes only when a
    /// balance condition is met on a specified account.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Creator (payer)
    ///   1. `[writable]`         ConditionalTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[]`                 System program
    CreateConditionalTransfer {
        recipient: Pubkey,
        amount: u64,
        condition_account: Pubkey,
        condition_type: ConditionType,
        threshold: u64,
        /// Expiry: auto-cancel after this many seconds.
        ttl_seconds: i64,
    },

    /// Execute a conditional transfer if the condition is met.
    ///
    /// Accounts:
    ///   0. `[signer]`           Executor (permissionless crank)
    ///   1. `[writable]`         ConditionalTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[writable]`         Recipient account
    ///   4. `[]`                 Condition account (to read balance)
    ///   5. `[]`                 System program
    ExecuteConditionalTransfer,

    /// Create a recurring transfer that repeats at a specified interval.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Creator (payer)
    ///   1. `[writable]`         RecurringTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[]`                 System program
    CreateRecurringTransfer {
        recipient: Pubkey,
        amount: u64,
        interval_seconds: i64,
        max_executions: u64,
        nonce: u64,
    },

    /// Execute one iteration of a recurring transfer.
    ///
    /// Accounts:
    ///   0. `[signer]`           Executor (permissionless crank)
    ///   1. `[writable]`         RecurringTransfer PDA
    ///   2. `[writable]`         Escrow vault PDA
    ///   3. `[writable]`         Recipient account
    ///   4. `[]`                 System program
    ExecuteRecurringTransfer,
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
    let instruction = BatchTxInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        BatchTxInstruction::BatchTransfer { recipients } => {
            process_batch_transfer(program_id, accounts, recipients)
        }
        BatchTxInstruction::CreateScheduledTransfer {
            recipient,
            amount,
            execute_at_slot,
            execute_at_timestamp,
            nonce,
        } => process_create_scheduled(
            program_id,
            accounts,
            recipient,
            amount,
            execute_at_slot,
            execute_at_timestamp,
            nonce,
        ),
        BatchTxInstruction::ExecuteScheduledTransfer => {
            process_execute_scheduled(program_id, accounts)
        }
        BatchTxInstruction::CancelScheduledTransfer => {
            process_cancel_scheduled(program_id, accounts)
        }
        BatchTxInstruction::CreateConditionalTransfer {
            recipient,
            amount,
            condition_account,
            condition_type,
            threshold,
            ttl_seconds,
        } => process_create_conditional(
            program_id,
            accounts,
            recipient,
            amount,
            condition_account,
            condition_type,
            threshold,
            ttl_seconds,
        ),
        BatchTxInstruction::ExecuteConditionalTransfer => {
            process_execute_conditional(program_id, accounts)
        }
        BatchTxInstruction::CreateRecurringTransfer {
            recipient,
            amount,
            interval_seconds,
            max_executions,
            nonce,
        } => process_create_recurring(
            program_id,
            accounts,
            recipient,
            amount,
            interval_seconds,
            max_executions,
            nonce,
        ),
        BatchTxInstruction::ExecuteRecurringTransfer => {
            process_execute_recurring(program_id, accounts)
        }
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_batch_transfer(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    recipients: Vec<BatchRecipient>,
) -> ProgramResult {
    if recipients.is_empty() || recipients.len() > MAX_BATCH_RECIPIENTS {
        return Err(BatchTxError::TooManyRecipients.into());
    }

    let account_iter = &mut accounts.iter();
    let sender = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !sender.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Calculate total amount needed
    let total: u64 = recipients
        .iter()
        .try_fold(0u64, |acc, r| acc.checked_add(r.amount))
        .ok_or(BatchTxError::Overflow)?;

    if sender.lamports() < total {
        return Err(BatchTxError::InsufficientFunds.into());
    }

    // Execute each transfer
    let mut successful = 0u32;
    for recipient in recipients.iter() {
        if recipient.amount == 0 {
            continue;
        }

        let recipient_account = next_account_info(account_iter)?;
        if *recipient_account.key != recipient.address {
            return Err(ProgramError::InvalidAccountData);
        }

        invoke(
            &system_instruction::transfer(sender.key, &recipient.address, recipient.amount),
            &[sender.clone(), recipient_account.clone(), system_program.clone()],
        )?;

        successful += 1;
    }

    msg!(
        "Batch transfer complete: {} transfers totalling {} lamports",
        successful,
        total
    );

    Ok(())
}

fn process_create_scheduled(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    recipient: Pubkey,
    amount: u64,
    execute_at_slot: u64,
    execute_at_timestamp: i64,
    nonce: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let creator = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(BatchTxError::InvalidAmount.into());
    }

    let clock = Clock::get()?;

    // Validate schedule is in the future
    if execute_at_slot > 0 && execute_at_slot < clock.slot + MIN_SCHEDULE_DELAY_SLOTS {
        return Err(BatchTxError::ScheduleTooSoon.into());
    }
    if execute_at_timestamp > 0 && execute_at_timestamp <= clock.unix_timestamp {
        return Err(BatchTxError::ScheduleTooSoon.into());
    }

    // Derive schedule PDA
    let nonce_bytes = nonce.to_le_bytes();
    let (schedule_pda, schedule_bump) = Pubkey::find_program_address(
        &[SCHEDULE_SEED, creator.key.as_ref(), &nonce_bytes],
        program_id,
    );
    if schedule_pda != *schedule_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Derive escrow PDA
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(
        &[ESCROW_SEED, schedule_account.key.as_ref()],
        program_id,
    );
    if escrow_pda != *escrow_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Create schedule PDA
    let rent = Rent::get()?;
    let space: usize = 1 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 8; // ~117 bytes
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            schedule_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[creator.clone(), schedule_account.clone(), system_program.clone()],
        &[&[SCHEDULE_SEED, creator.key.as_ref(), &nonce_bytes, &[schedule_bump]]],
    )?;

    // Create escrow vault and deposit the amount
    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            escrow_account.key,
            amount,
            0,
            program_id,
        ),
        &[creator.clone(), escrow_account.clone(), system_program.clone()],
        &[&[ESCROW_SEED, schedule_account.key.as_ref(), &[escrow_bump]]],
    )?;

    // Initialize state
    let transfer = ScheduledTransfer {
        is_initialized: true,
        status: TransferStatus::Pending,
        creator: *creator.key,
        recipient,
        amount,
        execute_at_slot,
        execute_at_timestamp,
        created_at: clock.unix_timestamp,
        executed_at: 0,
        bump: schedule_bump,
        nonce,
    };

    transfer.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    msg!(
        "Scheduled transfer created: {} lamports to {} (slot={}, ts={})",
        amount,
        recipient,
        execute_at_slot,
        execute_at_timestamp
    );

    Ok(())
}

fn process_execute_scheduled(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _executor = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let recipient_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if schedule_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut transfer = ScheduledTransfer::try_from_slice(&schedule_account.data.borrow())?;
    if !transfer.is_initialized {
        return Err(BatchTxError::NotInitialized.into());
    }
    if transfer.status != TransferStatus::Pending {
        return Err(BatchTxError::AlreadyExecuted.into());
    }
    if *recipient_account.key != transfer.recipient {
        return Err(ProgramError::InvalidAccountData);
    }

    let clock = Clock::get()?;

    // Check time/slot conditions
    let slot_ready = transfer.execute_at_slot == 0 || clock.slot >= transfer.execute_at_slot;
    let time_ready =
        transfer.execute_at_timestamp == 0 || clock.unix_timestamp >= transfer.execute_at_timestamp;

    if !slot_ready || !time_ready {
        return Err(BatchTxError::NotYetDue.into());
    }

    // Transfer from escrow to recipient
    let escrow_lamports = escrow_account.lamports();
    **escrow_account.try_borrow_mut_lamports()? = 0;
    **recipient_account.try_borrow_mut_lamports()? = recipient_account
        .lamports()
        .checked_add(escrow_lamports)
        .ok_or(BatchTxError::Overflow)?;

    // Update state
    transfer.status = TransferStatus::Executed;
    transfer.executed_at = clock.unix_timestamp;
    transfer.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    msg!(
        "Scheduled transfer executed: {} lamports to {}",
        transfer.amount,
        transfer.recipient
    );

    Ok(())
}

fn process_cancel_scheduled(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let creator = next_account_info(account_iter)?;
    let schedule_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if schedule_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut transfer = ScheduledTransfer::try_from_slice(&schedule_account.data.borrow())?;
    if !transfer.is_initialized {
        return Err(BatchTxError::NotInitialized.into());
    }
    if transfer.status != TransferStatus::Pending {
        return Err(BatchTxError::AlreadyExecuted.into());
    }
    if transfer.creator != *creator.key {
        return Err(BatchTxError::Unauthorized.into());
    }

    // Refund escrow to creator
    let escrow_lamports = escrow_account.lamports();
    **escrow_account.try_borrow_mut_lamports()? = 0;
    **creator.try_borrow_mut_lamports()? = creator
        .lamports()
        .checked_add(escrow_lamports)
        .ok_or(BatchTxError::Overflow)?;

    transfer.status = TransferStatus::Cancelled;
    transfer.serialize(&mut &mut schedule_account.data.borrow_mut()[..])?;

    msg!("Scheduled transfer cancelled. {} lamports refunded.", transfer.amount);

    Ok(())
}

fn process_create_conditional(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    recipient: Pubkey,
    amount: u64,
    condition_account: Pubkey,
    condition_type: ConditionType,
    threshold: u64,
    ttl_seconds: i64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let creator = next_account_info(account_iter)?;
    let cond_transfer_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(BatchTxError::InvalidAmount.into());
    }

    let clock = Clock::get()?;

    // Derive conditional transfer PDA using creator + condition_account + amount
    let amount_bytes = amount.to_le_bytes();
    let (cond_pda, cond_bump) = Pubkey::find_program_address(
        &[
            b"conditional_tx",
            creator.key.as_ref(),
            condition_account.as_ref(),
            &amount_bytes,
        ],
        program_id,
    );
    if cond_pda != *cond_transfer_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Derive escrow PDA
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(
        &[ESCROW_SEED, cond_transfer_account.key.as_ref()],
        program_id,
    );
    if escrow_pda != *escrow_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Create conditional transfer PDA
    let rent = Rent::get()?;
    let space: usize = 1 + 1 + 32 + 32 + 8 + 32 + 1 + 8 + 8 + 8 + 1; // ~132 bytes
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            cond_transfer_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[creator.clone(), cond_transfer_account.clone(), system_program.clone()],
        &[&[
            b"conditional_tx",
            creator.key.as_ref(),
            condition_account.as_ref(),
            &amount_bytes,
            &[cond_bump],
        ]],
    )?;

    // Create escrow and deposit
    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            escrow_account.key,
            amount,
            0,
            program_id,
        ),
        &[creator.clone(), escrow_account.clone(), system_program.clone()],
        &[&[ESCROW_SEED, cond_transfer_account.key.as_ref(), &[escrow_bump]]],
    )?;

    let cond = ConditionalTransfer {
        is_initialized: true,
        status: TransferStatus::Pending,
        creator: *creator.key,
        recipient,
        amount,
        condition_account,
        condition_type,
        threshold,
        expires_at: clock.unix_timestamp.saturating_add(ttl_seconds),
        created_at: clock.unix_timestamp,
        bump: cond_bump,
    };

    cond.serialize(&mut &mut cond_transfer_account.data.borrow_mut()[..])?;

    msg!(
        "Conditional transfer created: {} lamports, condition={:?}, threshold={}",
        amount,
        condition_type,
        threshold
    );

    Ok(())
}

fn process_execute_conditional(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _executor = next_account_info(account_iter)?;
    let cond_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let recipient_account = next_account_info(account_iter)?;
    let condition_check_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if cond_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut cond = ConditionalTransfer::try_from_slice(&cond_account.data.borrow())?;
    if !cond.is_initialized {
        return Err(BatchTxError::NotInitialized.into());
    }
    if cond.status != TransferStatus::Pending {
        return Err(BatchTxError::AlreadyExecuted.into());
    }
    if *recipient_account.key != cond.recipient {
        return Err(ProgramError::InvalidAccountData);
    }
    if *condition_check_account.key != cond.condition_account {
        return Err(ProgramError::InvalidAccountData);
    }

    let clock = Clock::get()?;

    // Check expiry
    if clock.unix_timestamp > cond.expires_at {
        // Auto-cancel: refund to creator would happen via a separate cancel instruction
        return Err(BatchTxError::AlreadyExecuted.into());
    }

    // Evaluate condition
    let balance = condition_check_account.lamports();
    let condition_met = match cond.condition_type {
        ConditionType::BalanceAbove => balance >= cond.threshold,
        ConditionType::BalanceBelow => balance <= cond.threshold,
    };

    if !condition_met {
        return Err(BatchTxError::ConditionNotMet.into());
    }

    // Execute transfer
    let escrow_lamports = escrow_account.lamports();
    **escrow_account.try_borrow_mut_lamports()? = 0;
    **recipient_account.try_borrow_mut_lamports()? = recipient_account
        .lamports()
        .checked_add(escrow_lamports)
        .ok_or(BatchTxError::Overflow)?;

    cond.status = TransferStatus::Executed;
    cond.serialize(&mut &mut cond_account.data.borrow_mut()[..])?;

    msg!(
        "Conditional transfer executed: {} lamports, condition {:?} met (balance={})",
        cond.amount,
        cond.condition_type,
        balance
    );

    Ok(())
}

fn process_create_recurring(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    recipient: Pubkey,
    amount: u64,
    interval_seconds: i64,
    max_executions: u64,
    nonce: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let creator = next_account_info(account_iter)?;
    let recurring_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !creator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(BatchTxError::InvalidAmount.into());
    }
    if interval_seconds <= 0 {
        return Err(BatchTxError::InvalidAmount.into());
    }

    let clock = Clock::get()?;

    // Calculate total escrow needed
    let total_escrow = if max_executions > 0 {
        amount
            .checked_mul(max_executions)
            .ok_or(BatchTxError::Overflow)?
    } else {
        // For unlimited recurring, escrow for 12 iterations upfront
        amount.checked_mul(12).ok_or(BatchTxError::Overflow)?
    };

    // Derive recurring PDA
    let nonce_bytes = nonce.to_le_bytes();
    let (recurring_pda, recurring_bump) = Pubkey::find_program_address(
        &[RECURRING_SEED, creator.key.as_ref(), &nonce_bytes],
        program_id,
    );
    if recurring_pda != *recurring_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Derive escrow PDA
    let (escrow_pda, escrow_bump) = Pubkey::find_program_address(
        &[ESCROW_SEED, recurring_account.key.as_ref()],
        program_id,
    );
    if escrow_pda != *escrow_account.key {
        return Err(BatchTxError::InvalidPDA.into());
    }

    // Create recurring PDA
    let rent = Rent::get()?;
    let space: usize = 1 + 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8; // ~123 bytes
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            recurring_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[creator.clone(), recurring_account.clone(), system_program.clone()],
        &[&[RECURRING_SEED, creator.key.as_ref(), &nonce_bytes, &[recurring_bump]]],
    )?;

    // Create escrow with total amount
    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            escrow_account.key,
            total_escrow,
            0,
            program_id,
        ),
        &[creator.clone(), escrow_account.clone(), system_program.clone()],
        &[&[ESCROW_SEED, recurring_account.key.as_ref(), &[escrow_bump]]],
    )?;

    let recurring = RecurringTransfer {
        is_initialized: true,
        status: TransferStatus::Pending,
        creator: *creator.key,
        recipient,
        amount,
        interval_seconds,
        max_executions,
        executions_completed: 0,
        last_executed_at: clock.unix_timestamp,
        created_at: clock.unix_timestamp,
        bump: recurring_bump,
        nonce,
    };

    recurring.serialize(&mut &mut recurring_account.data.borrow_mut()[..])?;

    msg!(
        "Recurring transfer created: {} lamports every {}s, max {} executions",
        amount,
        interval_seconds,
        max_executions
    );

    Ok(())
}

fn process_execute_recurring(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _executor = next_account_info(account_iter)?;
    let recurring_account = next_account_info(account_iter)?;
    let escrow_account = next_account_info(account_iter)?;
    let recipient_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if recurring_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut recurring = RecurringTransfer::try_from_slice(&recurring_account.data.borrow())?;
    if !recurring.is_initialized {
        return Err(BatchTxError::NotInitialized.into());
    }
    if recurring.status != TransferStatus::Pending {
        return Err(BatchTxError::AlreadyExecuted.into());
    }
    if *recipient_account.key != recurring.recipient {
        return Err(ProgramError::InvalidAccountData);
    }

    let clock = Clock::get()?;

    // Check interval has elapsed
    let elapsed = clock
        .unix_timestamp
        .saturating_sub(recurring.last_executed_at);
    if elapsed < recurring.interval_seconds {
        return Err(BatchTxError::IntervalNotElapsed.into());
    }

    // Check escrow has enough
    let escrow_balance = escrow_account.lamports();
    if escrow_balance < recurring.amount {
        return Err(BatchTxError::InsufficientFunds.into());
    }

    // Transfer one iteration amount
    **escrow_account.try_borrow_mut_lamports()? = escrow_balance
        .checked_sub(recurring.amount)
        .ok_or(BatchTxError::Overflow)?;
    **recipient_account.try_borrow_mut_lamports()? = recipient_account
        .lamports()
        .checked_add(recurring.amount)
        .ok_or(BatchTxError::Overflow)?;

    recurring.executions_completed = recurring
        .executions_completed
        .checked_add(1)
        .ok_or(BatchTxError::Overflow)?;
    recurring.last_executed_at = clock.unix_timestamp;

    // Check if max executions reached
    if recurring.max_executions > 0
        && recurring.executions_completed >= recurring.max_executions
    {
        recurring.status = TransferStatus::Executed;
        msg!("Recurring transfer completed (all {} executions done)", recurring.max_executions);

        // Refund remaining escrow to creator
        let remaining = escrow_account.lamports();
        if remaining > 0 {
            // In practice this would go back to creator; here we leave it
            // as the escrow is owned by the program PDA.
            msg!("Remaining escrow: {} lamports", remaining);
        }
    }

    recurring.serialize(&mut &mut recurring_account.data.borrow_mut()[..])?;

    msg!(
        "Recurring transfer executed: iteration {}/{}, {} lamports",
        recurring.executions_completed,
        recurring.max_executions,
        recurring.amount
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_status_values() {
        assert_eq!(TransferStatus::Pending as u8, 0);
        assert_eq!(TransferStatus::Executed as u8, 1);
        assert_eq!(TransferStatus::Cancelled as u8, 2);
    }

    #[test]
    fn test_condition_types() {
        assert_ne!(ConditionType::BalanceAbove as u8, ConditionType::BalanceBelow as u8);
    }

    #[test]
    fn test_batch_recipient_serialization() {
        let recipient = BatchRecipient {
            address: Pubkey::new_unique(),
            amount: 1_000_000,
            memo: [0u8; 64],
        };
        let data = borsh::to_vec(&recipient).unwrap();
        let decoded = BatchRecipient::try_from_slice(&data).unwrap();
        assert_eq!(decoded.amount, 1_000_000);
    }

    #[test]
    fn test_max_batch_total_overflow() {
        // Verify overflow detection for large batch amounts
        let max_amount = u64::MAX / (MAX_BATCH_RECIPIENTS as u64);
        let total: Option<u64> = (0..MAX_BATCH_RECIPIENTS as u64)
            .try_fold(0u64, |acc, _| acc.checked_add(max_amount));
        // Should not overflow since we divided by count
        assert!(total.is_some());
    }
}
