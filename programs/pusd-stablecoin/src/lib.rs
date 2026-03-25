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

solana_program::declare_id!("PUSD1111111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum collateral ratio: 150% (1.5x). Stored as basis points.
/// User must deposit at least 150% of the PUSD value in PRISM collateral.
const MIN_COLLATERAL_RATIO_BPS: u64 = 15_000; // 150.00%

/// Liquidation threshold: 120% (1.2x). Stored as basis points.
/// If collateral ratio drops below this, the vault can be liquidated.
const LIQUIDATION_THRESHOLD_BPS: u64 = 12_000; // 120.00%

/// Liquidation penalty: 10% of debt value goes to the liquidator.
const LIQUIDATION_PENALTY_BPS: u64 = 1_000; // 10.00%

/// Stability fee: 2% annual interest on minted PUSD.
/// Stored as basis points per year; accrued continuously.
const STABILITY_FEE_ANNUAL_BPS: u64 = 200; // 2.00%

/// Basis points denominator.
const BPS_DENOMINATOR: u64 = 10_000;

/// Seconds in a year (365.25 days).
const SECONDS_PER_YEAR: u64 = 31_557_600;

/// Price precision: PRISM/USD price with 8 decimal places.
const PRICE_DECIMALS: u64 = 100_000_000;

/// Minimum vault debt: 10 PUSD (in smallest unit, 6 decimals).
const MIN_DEBT: u64 = 10_000_000;

/// PUSD decimals (same as USDC).
const PUSD_DECIMALS: u8 = 6;

/// Seed for global config PDA.
const CONFIG_SEED: &[u8] = b"pusd_config";

/// Seed for vault PDA.
const VAULT_SEED: &[u8] = b"pusd_vault";

/// Seed for collateral pool PDA.
const COLLATERAL_SEED: &[u8] = b"pusd_collateral";

/// Seed for PUSD mint authority PDA.
const MINT_AUTH_SEED: &[u8] = b"pusd_mint_auth";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum PusdError {
    #[error("Already initialized")]
    AlreadyInitialized,

    #[error("Not initialized")]
    NotInitialized,

    #[error("Collateral ratio is below the minimum required (150%)")]
    BelowMinCollateralRatio,

    #[error("Vault is not eligible for liquidation (above 120% ratio)")]
    NotLiquidatable,

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Debt is below the minimum vault debt")]
    BelowMinDebt,

    #[error("Cannot withdraw: would bring collateral ratio below minimum")]
    WithdrawalWouldUndercollateralize,

    #[error("Burn amount exceeds outstanding debt")]
    BurnExceedsDebt,

    #[error("Unauthorized: signer does not match vault owner")]
    Unauthorized,

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Invalid PDA derivation")]
    InvalidPDA,

    #[error("Oracle price is stale or invalid")]
    InvalidOraclePrice,

    #[error("Insufficient collateral in vault")]
    InsufficientCollateral,

    #[error("Redemption amount exceeds available PRISM")]
    RedemptionExceedsCollateral,
}

impl From<PusdError> for ProgramError {
    fn from(e: PusdError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Global configuration for the PUSD stablecoin system.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PusdConfig {
    /// Whether the config has been initialized.
    pub is_initialized: bool,
    /// Admin who can update oracle and parameters.
    pub admin: Pubkey,
    /// Oracle price feed account (PRISM/USD).
    pub oracle_feed: Pubkey,
    /// PUSD SPL token mint.
    pub pusd_mint: Pubkey,
    /// Total PUSD in circulation.
    pub total_pusd_supply: u64,
    /// Total PRISM collateral locked across all vaults.
    pub total_collateral_locked: u64,
    /// Total number of active vaults.
    pub total_vaults: u64,
    /// Total stability fees collected.
    pub total_fees_collected: u64,
    /// Whether new vaults can be created.
    pub is_active: bool,
    /// PDA bump seed.
    pub bump: u8,
}

/// A user's collateral vault. Users deposit PRISM and mint PUSD against it.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Vault {
    /// Whether this vault has been initialized.
    pub is_initialized: bool,
    /// Vault owner.
    pub owner: Pubkey,
    /// Amount of PRISM collateral deposited (in lamports).
    pub collateral: u64,
    /// Amount of PUSD debt (in PUSD smallest unit, 6 decimals).
    pub debt: u64,
    /// Accrued stability fee (in PUSD smallest unit).
    pub accrued_fee: u64,
    /// Timestamp of last fee accrual.
    pub last_fee_update: i64,
    /// Timestamp when vault was created.
    pub created_at: i64,
    /// PDA bump seed.
    pub bump: u8,
    /// Unique vault nonce per user (allows multiple vaults).
    pub nonce: u64,
}

impl Vault {
    /// Calculate the collateral ratio in basis points.
    /// collateral_ratio = (collateral_value_usd / debt_usd) * 10000
    /// where collateral_value_usd = collateral_lamports * prism_price / 10^9 (lamports per PRISM)
    /// and debt_usd = debt / 10^6 (PUSD decimals)
    pub fn collateral_ratio_bps(&self, prism_price: u64) -> Result<u64, PusdError> {
        if self.debt == 0 {
            return Ok(u64::MAX); // infinite ratio if no debt
        }

        // collateral value in USD cents (with PRICE_DECIMALS precision):
        // collateral (lamports) * prism_price (with 8 decimals) / 10^9
        let collateral_value = (self.collateral as u128)
            .checked_mul(prism_price as u128)
            .ok_or(PusdError::Overflow)?;

        // debt value scaled to same precision:
        // debt (6 decimals) * 10^9 (lamports) * 10^8 (price decimals) / 10^6 (pusd decimals)
        // = debt * 10^11
        let debt_value = (self.total_debt() as u128)
            .checked_mul(1_000_000_000u128) // lamports per SOL/PRISM
            .ok_or(PusdError::Overflow)?;

        // ratio = (collateral_value / debt_value) * BPS_DENOMINATOR
        let ratio = collateral_value
            .checked_mul(BPS_DENOMINATOR as u128)
            .ok_or(PusdError::Overflow)?
            .checked_div(debt_value)
            .ok_or(PusdError::Overflow)?;

        Ok(ratio as u64)
    }

    /// Total debt including accrued stability fee.
    pub fn total_debt(&self) -> u64 {
        self.debt.saturating_add(self.accrued_fee)
    }

    /// Accrue stability fee based on elapsed time.
    pub fn accrue_fee(&mut self, current_timestamp: i64) {
        if self.debt == 0 || self.last_fee_update >= current_timestamp {
            return;
        }

        let elapsed = (current_timestamp - self.last_fee_update) as u64;

        // fee = debt * annual_rate * elapsed / seconds_per_year / BPS_DENOMINATOR
        // Using u128 to avoid overflow
        let fee = (self.debt as u128)
            .checked_mul(STABILITY_FEE_ANNUAL_BPS as u128)
            .and_then(|v| v.checked_mul(elapsed as u128))
            .and_then(|v| v.checked_div(SECONDS_PER_YEAR as u128))
            .and_then(|v| v.checked_div(BPS_DENOMINATOR as u128))
            .unwrap_or(0) as u64;

        self.accrued_fee = self.accrued_fee.saturating_add(fee);
        self.last_fee_update = current_timestamp;
    }
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum PusdInstruction {
    /// Initialize the PUSD stablecoin system.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Admin (payer)
    ///   1. `[writable]`         PusdConfig PDA
    ///   2. `[]`                 Oracle price feed account
    ///   3. `[]`                 PUSD mint account
    ///   4. `[]`                 System program
    Initialize {
        oracle_feed: Pubkey,
        pusd_mint: Pubkey,
    },

    /// Open a new vault and deposit PRISM collateral.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Owner (payer)
    ///   1. `[writable]`         PusdConfig PDA
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Collateral pool PDA
    ///   4. `[]`                 System program
    OpenVault {
        /// Initial collateral deposit in lamports.
        collateral_amount: u64,
        /// Unique nonce for this vault.
        nonce: u64,
    },

    /// Deposit additional PRISM collateral into an existing vault.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Owner
    ///   1. `[writable]`         Vault PDA
    ///   2. `[writable]`         Collateral pool PDA
    ///   3. `[writable]`         PusdConfig PDA
    ///   4. `[]`                 System program
    DepositCollateral {
        amount: u64,
    },

    /// Mint PUSD against deposited collateral. Requires 150% collateral ratio.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Owner
    ///   1. `[writable]`         Vault PDA
    ///   2. `[writable]`         PusdConfig PDA
    ///   3. `[]`                 Oracle price feed
    ///   4. `[writable]`         PUSD mint
    ///   5. `[writable]`         Owner's PUSD token account
    ///   6. `[]`                 Mint authority PDA
    ///   7. `[]`                 SPL Token program
    MintPusd {
        /// Amount of PUSD to mint (6 decimal places).
        amount: u64,
    },

    /// Burn PUSD to reduce vault debt and unlock collateral.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Owner
    ///   1. `[writable]`         Vault PDA
    ///   2. `[writable]`         PusdConfig PDA
    ///   3. `[writable]`         Owner's PUSD token account
    ///   4. `[writable]`         PUSD mint
    ///   5. `[]`                 SPL Token program
    BurnPusd {
        /// Amount of PUSD to burn (6 decimal places).
        amount: u64,
    },

    /// Withdraw excess PRISM collateral (must maintain 150% ratio).
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Owner
    ///   1. `[writable]`         Vault PDA
    ///   2. `[writable]`         PusdConfig PDA
    ///   3. `[writable]`         Collateral pool PDA
    ///   4. `[]`                 Oracle price feed
    ///   5. `[]`                 System program
    WithdrawCollateral {
        amount: u64,
    },

    /// Liquidate an undercollateralized vault (below 120%).
    /// Liquidator repays the vault's PUSD debt and receives the collateral
    /// at a discount (10% penalty/bonus).
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Liquidator
    ///   1. `[writable]`         Vault PDA
    ///   2. `[writable]`         PusdConfig PDA
    ///   3. `[writable]`         Collateral pool PDA
    ///   4. `[]`                 Oracle price feed
    ///   5. `[writable]`         Liquidator's PUSD token account
    ///   6. `[writable]`         PUSD mint
    ///   7. `[]`                 SPL Token program
    ///   8. `[]`                 System program
    Liquidate,

    /// Redeem PUSD for $1 worth of PRISM collateral.
    /// This is the stability mechanism: if PUSD < $1, arbitrageurs can
    /// buy cheap PUSD and redeem for $1 of PRISM.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Redeemer
    ///   1. `[writable]`         Vault PDA (target vault to redeem from)
    ///   2. `[writable]`         PusdConfig PDA
    ///   3. `[writable]`         Collateral pool PDA
    ///   4. `[]`                 Oracle price feed
    ///   5. `[writable]`         Redeemer's PUSD token account
    ///   6. `[writable]`         PUSD mint
    ///   7. `[]`                 SPL Token program
    ///   8. `[]`                 System program
    Redeem {
        /// Amount of PUSD to redeem (6 decimal places).
        pusd_amount: u64,
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
    let instruction = PusdInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        PusdInstruction::Initialize {
            oracle_feed,
            pusd_mint,
        } => process_initialize(program_id, accounts, oracle_feed, pusd_mint),
        PusdInstruction::OpenVault {
            collateral_amount,
            nonce,
        } => process_open_vault(program_id, accounts, collateral_amount, nonce),
        PusdInstruction::DepositCollateral { amount } => {
            process_deposit_collateral(program_id, accounts, amount)
        }
        PusdInstruction::MintPusd { amount } => {
            process_mint_pusd(program_id, accounts, amount)
        }
        PusdInstruction::BurnPusd { amount } => {
            process_burn_pusd(program_id, accounts, amount)
        }
        PusdInstruction::WithdrawCollateral { amount } => {
            process_withdraw_collateral(program_id, accounts, amount)
        }
        PusdInstruction::Liquidate => process_liquidate(program_id, accounts),
        PusdInstruction::Redeem { pusd_amount } => {
            process_redeem(program_id, accounts, pusd_amount)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read the PRISM/USD price from the oracle feed account.
/// In production, this would deserialize the oracle program's PriceFeed struct.
/// Here we read a simplified format: first 8 bytes = price (u64, 8 decimals).
fn read_oracle_price(oracle_account: &AccountInfo) -> Result<u64, PusdError> {
    let data = oracle_account.data.borrow();
    if data.len() < 8 {
        return Err(PusdError::InvalidOraclePrice);
    }
    let price_bytes: [u8; 8] = data[0..8]
        .try_into()
        .map_err(|_| PusdError::InvalidOraclePrice)?;
    let price = u64::from_le_bytes(price_bytes);
    if price == 0 {
        return Err(PusdError::InvalidOraclePrice);
    }
    Ok(price)
}

/// Calculate how much PRISM collateral corresponds to a given USD amount.
/// prism_amount = usd_amount * 10^9 / prism_price
/// where usd_amount is in PUSD (6 decimals) and prism_price has 8 decimals.
fn usd_to_prism(usd_amount: u64, prism_price: u64) -> Result<u64, PusdError> {
    // usd_amount (6 dec) * 10^9 (lamports/PRISM) * 10^8 (price dec) / 10^6 (usd dec) / prism_price
    // = usd_amount * 10^11 / prism_price
    let prism = (usd_amount as u128)
        .checked_mul(100_000_000_000u128) // 10^11
        .ok_or(PusdError::Overflow)?
        .checked_div(prism_price as u128)
        .ok_or(PusdError::Overflow)?;
    Ok(prism as u64)
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    oracle_feed: Pubkey,
    pusd_mint: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let _oracle_account = next_account_info(account_iter)?;
    let _mint_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (config_pda, config_bump) = Pubkey::find_program_address(
        &[CONFIG_SEED],
        program_id,
    );
    if config_pda != *config_account.key {
        return Err(PusdError::InvalidPDA.into());
    }

    let rent = Rent::get()?;
    let space: usize = 1 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1; // ~131 bytes
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            config_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[admin.clone(), config_account.clone(), system_program.clone()],
        &[&[CONFIG_SEED, &[config_bump]]],
    )?;

    let config = PusdConfig {
        is_initialized: true,
        admin: *admin.key,
        oracle_feed,
        pusd_mint,
        total_pusd_supply: 0,
        total_collateral_locked: 0,
        total_vaults: 0,
        total_fees_collected: 0,
        is_active: true,
        bump: config_bump,
    };

    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("PUSD stablecoin system initialized");
    msg!("Oracle: {}, Mint: {}", oracle_feed, pusd_mint);

    Ok(())
}

fn process_open_vault(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    collateral_amount: u64,
    nonce: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if collateral_amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if config_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    if !config.is_initialized || !config.is_active {
        return Err(PusdError::NotInitialized.into());
    }

    // Derive vault PDA
    let nonce_bytes = nonce.to_le_bytes();
    let (vault_pda, vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, owner.key.as_ref(), &nonce_bytes],
        program_id,
    );
    if vault_pda != *vault_account.key {
        return Err(PusdError::InvalidPDA.into());
    }

    // Derive collateral pool PDA
    let (collateral_pda, collateral_bump) = Pubkey::find_program_address(
        &[COLLATERAL_SEED, vault_account.key.as_ref()],
        program_id,
    );
    if collateral_pda != *collateral_pool.key {
        return Err(PusdError::InvalidPDA.into());
    }

    let clock = Clock::get()?;
    let rent = Rent::get()?;

    // Create vault PDA
    let vault_space: usize = 1 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 8; // ~82 bytes
    let vault_lamports = rent.minimum_balance(vault_space);

    invoke_signed(
        &system_instruction::create_account(
            owner.key,
            vault_account.key,
            vault_lamports,
            vault_space as u64,
            program_id,
        ),
        &[owner.clone(), vault_account.clone(), system_program.clone()],
        &[&[VAULT_SEED, owner.key.as_ref(), &nonce_bytes, &[vault_bump]]],
    )?;

    // Create collateral pool and deposit
    invoke_signed(
        &system_instruction::create_account(
            owner.key,
            collateral_pool.key,
            collateral_amount,
            0,
            program_id,
        ),
        &[owner.clone(), collateral_pool.clone(), system_program.clone()],
        &[&[COLLATERAL_SEED, vault_account.key.as_ref(), &[collateral_bump]]],
    )?;

    // Initialize vault state
    let vault = Vault {
        is_initialized: true,
        owner: *owner.key,
        collateral: collateral_amount,
        debt: 0,
        accrued_fee: 0,
        last_fee_update: clock.unix_timestamp,
        created_at: clock.unix_timestamp,
        bump: vault_bump,
        nonce,
    };

    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    // Update global config
    config.total_collateral_locked = config
        .total_collateral_locked
        .checked_add(collateral_amount)
        .ok_or(PusdError::Overflow)?;
    config.total_vaults = config
        .total_vaults
        .checked_add(1)
        .ok_or(PusdError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!(
        "Vault opened: {} PRISM collateral deposited (nonce={})",
        collateral_amount,
        nonce
    );

    Ok(())
}

fn process_deposit_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }
    if vault.owner != *owner.key {
        return Err(PusdError::Unauthorized.into());
    }

    // Transfer lamports to collateral pool
    invoke_signed(
        &system_instruction::transfer(owner.key, collateral_pool.key, amount),
        &[owner.clone(), collateral_pool.clone(), system_program.clone()],
        &[],
    )?;

    vault.collateral = vault
        .collateral
        .checked_add(amount)
        .ok_or(PusdError::Overflow)?;
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    // Update global config
    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_collateral_locked = config
        .total_collateral_locked
        .checked_add(amount)
        .ok_or(PusdError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Deposited {} additional PRISM collateral", amount);

    Ok(())
}

fn process_mint_pusd(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let oracle_account = next_account_info(account_iter)?;
    let _pusd_mint = next_account_info(account_iter)?;
    let _owner_pusd_account = next_account_info(account_iter)?;
    let _mint_authority = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }
    if vault.owner != *owner.key {
        return Err(PusdError::Unauthorized.into());
    }

    let clock = Clock::get()?;

    // Accrue stability fee before modifying debt
    vault.accrue_fee(clock.unix_timestamp);

    // Calculate new debt
    let new_debt = vault
        .debt
        .checked_add(amount)
        .ok_or(PusdError::Overflow)?;

    // Read oracle price
    let prism_price = read_oracle_price(oracle_account)?;

    // Check collateral ratio with new debt
    let temp_vault = Vault {
        debt: new_debt,
        ..vault.clone()
    };
    let ratio = temp_vault.collateral_ratio_bps(prism_price)?;
    if ratio < MIN_COLLATERAL_RATIO_BPS {
        msg!(
            "Collateral ratio {} bps is below minimum {} bps",
            ratio,
            MIN_COLLATERAL_RATIO_BPS
        );
        return Err(PusdError::BelowMinCollateralRatio.into());
    }

    // Check minimum debt
    if new_debt < MIN_DEBT {
        return Err(PusdError::BelowMinDebt.into());
    }

    // Update vault debt
    vault.debt = new_debt;
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    // Update global supply
    // In production, this would also invoke the SPL Token mint_to instruction
    // to actually mint PUSD tokens to the owner's token account.
    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_pusd_supply = config
        .total_pusd_supply
        .checked_add(amount)
        .ok_or(PusdError::Overflow)?;
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!(
        "Minted {} PUSD (collateral ratio: {} bps, price: {})",
        amount,
        ratio,
        prism_price
    );

    Ok(())
}

fn process_burn_pusd(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let _owner_pusd_account = next_account_info(account_iter)?;
    let _pusd_mint = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }
    if vault.owner != *owner.key {
        return Err(PusdError::Unauthorized.into());
    }

    let clock = Clock::get()?;
    vault.accrue_fee(clock.unix_timestamp);

    // Apply burn: first to accrued fee, then to principal debt
    let total = vault.total_debt();
    if amount > total {
        return Err(PusdError::BurnExceedsDebt.into());
    }

    if amount <= vault.accrued_fee {
        vault.accrued_fee = vault.accrued_fee.checked_sub(amount).unwrap();
    } else {
        let remaining = amount.checked_sub(vault.accrued_fee).unwrap();
        vault.accrued_fee = 0;
        vault.debt = vault.debt.checked_sub(remaining).unwrap();
    }

    // Check minimum debt if not fully closing
    if vault.debt > 0 && vault.debt < MIN_DEBT {
        return Err(PusdError::BelowMinDebt.into());
    }

    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    // In production, invoke SPL Token burn_checked here
    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_pusd_supply = config
        .total_pusd_supply
        .saturating_sub(amount);
    config.total_fees_collected = config
        .total_fees_collected
        .saturating_add(amount.min(vault.accrued_fee.saturating_add(amount))); // fees portion
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Burned {} PUSD. Remaining debt: {}", amount, vault.total_debt());

    Ok(())
}

fn process_withdraw_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let oracle_account = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }
    if vault.owner != *owner.key {
        return Err(PusdError::Unauthorized.into());
    }
    if amount > vault.collateral {
        return Err(PusdError::InsufficientCollateral.into());
    }

    let clock = Clock::get()?;
    vault.accrue_fee(clock.unix_timestamp);

    // If there's outstanding debt, check ratio after withdrawal
    if vault.total_debt() > 0 {
        let prism_price = read_oracle_price(oracle_account)?;
        let new_collateral = vault
            .collateral
            .checked_sub(amount)
            .ok_or(PusdError::Overflow)?;

        let temp_vault = Vault {
            collateral: new_collateral,
            ..vault.clone()
        };
        let ratio = temp_vault.collateral_ratio_bps(prism_price)?;
        if ratio < MIN_COLLATERAL_RATIO_BPS {
            return Err(PusdError::WithdrawalWouldUndercollateralize.into());
        }
    }

    // Transfer collateral back to owner
    let pool_balance = collateral_pool.lamports();
    if amount > pool_balance {
        return Err(PusdError::InsufficientCollateral.into());
    }
    **collateral_pool.try_borrow_mut_lamports()? = pool_balance
        .checked_sub(amount)
        .ok_or(PusdError::Overflow)?;
    **owner.try_borrow_mut_lamports()? = owner
        .lamports()
        .checked_add(amount)
        .ok_or(PusdError::Overflow)?;

    vault.collateral = vault.collateral.checked_sub(amount).unwrap();
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_collateral_locked = config
        .total_collateral_locked
        .saturating_sub(amount);
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!("Withdrew {} PRISM collateral from vault", amount);

    Ok(())
}

