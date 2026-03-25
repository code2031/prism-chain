use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Program ID
// ---------------------------------------------------------------------------

solana_program::declare_id!("FLash111111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Flash loan fee: 0.09% = 9 basis points.
/// Stored as numerator; denominator is 10_000.
const FLASH_LOAN_FEE_BPS: u64 = 9;
const FEE_DENOMINATOR: u64 = 10_000;

/// Maximum number of liquidity providers tracked per pool.
const MAX_PROVIDERS: usize = 256;

/// Seed for pool PDA.
const POOL_SEED: &[u8] = b"flash_pool";

/// Seed for pool vault PDA.
const VAULT_SEED: &[u8] = b"flash_vault";

/// Seed for flash loan receipt PDA (ephemeral within a transaction).
const RECEIPT_SEED: &[u8] = b"flash_receipt";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum FlashLoanError {
    #[error("Pool has already been initialized")]
    AlreadyInitialized,

    #[error("Pool is not initialized")]
    NotInitialized,

    #[error("Insufficient liquidity in the pool")]
    InsufficientLiquidity,

    #[error("Flash loan not repaid: vault balance is less than required")]
    LoanNotRepaid,

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Unauthorized: signer does not match expected authority")]
    Unauthorized,

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Invalid PDA derivation")]
    InvalidPDA,

    #[error("Flash loan receipt already exists (re-entrancy guard)")]
    ReentrancyDetected,

    #[error("No active flash loan to repay")]
    NoActiveLoan,

    #[error("Maximum number of liquidity providers reached")]
    MaxProvidersReached,

    #[error("Provider not found in pool")]
    ProviderNotFound,

    #[error("Insufficient provider balance to withdraw")]
    InsufficientProviderBalance,
}

impl From<FlashLoanError> for ProgramError {
    fn from(e: FlashLoanError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Tracks a single liquidity provider's share in a pool.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ProviderShare {
    /// Provider's wallet address.
    pub owner: Pubkey,
    /// Amount of tokens deposited (in smallest unit).
    pub deposited: u64,
    /// Accumulated fees earned (claimable).
    pub fees_earned: u64,
    /// Timestamp of the last deposit or withdrawal.
    pub last_updated: i64,
}

/// On-chain state for a flash loan liquidity pool.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct FlashLoanPool {
    /// Whether this pool has been initialized.
    pub is_initialized: bool,
    /// Pool admin who can pause/unpause.
    pub admin: Pubkey,
    /// SPL token mint for this pool (Pubkey::default() for native PRISM).
    pub token_mint: Pubkey,
    /// Total liquidity available in the pool.
    pub total_liquidity: u64,
    /// Total fees collected since pool creation.
    pub total_fees_collected: u64,
    /// Total flash loans executed.
    pub total_loans_executed: u64,
    /// Whether the pool is paused (no new loans).
    pub is_paused: bool,
    /// PDA bump seed.
    pub bump: u8,
    /// Number of active providers.
    pub provider_count: u32,
    /// Liquidity provider shares.
    pub providers: Vec<ProviderShare>,
}

/// Ephemeral receipt created at the start of a flash loan and burned on repay.
/// Acts as a re-entrancy guard: only one flash loan active per pool per tx.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct FlashLoanReceipt {
    /// Whether this receipt is active.
    pub is_active: bool,
    /// Pool this loan was borrowed from.
    pub pool: Pubkey,
    /// Borrower.
    pub borrower: Pubkey,
    /// Amount borrowed.
    pub amount: u64,
    /// Fee owed.
    pub fee: u64,
    /// Vault balance at time of borrow (for verification).
    pub vault_balance_before: u64,
    /// PDA bump seed.
    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum FlashLoanInstruction {
    /// Initialize a new flash loan liquidity pool.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Admin (payer)
    ///   1. `[writable]`         Pool PDA
    ///   2. `[writable]`         Vault PDA (holds pool funds)
    ///   3. `[]`                 Token mint (or system program for native)
    ///   4. `[]`                 System program
    InitializePool {
        /// SPL token mint (Pubkey::default() for native PRISM).
        token_mint: Pubkey,
    },

    /// Deposit liquidity into the pool.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Depositor
    ///   1. `[writable]`         Pool PDA
    ///   2. `[writable]`         Vault PDA
    ///   3. `[]`                 System program
    Deposit {
        amount: u64,
    },

    /// Withdraw liquidity and earned fees from the pool.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Provider (withdrawer)
    ///   1. `[writable]`         Pool PDA
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Provider receive account
    ///   4. `[]`                 System program
    Withdraw {
        amount: u64,
    },

    /// Borrow a flash loan. Must be repaid within the same transaction.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Borrower
    ///   1. `[writable]`         Pool PDA
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Receipt PDA (created here)
    ///   4. `[writable]`         Borrower receive account
    ///   5. `[]`                 System program
    Borrow {
        amount: u64,
    },

    /// Repay a flash loan. Must be called in the same transaction as Borrow.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Borrower
    ///   1. `[writable]`         Pool PDA
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Receipt PDA (closed here)
    ///   4. `[]`                 System program
    Repay,
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
    let instruction = FlashLoanInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        FlashLoanInstruction::InitializePool { token_mint } => {
            process_initialize_pool(program_id, accounts, token_mint)
        }
        FlashLoanInstruction::Deposit { amount } => {
            process_deposit(program_id, accounts, amount)
        }
        FlashLoanInstruction::Withdraw { amount } => {
            process_withdraw(program_id, accounts, amount)
        }
        FlashLoanInstruction::Borrow { amount } => {
            process_borrow(program_id, accounts, amount)
        }
        FlashLoanInstruction::Repay => process_repay(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate flash loan fee: amount * 9 / 10_000 (0.09%).
/// Rounds up to ensure the pool never loses value.
fn calculate_fee(amount: u64) -> Result<u64, FlashLoanError> {
    let fee = amount
        .checked_mul(FLASH_LOAN_FEE_BPS)
        .ok_or(FlashLoanError::Overflow)?
        .checked_add(FEE_DENOMINATOR - 1) // round up
        .ok_or(FlashLoanError::Overflow)?
        .checked_div(FEE_DENOMINATOR)
        .ok_or(FlashLoanError::Overflow)?;
    Ok(fee)
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    token_mint: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let _mint_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive and validate pool PDA
    let (pool_pda, pool_bump) = Pubkey::find_program_address(
        &[POOL_SEED, token_mint.as_ref()],
        program_id,
    );
    if pool_pda != *pool_account.key {
        return Err(FlashLoanError::InvalidPDA.into());
    }

    // Derive and validate vault PDA
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, pool_account.key.as_ref()],
        program_id,
    );
    if vault_pda != *vault_account.key {
        return Err(FlashLoanError::InvalidPDA.into());
    }

    // Create pool PDA account
    let rent = Rent::get()?;
    // Pool size is variable due to Vec<ProviderShare>, but we allocate a generous
    // fixed amount to support up to MAX_PROVIDERS entries.
    let space: usize = 1 + 32 + 32 + 8 + 8 + 8 + 1 + 1 + 4 + (MAX_PROVIDERS * (32 + 8 + 8 + 8));
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            pool_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[admin.clone(), pool_account.clone(), system_program.clone()],
        &[&[POOL_SEED, token_mint.as_ref(), &[pool_bump]]],
    )?;

    // Create vault PDA
    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            vault_account.key,
            rent.minimum_balance(0),
            0,
            program_id,
        ),
        &[admin.clone(), vault_account.clone(), system_program.clone()],
        &[&[VAULT_SEED, pool_account.key.as_ref(), &[vault_bump]]],
    )?;

    // Initialize pool state
    let pool = FlashLoanPool {
        is_initialized: true,
        admin: *admin.key,
        token_mint,
        total_liquidity: 0,
        total_fees_collected: 0,
        total_loans_executed: 0,
        is_paused: false,
        bump: pool_bump,
        provider_count: 0,
        providers: Vec::new(),
    };

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Flash loan pool initialized for mint: {}", token_mint);

    Ok(())
}

