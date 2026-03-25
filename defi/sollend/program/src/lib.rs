use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::Sysvar,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Precision scalar – all basis-point maths use 10 000 as 100 %.
const BPS: u64 = 10_000;

/// Interest-rate model parameters (basis points per year).
const BASE_RATE_BPS: u64 = 200;       // 2 %
const SLOPE1_BPS: u64 = 400;          // 4 % (0 – 80 % utilization)
const SLOPE2_BPS: u64 = 7_500;        // 75 % (80 – 100 % utilization)
const OPTIMAL_UTILIZATION_BPS: u64 = 8_000; // 80 %

/// Risk parameters (basis points).
const LTV_RATIO_BPS: u64 = 8_000;               // 80 %
const LIQUIDATION_THRESHOLD_BPS: u64 = 8_500;    // 85 %
const LIQUIDATION_BONUS_BPS: u64 = 500;          // 5 %

/// Approximate slots per year (400 ms slots).
const SLOTS_PER_YEAR: u64 = 78_840_000;

// ---------------------------------------------------------------------------
// State – Lending Market
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct LendingMarket {
    /// Is this account initialised?
    pub is_initialized: bool,
    /// Authority that can manage the market.
    pub authority: Pubkey,
    /// Token mint this market is for.
    pub token_mint: Pubkey,
    /// Total tokens deposited (lamports / smallest unit).
    pub total_deposits: u64,
    /// Total tokens borrowed.
    pub total_borrows: u64,
    /// Current annualised deposit rate (bps).
    pub deposit_rate: u64,
    /// Current annualised borrow rate (bps).
    pub borrow_rate: u64,
    /// Last slot interest was accrued.
    pub last_update_slot: u64,
    /// Current utilization rate (bps, 0 – 10 000).
    pub utilization_rate: u64,
    /// Loan-to-value ratio (bps).
    pub ltv_ratio: u64,
    /// Liquidation threshold (bps).
    pub liquidation_threshold: u64,
    /// Liquidation bonus (bps).
    pub liquidation_bonus: u64,
    /// Accumulated deposit interest index (scaled by 1e9).
    pub cumulative_deposit_index: u64,
    /// Accumulated borrow interest index (scaled by 1e9).
    pub cumulative_borrow_index: u64,
}

impl LendingMarket {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8;
}

// ---------------------------------------------------------------------------
// State – User Position
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct UserPosition {
    /// Is this account initialised?
    pub is_initialized: bool,
    /// Owner of this position.
    pub owner: Pubkey,
    /// The lending market this position belongs to.
    pub market: Pubkey,
    /// Amount deposited (in token base units).
    pub deposited_amount: u64,
    /// Amount borrowed (in token base units).
    pub borrowed_amount: u64,
    /// Deposit index snapshot at last interaction.
    pub deposit_index_snapshot: u64,
    /// Borrow index snapshot at last interaction.
    pub borrow_index_snapshot: u64,
    /// Computed health factor (scaled by 1e4, i.e. 10 000 = 1.0).
    pub health_factor: u64,
}

impl UserPosition {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8 + 8 + 8 + 8;
}