fn process_liquidate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let liquidator = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let oracle_account = next_account_info(account_iter)?;
    let _liquidator_pusd = next_account_info(account_iter)?;
    let _pusd_mint = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !liquidator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }

    let clock = Clock::get()?;
    vault.accrue_fee(clock.unix_timestamp);

    let prism_price = read_oracle_price(oracle_account)?;
    let ratio = vault.collateral_ratio_bps(prism_price)?;

    // Must be below liquidation threshold
    if ratio >= LIQUIDATION_THRESHOLD_BPS {
        msg!("Vault collateral ratio {} bps is above liquidation threshold {} bps",
            ratio, LIQUIDATION_THRESHOLD_BPS);
        return Err(PusdError::NotLiquidatable.into());
    }

    let total_debt = vault.total_debt();

    // Calculate collateral to seize: debt_value_in_prism * (1 + penalty)
    let debt_in_prism = usd_to_prism(total_debt, prism_price)?;
    let penalty = (debt_in_prism as u128)
        .checked_mul(LIQUIDATION_PENALTY_BPS as u128)
        .ok_or(PusdError::Overflow)?
        .checked_div(BPS_DENOMINATOR as u128)
        .ok_or(PusdError::Overflow)? as u64;
    let total_seize = debt_in_prism.saturating_add(penalty).min(vault.collateral);

    // In production: burn liquidator's PUSD via SPL Token burn
    // Transfer collateral from pool to liquidator
    let pool_balance = collateral_pool.lamports();
    let seize_amount = total_seize.min(pool_balance);
    **collateral_pool.try_borrow_mut_lamports()? = pool_balance
        .checked_sub(seize_amount)
        .ok_or(PusdError::Overflow)?;
    **liquidator.try_borrow_mut_lamports()? = liquidator
        .lamports()
        .checked_add(seize_amount)
        .ok_or(PusdError::Overflow)?;

    // Clear vault
    let remaining_collateral = vault.collateral.saturating_sub(seize_amount);

    // Update global config
    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_pusd_supply = config.total_pusd_supply.saturating_sub(total_debt);
    config.total_collateral_locked = config
        .total_collateral_locked
        .saturating_sub(seize_amount);
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    vault.debt = 0;
    vault.accrued_fee = 0;
    vault.collateral = remaining_collateral;
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    msg!(
        "Vault liquidated: debt={}, collateral seized={} (penalty={}), remaining={}",
        total_debt,
        seize_amount,
        penalty,
        remaining_collateral
    );

    Ok(())
}