fn process_deposit(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let depositor = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !depositor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(FlashLoanError::InvalidAmount.into());
    }
    if pool_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut pool = FlashLoanPool::try_from_slice(&pool_account.data.borrow())?;
    if !pool.is_initialized {
        return Err(FlashLoanError::NotInitialized.into());
    }

    // Transfer lamports from depositor to vault
    invoke_signed(
        &system_instruction::transfer(depositor.key, vault_account.key, amount),
        &[depositor.clone(), vault_account.clone(), system_program.clone()],
        &[],
    )?;

    // Update or add provider share
    let clock = Clock::get()?;
    let mut found = false;
    for provider in pool.providers.iter_mut() {
        if provider.owner == *depositor.key {
            provider.deposited = provider
                .deposited
                .checked_add(amount)
                .ok_or(FlashLoanError::Overflow)?;
            provider.last_updated = clock.unix_timestamp;
            found = true;
            break;
        }
    }
    if !found {
        if pool.providers.len() >= MAX_PROVIDERS {
            return Err(FlashLoanError::MaxProvidersReached.into());
        }
        pool.providers.push(ProviderShare {
            owner: *depositor.key,
            deposited: amount,
            fees_earned: 0,
            last_updated: clock.unix_timestamp,
        });
        pool.provider_count = pool.provider_count.checked_add(1).ok_or(FlashLoanError::Overflow)?;
    }

    pool.total_liquidity = pool
        .total_liquidity
        .checked_add(amount)
        .ok_or(FlashLoanError::Overflow)?;

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Deposited {} lamports into flash loan pool", amount);

    Ok(())
}