// ---------------------------------------------------------------------------
// Instruction enum
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum LendingInstruction {
    /// 0 – Initialise a new lending market for a token.
    InitializeMarket,

    /// 1 – Deposit tokens into the market.
    Deposit { amount: u64 },

    /// 2 – Withdraw tokens (plus accrued interest).
    Withdraw { amount: u64 },

    /// 3 – Borrow against existing collateral.
    Borrow { amount: u64 },

    /// 4 – Repay borrowed tokens (plus interest).
    Repay { amount: u64 },

    /// 5 – Liquidate an unhealthy position.
    Liquidate { amount: u64 },

    /// 6 – Accrue interest – anyone may crank this.
    AccrueInterest,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum LendingError {
    #[error("Market already initialized")]
    AlreadyInitialized,
    #[error("Market not initialized")]
    NotInitialized,
    #[error("Insufficient collateral for this borrow")]
    InsufficientCollateral,
    #[error("Position is still healthy – cannot liquidate")]
    PositionHealthy,
    #[error("Insufficient deposited amount")]
    InsufficientDeposit,
    #[error("Repay amount exceeds outstanding debt")]
    RepayExceedsDebt,
    #[error("Arithmetic overflow")]
    MathOverflow,
    #[error("Invalid authority")]
    InvalidAuthority,
}

impl From<LendingError> for ProgramError {
    fn from(e: LendingError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// Helpers – interest-rate model (kinked / two-slope curve)
// ---------------------------------------------------------------------------

/// Calculate the annualised borrow rate in bps given utilization (bps).
///
/// ```text
/// if util <= 80 %:
///     rate = base_rate + (util / optimal) * slope1
/// else:
///     rate = base_rate + slope1 + ((util - optimal) / (1 - optimal)) * slope2
/// ```
pub fn calculate_interest_rate(utilization_bps: u64) -> u64 {
    if utilization_bps == 0 {
        return BASE_RATE_BPS;
    }

    if utilization_bps <= OPTIMAL_UTILIZATION_BPS {
        // Linear portion below kink.
        let variable = SLOPE1_BPS
            .checked_mul(utilization_bps)
            .unwrap_or(u64::MAX)
            / OPTIMAL_UTILIZATION_BPS;
        BASE_RATE_BPS.checked_add(variable).unwrap_or(u64::MAX)
    } else {
        // Above the kink – steep portion.
        let excess = utilization_bps.saturating_sub(OPTIMAL_UTILIZATION_BPS);
        let remaining = BPS.saturating_sub(OPTIMAL_UTILIZATION_BPS); // 2 000 bps
        let steep = SLOPE2_BPS.checked_mul(excess).unwrap_or(u64::MAX) / remaining;
        BASE_RATE_BPS
            .checked_add(SLOPE1_BPS)
            .and_then(|v| v.checked_add(steep))
            .unwrap_or(u64::MAX)
    }
}

/// Utilization = total_borrows / total_deposits (returned in bps).
pub fn calculate_utilization(total_deposits: u64, total_borrows: u64) -> u64 {
    if total_deposits == 0 {
        return 0;
    }
    total_borrows
        .checked_mul(BPS)
        .unwrap_or(u64::MAX)
        / total_deposits
}

/// Health factor = (deposited * liquidation_threshold) / (borrowed * BPS).
/// Returns value scaled by BPS (10 000 = 1.0).
pub fn calculate_health_factor(
    deposited: u64,
    borrowed: u64,
    liquidation_threshold_bps: u64,
) -> u64 {
    if borrowed == 0 {
        return u64::MAX; // infinite health
    }
    deposited
        .checked_mul(liquidation_threshold_bps)
        .unwrap_or(u64::MAX)
        / borrowed
}

/// Apply accrued interest over `slots_elapsed` to a principal.
/// index_new = index_old * (1 + rate_bps / BPS * slots / SLOTS_PER_YEAR)
/// Using 1e9 scale for the index.
fn compound_index(current_index: u64, rate_bps: u64, slots_elapsed: u64) -> u64 {
    // interest_factor = rate_bps * slots_elapsed  (scaled by BPS * SLOTS_PER_YEAR)
    // new_index = current_index + current_index * interest_factor / (BPS * SLOTS_PER_YEAR)
    let numerator = current_index
        .checked_mul(rate_bps)
        .and_then(|v| v.checked_mul(slots_elapsed));
    let denominator = BPS.checked_mul(SLOTS_PER_YEAR);

    match (numerator, denominator) {
        (Some(n), Some(d)) if d > 0 => current_index.saturating_add(n / d),
        _ => current_index,
    }
}

/// Settle a user's position against the latest market indices.
fn settle_user_interest(position: &mut UserPosition, market: &LendingMarket) {
    if position.deposit_index_snapshot > 0 && position.deposited_amount > 0 {
        position.deposited_amount = position
            .deposited_amount
            .checked_mul(market.cumulative_deposit_index)
            .map(|v| v / position.deposit_index_snapshot)
            .unwrap_or(position.deposited_amount);
    }
    if position.borrow_index_snapshot > 0 && position.borrowed_amount > 0 {
        position.borrowed_amount = position
            .borrowed_amount
            .checked_mul(market.cumulative_borrow_index)
            .map(|v| v / position.borrow_index_snapshot)
            .unwrap_or(position.borrowed_amount);
    }
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = LendingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        LendingInstruction::InitializeMarket => initialize_market(program_id, accounts),
        LendingInstruction::Deposit { amount } => deposit(program_id, accounts, amount),
        LendingInstruction::Withdraw { amount } => withdraw(program_id, accounts, amount),
        LendingInstruction::Borrow { amount } => borrow(program_id, accounts, amount),
        LendingInstruction::Repay { amount } => repay(program_id, accounts, amount),
        LendingInstruction::Liquidate { amount } => liquidate(program_id, accounts, amount),
        LendingInstruction::AccrueInterest => accrue_interest(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Instruction handlers
// ---------------------------------------------------------------------------

/// Initialize a new lending market.
///
/// Accounts:
///   0. `[writable]` Market account (uninitialised, allocated off-chain).
///   1. `[signer]`   Authority.
///   2. `[]`         Token mint.
fn initialize_market(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let authority = next_account_info(account_iter)?;
    let token_mint = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())
        .unwrap_or(LendingMarket {
            is_initialized: false,
            authority: Pubkey::default(),
            token_mint: Pubkey::default(),
            total_deposits: 0,
            total_borrows: 0,
            deposit_rate: 0,
            borrow_rate: BASE_RATE_BPS,
            last_update_slot: 0,
            utilization_rate: 0,
            ltv_ratio: LTV_RATIO_BPS,
            liquidation_threshold: LIQUIDATION_THRESHOLD_BPS,
            liquidation_bonus: LIQUIDATION_BONUS_BPS,
            cumulative_deposit_index: 1_000_000_000, // 1e9
            cumulative_borrow_index: 1_000_000_000,
        });

    if market.is_initialized {
        return Err(LendingError::AlreadyInitialized.into());
    }

    let clock = Clock::get()?;

    market.is_initialized = true;
    market.authority = *authority.key;
    market.token_mint = *token_mint.key;
    market.last_update_slot = clock.slot;

    market.serialize(&mut *market_account.data.borrow_mut())?;

    msg!("SolLend: Market initialized for mint {}", token_mint.key);
    Ok(())
}

/// Deposit tokens into the lending market.
///
/// Accounts:
///   0. `[writable]` Market account.
///   1. `[writable]` User position account.
///   2. `[signer]`   Depositor.
fn deposit(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let position_account = next_account_info(account_iter)?;
    let depositor = next_account_info(account_iter)?;

    if !depositor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let mut position = UserPosition::try_from_slice(&position_account.data.borrow())
        .unwrap_or(UserPosition {
            is_initialized: false,
            owner: *depositor.key,
            market: *market_account.key,
            deposited_amount: 0,
            borrowed_amount: 0,
            deposit_index_snapshot: market.cumulative_deposit_index,
            borrow_index_snapshot: market.cumulative_borrow_index,
            health_factor: u64::MAX,
        });

    if !position.is_initialized {
        position.is_initialized = true;
        position.owner = *depositor.key;
        position.market = *market_account.key;
        position.deposit_index_snapshot = market.cumulative_deposit_index;
        position.borrow_index_snapshot = market.cumulative_borrow_index;
    }

    // Settle any previously accrued interest.
    settle_user_interest(&mut position, &market);

    // Credit deposit.
    position.deposited_amount = position
        .deposited_amount
        .checked_add(amount)
        .ok_or(LendingError::MathOverflow)?;

    market.total_deposits = market
        .total_deposits
        .checked_add(amount)
        .ok_or(LendingError::MathOverflow)?;

    // Recalculate rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    // Update health factor.
    position.health_factor = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );

    // Snapshot indices.
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;

    market.serialize(&mut *market_account.data.borrow_mut())?;
    position.serialize(&mut *position_account.data.borrow_mut())?;

    msg!(
        "SolLend: Deposited {} tokens. Total deposits: {}",
        amount,
        market.total_deposits
    );
    Ok(())
}

/// Withdraw tokens from the lending market.
///
/// Accounts:
///   0. `[writable]` Market account.
///   1. `[writable]` User position account.
///   2. `[signer]`   Withdrawer (position owner).
fn withdraw(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let position_account = next_account_info(account_iter)?;
    let withdrawer = next_account_info(account_iter)?;

    if !withdrawer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let mut position = UserPosition::try_from_slice(&position_account.data.borrow())?;
    if position.owner != *withdrawer.key {
        return Err(LendingError::InvalidAuthority.into());
    }

    // Settle interest first.
    settle_user_interest(&mut position, &market);

    if amount > position.deposited_amount {
        return Err(LendingError::InsufficientDeposit.into());
    }

    // Check that withdrawal keeps position healthy.
    let new_deposited = position.deposited_amount.saturating_sub(amount);
    if position.borrowed_amount > 0 {
        let new_health =
            calculate_health_factor(new_deposited, position.borrowed_amount, market.liquidation_threshold);
        if new_health < BPS {
            msg!("SolLend: Withdrawal would make position unhealthy");
            return Err(LendingError::InsufficientCollateral.into());
        }
    }

    position.deposited_amount = new_deposited;
    market.total_deposits = market.total_deposits.saturating_sub(amount);

    // Recalculate rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    position.health_factor = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;

    market.serialize(&mut *market_account.data.borrow_mut())?;
    position.serialize(&mut *position_account.data.borrow_mut())?;

    msg!(
        "SolLend: Withdrew {} tokens. Remaining deposit: {}",
        amount,
        position.deposited_amount
    );
    Ok(())
}

/// Borrow tokens against existing collateral.
///
/// Accounts:
///   0. `[writable]` Market account.
///   1. `[writable]` User position account.
///   2. `[signer]`   Borrower (position owner).
fn borrow(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let position_account = next_account_info(account_iter)?;
    let borrower = next_account_info(account_iter)?;

    if !borrower.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let mut position = UserPosition::try_from_slice(&position_account.data.borrow())?;
    if position.owner != *borrower.key {
        return Err(LendingError::InvalidAuthority.into());
    }

    // Settle interest first.
    settle_user_interest(&mut position, &market);

    // Calculate maximum borrow allowed: deposited * LTV / BPS.
    let max_borrow = position
        .deposited_amount
        .checked_mul(market.ltv_ratio)
        .unwrap_or(0)
        / BPS;
    let new_borrowed = position
        .borrowed_amount
        .checked_add(amount)
        .ok_or(LendingError::MathOverflow)?;

    if new_borrowed > max_borrow {
        msg!(
            "SolLend: Borrow {} would exceed max {} (collateral {} * LTV {})",
            new_borrowed,
            max_borrow,
            position.deposited_amount,
            market.ltv_ratio
        );
        return Err(LendingError::InsufficientCollateral.into());
    }

    position.borrowed_amount = new_borrowed;
    market.total_borrows = market
        .total_borrows
        .checked_add(amount)
        .ok_or(LendingError::MathOverflow)?;

    // Recalculate rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    position.health_factor = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;

    market.serialize(&mut *market_account.data.borrow_mut())?;
    position.serialize(&mut *position_account.data.borrow_mut())?;

    msg!(
        "SolLend: Borrowed {} tokens. Total borrows: {}. Health factor: {}",
        amount,
        market.total_borrows,
        position.health_factor
    );
    Ok(())
}

/// Repay borrowed tokens.
///
/// Accounts:
///   0. `[writable]` Market account.
///   1. `[writable]` User position account.
///   2. `[signer]`   Repayer (position owner).
fn repay(_program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let position_account = next_account_info(account_iter)?;
    let repayer = next_account_info(account_iter)?;

    if !repayer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let mut position = UserPosition::try_from_slice(&position_account.data.borrow())?;
    if position.owner != *repayer.key {
        return Err(LendingError::InvalidAuthority.into());
    }

    // Settle interest first.
    settle_user_interest(&mut position, &market);

    if amount > position.borrowed_amount {
        return Err(LendingError::RepayExceedsDebt.into());
    }

    position.borrowed_amount = position.borrowed_amount.saturating_sub(amount);
    market.total_borrows = market.total_borrows.saturating_sub(amount);

    // Recalculate rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    position.health_factor = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;

    market.serialize(&mut *market_account.data.borrow_mut())?;
    position.serialize(&mut *position_account.data.borrow_mut())?;

    msg!(
        "SolLend: Repaid {} tokens. Remaining debt: {}",
        amount,
        position.borrowed_amount
    );
    Ok(())
}

/// Liquidate an unhealthy position.
///
/// The liquidator repays part of the borrower's debt and receives their
/// collateral at a discount (liquidation bonus).
///
/// Accounts:
///   0. `[writable]` Market account.
///   1. `[writable]` Borrower's position account (unhealthy).
///   2. `[signer]`   Liquidator.
fn liquidate(_program_id: &Pubkey, accounts: &[AccountInfo], repay_amount: u64) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;
    let position_account = next_account_info(account_iter)?;
    let liquidator = next_account_info(account_iter)?;

    if !liquidator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let mut position = UserPosition::try_from_slice(&position_account.data.borrow())?;

    // Settle interest to get latest balances.
    settle_user_interest(&mut position, &market);

    // Check that position is actually unhealthy (health_factor < 1.0 = BPS).
    let health = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );
    if health >= BPS {
        msg!("SolLend: Position health {} is >= 1.0 – cannot liquidate", health);
        return Err(LendingError::PositionHealthy.into());
    }

    // Cap repay to outstanding debt.
    let actual_repay = repay_amount.min(position.borrowed_amount);

    // Collateral seized = repay_amount * (1 + liquidation_bonus).
    let collateral_seized = actual_repay
        .checked_mul(BPS.checked_add(market.liquidation_bonus).unwrap_or(BPS))
        .unwrap_or(u64::MAX)
        / BPS;
    let collateral_seized = collateral_seized.min(position.deposited_amount);

    // Update position.
    position.borrowed_amount = position.borrowed_amount.saturating_sub(actual_repay);
    position.deposited_amount = position.deposited_amount.saturating_sub(collateral_seized);

    // Update market totals.
    market.total_borrows = market.total_borrows.saturating_sub(actual_repay);
    market.total_deposits = market.total_deposits.saturating_sub(collateral_seized);

    // Recalculate rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    position.health_factor = calculate_health_factor(
        position.deposited_amount,
        position.borrowed_amount,
        market.liquidation_threshold,
    );
    position.deposit_index_snapshot = market.cumulative_deposit_index;
    position.borrow_index_snapshot = market.cumulative_borrow_index;

    market.serialize(&mut *market_account.data.borrow_mut())?;
    position.serialize(&mut *position_account.data.borrow_mut())?;

    msg!(
        "SolLend: Liquidated – repaid {} debt, seized {} collateral (bonus {}%)",
        actual_repay,
        collateral_seized,
        market.liquidation_bonus / 100
    );
    Ok(())
}