fn process_redeem(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    pusd_amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let redeemer = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let config_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let oracle_account = next_account_info(account_iter)?;
    let _redeemer_pusd = next_account_info(account_iter)?;
    let _pusd_mint = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !redeemer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if pusd_amount == 0 {
        return Err(PusdError::InvalidAmount.into());
    }
    if vault_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut vault = Vault::try_from_slice(&vault_account.data.borrow())?;
    if !vault.is_initialized {
        return Err(PusdError::NotInitialized.into());
    }

    let clock = Clock::get()?;
    vault.accrue_fee(clock.unix_timestamp);

    if pusd_amount > vault.total_debt() {
        return Err(PusdError::BurnExceedsDebt.into());
    }

    let prism_price = read_oracle_price(oracle_account)?;

    // Calculate $1 worth of PRISM per PUSD
    let prism_to_return = usd_to_prism(pusd_amount, prism_price)?;
    if prism_to_return > vault.collateral {
        return Err(PusdError::RedemptionExceedsCollateral.into());
    }

    // In production: burn redeemer's PUSD via SPL Token burn
    // Transfer PRISM from collateral pool to redeemer
    let pool_balance = collateral_pool.lamports();
    if prism_to_return > pool_balance {
        return Err(PusdError::RedemptionExceedsCollateral.into());
    }
    **collateral_pool.try_borrow_mut_lamports()? = pool_balance
        .checked_sub(prism_to_return)
        .ok_or(PusdError::Overflow)?;
    **redeemer.try_borrow_mut_lamports()? = redeemer
        .lamports()
        .checked_add(prism_to_return)
        .ok_or(PusdError::Overflow)?;

    // Reduce vault debt
    if pusd_amount <= vault.accrued_fee {
        vault.accrued_fee = vault.accrued_fee.checked_sub(pusd_amount).unwrap();
    } else {
        let remaining = pusd_amount.checked_sub(vault.accrued_fee).unwrap();
        vault.accrued_fee = 0;
        vault.debt = vault.debt.saturating_sub(remaining);
    }
    vault.collateral = vault.collateral.saturating_sub(prism_to_return);
    vault.serialize(&mut &mut vault_account.data.borrow_mut()[..])?;

    // Update global config
    let mut config = PusdConfig::try_from_slice(&config_account.data.borrow())?;
    config.total_pusd_supply = config.total_pusd_supply.saturating_sub(pusd_amount);
    config.total_collateral_locked = config
        .total_collateral_locked
        .saturating_sub(prism_to_return);
    config.serialize(&mut &mut config_account.data.borrow_mut()[..])?;

    msg!(
        "Redeemed {} PUSD for {} PRISM (price={})",
        pusd_amount,
        prism_to_return,
        prism_price
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault(collateral: u64, debt: u64) -> Vault {
        Vault {
            is_initialized: true,
            owner: Pubkey::new_unique(),
            collateral,
            debt,
            accrued_fee: 0,
            last_fee_update: 1000,
            created_at: 1000,
            bump: 255,
            nonce: 0,
        }
    }

    #[test]
    fn test_collateral_ratio_no_debt() {
        let vault = make_vault(1_000_000_000, 0);
        let ratio = vault.collateral_ratio_bps(100_00000000).unwrap();
        assert_eq!(ratio, u64::MAX);
    }

    #[test]
    fn test_collateral_ratio_healthy() {
        // 10 PRISM collateral at $10/PRISM = $100 collateral
        // 50 PUSD debt = $50
        // Ratio = 200% = 20000 bps
        let vault = make_vault(
            10_000_000_000, // 10 PRISM in lamports
            50_000_000,     // 50 PUSD (6 decimals)
        );
        let ratio = vault.collateral_ratio_bps(10_00000000).unwrap(); // $10
        assert_eq!(ratio, 20000); // 200%
    }

    #[test]
    fn test_collateral_ratio_undercollateralized() {
        // 1 PRISM at $10 = $10 collateral
        // 10 PUSD debt = $10
        // Ratio = 100% = 10000 bps (below 120% liquidation threshold)
        let vault = make_vault(
            1_000_000_000,  // 1 PRISM
            10_000_000,     // 10 PUSD
        );
        let ratio = vault.collateral_ratio_bps(10_00000000).unwrap();
        assert_eq!(ratio, 10000);
        assert!(ratio < LIQUIDATION_THRESHOLD_BPS);
    }

    #[test]
    fn test_fee_accrual() {
        let mut vault = make_vault(10_000_000_000, 100_000_000); // 100 PUSD debt
        // Advance 1 year
        vault.accrue_fee(1000 + SECONDS_PER_YEAR as i64);
        // 2% of 100 PUSD = 2 PUSD = 2_000_000
        assert_eq!(vault.accrued_fee, 2_000_000);
    }

    #[test]
    fn test_fee_accrual_half_year() {
        let mut vault = make_vault(10_000_000_000, 100_000_000);
        vault.accrue_fee(1000 + (SECONDS_PER_YEAR / 2) as i64);
        // 1% of 100 PUSD = 1 PUSD = 1_000_000
        assert_eq!(vault.accrued_fee, 1_000_000);
    }

    #[test]
    fn test_fee_accrual_no_debt() {
        let mut vault = make_vault(10_000_000_000, 0);
        vault.accrue_fee(1000 + SECONDS_PER_YEAR as i64);
        assert_eq!(vault.accrued_fee, 0);
    }

    #[test]
    fn test_total_debt() {
        let mut vault = make_vault(10_000_000_000, 100_000_000);
        vault.accrued_fee = 5_000_000;
        assert_eq!(vault.total_debt(), 105_000_000);
    }

    #[test]
    fn test_usd_to_prism() {
        // 100 PUSD at $10/PRISM = 10 PRISM
        let prism = usd_to_prism(100_000_000, 10_00000000).unwrap();
        assert_eq!(prism, 10_000_000_000); // 10 PRISM in lamports
    }
}