fn process_withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let provider = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let provider_receive = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !provider.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(FlashLoanError::InvalidAmount.into());
    }
    if pool_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut pool = FlashLoanPool::try_from_slice(&pool_account.data.borrow())?;
    if !pool.is_initialized {
        return Err(FlashLoanError::NotInitialized.into());
    }

    // Find the provider and validate balance
    let clock = Clock::get()?;
    let mut provider_found = false;
    let mut withdraw_amount = amount;

    for ps in pool.providers.iter_mut() {
        if ps.owner == *provider.key {
            provider_found = true;
            // Allow withdrawing deposited + earned fees
            let total_available = ps
                .deposited
                .checked_add(ps.fees_earned)
                .ok_or(FlashLoanError::Overflow)?;
            if amount > total_available {
                return Err(FlashLoanError::InsufficientProviderBalance.into());
            }
            // Deduct from fees first, then deposited
            if amount <= ps.fees_earned {
                ps.fees_earned = ps.fees_earned.checked_sub(amount).unwrap();
            } else {
                let remaining = amount.checked_sub(ps.fees_earned).unwrap();
                ps.fees_earned = 0;
                ps.deposited = ps.deposited.checked_sub(remaining).unwrap();
            }
            ps.last_updated = clock.unix_timestamp;
            break;
        }
    }
    if !provider_found {
        return Err(FlashLoanError::ProviderNotFound.into());
    }

    // Transfer from vault to provider
    let vault_lamports = vault_account.lamports();
    if withdraw_amount > vault_lamports {
        return Err(FlashLoanError::InsufficientLiquidity.into());
    }
    **vault_account.try_borrow_mut_lamports()? = vault_lamports
        .checked_sub(withdraw_amount)
        .ok_or(FlashLoanError::Overflow)?;
    **provider_receive.try_borrow_mut_lamports()? = provider_receive
        .lamports()
        .checked_add(withdraw_amount)
        .ok_or(FlashLoanError::Overflow)?;

    pool.total_liquidity = pool
        .total_liquidity
        .checked_sub(withdraw_amount.min(pool.total_liquidity))
        .ok_or(FlashLoanError::Overflow)?;

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Withdrew {} lamports from flash loan pool", withdraw_amount);

    Ok(())
}

fn process_borrow(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let borrower = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let receipt_account = next_account_info(account_iter)?;
    let borrower_receive = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !borrower.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(FlashLoanError::InvalidAmount.into());
    }
    if pool_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut pool = FlashLoanPool::try_from_slice(&pool_account.data.borrow())?;
    if !pool.is_initialized {
        return Err(FlashLoanError::NotInitialized.into());
    }
    if pool.is_paused {
        return Err(FlashLoanError::InsufficientLiquidity.into());
    }

    // Check liquidity
    let vault_balance = vault_account.lamports();
    if amount > vault_balance {
        return Err(FlashLoanError::InsufficientLiquidity.into());
    }

    // Calculate fee
    let fee = calculate_fee(amount)?;

    // Create receipt PDA (re-entrancy guard)
    let (receipt_pda, receipt_bump) = Pubkey::find_program_address(
        &[RECEIPT_SEED, pool_account.key.as_ref(), borrower.key.as_ref()],
        program_id,
    );
    if receipt_pda != *receipt_account.key {
        return Err(FlashLoanError::InvalidPDA.into());
    }

    let rent = Rent::get()?;
    let receipt_space: usize = 1 + 32 + 32 + 8 + 8 + 8 + 1; // 90 bytes
    let receipt_lamports = rent.minimum_balance(receipt_space);

    invoke_signed(
        &system_instruction::create_account(
            borrower.key,
            receipt_account.key,
            receipt_lamports,
            receipt_space as u64,
            program_id,
        ),
        &[borrower.clone(), receipt_account.clone(), system_program.clone()],
        &[&[
            RECEIPT_SEED,
            pool_account.key.as_ref(),
            borrower.key.as_ref(),
            &[receipt_bump],
        ]],
    )?;

    // Save the receipt
    let receipt = FlashLoanReceipt {
        is_active: true,
        pool: *pool_account.key,
        borrower: *borrower.key,
        amount,
        fee,
        vault_balance_before: vault_balance,
        bump: receipt_bump,
    };
    receipt.serialize(&mut &mut receipt_account.data.borrow_mut()[..])?;

    // Transfer borrowed amount from vault to borrower
    **vault_account.try_borrow_mut_lamports()? = vault_balance
        .checked_sub(amount)
        .ok_or(FlashLoanError::Overflow)?;
    **borrower_receive.try_borrow_mut_lamports()? = borrower_receive
        .lamports()
        .checked_add(amount)
        .ok_or(FlashLoanError::Overflow)?;

    pool.total_loans_executed = pool
        .total_loans_executed
        .checked_add(1)
        .ok_or(FlashLoanError::Overflow)?;
    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!(
        "Flash loan: borrowed {} lamports (fee: {} lamports, 0.09%)",
        amount,
        fee
    );

    Ok(())
}