/// Accrue interest – permissionless crank.
///
/// Accounts:
///   0. `[writable]` Market account.
fn accrue_interest(_program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let market_account = next_account_info(account_iter)?;

    let mut market = LendingMarket::try_from_slice(&market_account.data.borrow())?;
    if !market.is_initialized {
        return Err(LendingError::NotInitialized.into());
    }

    let clock = Clock::get()?;
    let current_slot = clock.slot;

    if current_slot <= market.last_update_slot {
        msg!("SolLend: Interest already up to date");
        return Ok(());
    }

    let slots_elapsed = current_slot.saturating_sub(market.last_update_slot);

    // Update cumulative indices.
    market.cumulative_borrow_index =
        compound_index(market.cumulative_borrow_index, market.borrow_rate, slots_elapsed);
    market.cumulative_deposit_index =
        compound_index(market.cumulative_deposit_index, market.deposit_rate, slots_elapsed);

    // Recalculate interest owed and earned.
    let interest_accrued = market
        .total_borrows
        .checked_mul(market.borrow_rate)
        .and_then(|v| v.checked_mul(slots_elapsed))
        .map(|v| v / (BPS * SLOTS_PER_YEAR))
        .unwrap_or(0);

    market.total_borrows = market.total_borrows.saturating_add(interest_accrued);
    market.total_deposits = market.total_deposits.saturating_add(interest_accrued);

    // Refresh utilization & rates.
    market.utilization_rate = calculate_utilization(market.total_deposits, market.total_borrows);
    market.borrow_rate = calculate_interest_rate(market.utilization_rate);
    market.deposit_rate = market.borrow_rate
        .checked_mul(market.utilization_rate)
        .unwrap_or(0)
        / BPS;

    market.last_update_slot = current_slot;

    market.serialize(&mut *market_account.data.borrow_mut())?;

    msg!(
        "SolLend: Interest accrued – {} slots, borrow rate {} bps, deposit rate {} bps, utilization {} bps",
        slots_elapsed,
        market.borrow_rate,
        market.deposit_rate,
        market.utilization_rate
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_rate_at_zero_utilization() {
        assert_eq!(calculate_interest_rate(0), BASE_RATE_BPS); // 200
    }

    #[test]
    fn test_interest_rate_at_optimal() {
        // At 80% util: base (200) + slope1 (400) = 600 bps = 6%
        let rate = calculate_interest_rate(OPTIMAL_UTILIZATION_BPS);
        assert_eq!(rate, 600);
    }

    #[test]
    fn test_interest_rate_at_full_utilization() {
        // At 100%: base (200) + slope1 (400) + slope2 (7500) = 8100 bps = 81%
        let rate = calculate_interest_rate(BPS);
        assert_eq!(rate, 8100);
    }

    #[test]
    fn test_interest_rate_at_40_percent() {
        // 40% = 4000 bps. Variable = 400 * 4000 / 8000 = 200. Total = 400 bps = 4%
        let rate = calculate_interest_rate(4_000);
        assert_eq!(rate, 400);
    }

    #[test]
    fn test_utilization_50_percent() {
        let util = calculate_utilization(1_000_000, 500_000);
        assert_eq!(util, 5_000); // 50%
    }

    #[test]
    fn test_utilization_zero_deposits() {
        assert_eq!(calculate_utilization(0, 100), 0);
    }

    #[test]
    fn test_health_factor_no_borrows() {
        let hf = calculate_health_factor(1_000, 0, LIQUIDATION_THRESHOLD_BPS);
        assert_eq!(hf, u64::MAX);
    }

    #[test]
    fn test_health_factor_healthy() {
        // deposit 10 000, borrow 5 000, threshold 85%
        // hf = 10000 * 8500 / 5000 = 17000 (1.7 scaled by BPS)
        let hf = calculate_health_factor(10_000, 5_000, LIQUIDATION_THRESHOLD_BPS);
        assert_eq!(hf, 17_000);
    }

    #[test]
    fn test_health_factor_underwater() {
        // deposit 1 000, borrow 900, threshold 85%
        // hf = 1000 * 8500 / 900 = 9444 (< 10000 = 1.0)
        let hf = calculate_health_factor(1_000, 900, LIQUIDATION_THRESHOLD_BPS);
        assert!(hf < BPS);
    }

    #[test]
    fn test_compound_index() {
        let idx = compound_index(1_000_000_000, 600, 78_840_000); // 6% for 1 year
        // Expected: 1e9 + 1e9 * 600 * 78840000 / (10000 * 78840000) = 1e9 + 60_000_000
        assert_eq!(idx, 1_060_000_000);
    }
}