fn process_repay(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let borrower = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let receipt_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !borrower.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if receipt_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let receipt = FlashLoanReceipt::try_from_slice(&receipt_account.data.borrow())?;
    if !receipt.is_active {
        return Err(FlashLoanError::NoActiveLoan.into());
    }
    if receipt.borrower != *borrower.key {
        return Err(FlashLoanError::Unauthorized.into());
    }
    if receipt.pool != *pool_account.key {
        return Err(FlashLoanError::InvalidPDA.into());
    }

    // Verify repayment: vault must have at least (balance_before + fee)
    let required_balance = receipt
        .vault_balance_before
        .checked_add(receipt.fee)
        .ok_or(FlashLoanError::Overflow)?;
    let current_vault_balance = vault_account.lamports();

    if current_vault_balance < required_balance {
        return Err(FlashLoanError::LoanNotRepaid.into());
    }

    // Distribute fees proportionally to liquidity providers
    let mut pool = FlashLoanPool::try_from_slice(&pool_account.data.borrow())?;
    if pool.total_liquidity > 0 {
        for ps in pool.providers.iter_mut() {
            // Fee share = (provider_deposited / total_liquidity) * fee
            let share = (receipt.fee as u128)
                .checked_mul(ps.deposited as u128)
                .unwrap_or(0)
                .checked_div(pool.total_liquidity as u128)
                .unwrap_or(0) as u64;
            ps.fees_earned = ps.fees_earned.saturating_add(share);
        }
    }

    pool.total_fees_collected = pool
        .total_fees_collected
        .checked_add(receipt.fee)
        .ok_or(FlashLoanError::Overflow)?;
    pool.total_liquidity = pool
        .total_liquidity
        .checked_add(receipt.fee)
        .ok_or(FlashLoanError::Overflow)?;

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    // Close receipt account: return lamports to borrower
    let receipt_lamports = receipt_account.lamports();
    **receipt_account.try_borrow_mut_lamports()? = 0;
    **borrower.try_borrow_mut_lamports()? = borrower
        .lamports()
        .checked_add(receipt_lamports)
        .ok_or(FlashLoanError::Overflow)?;

    msg!(
        "Flash loan repaid: {} + {} fee. Pool liquidity grew.",
        receipt.amount,
        receipt.fee
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
    fn test_fee_calculation() {
        // 0.09% of 1_000_000 = 900, but we round up so: (1_000_000 * 9 + 9999) / 10000 = 900
        assert_eq!(calculate_fee(1_000_000).unwrap(), 900);
        // 0.09% of 100 = 0.09 -> rounds up to 1
        assert_eq!(calculate_fee(100).unwrap(), 1);
        // 0.09% of 0 = 0
        assert_eq!(calculate_fee(0).unwrap(), 0);
        // 0.09% of 10_000 = 9
        assert_eq!(calculate_fee(10_000).unwrap(), 9);
        // 0.09% of 1 = rounds up to 1
        assert_eq!(calculate_fee(1).unwrap(), 1);
        // Large amount: 0.09% of 1_000_000_000_000 (1 trillion)
        assert_eq!(calculate_fee(1_000_000_000_000).unwrap(), 90_000_000);
    }

    #[test]
    fn test_fee_calculation_precision() {
        // Ensure fee is always at least 1 lamport for non-zero amounts
        for amount in 1..=200 {
            let fee = calculate_fee(amount).unwrap();
            assert!(fee >= 1, "Fee for {} should be at least 1, got {}", amount, fee);
        }
    }

    #[test]
    fn test_receipt_size() {
        // Verify receipt struct can serialize within expected bounds
        let receipt = FlashLoanReceipt {
            is_active: true,
            pool: Pubkey::new_unique(),
            borrower: Pubkey::new_unique(),
            amount: u64::MAX,
            fee: u64::MAX,
            vault_balance_before: u64::MAX,
            bump: 255,
        };
        let data = borsh::to_vec(&receipt).unwrap();
        assert!(data.len() <= 90, "Receipt serialized to {} bytes", data.len());
    }
}
