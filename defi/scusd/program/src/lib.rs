use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Minimum collateral ratio: 150 % (stored as basis points, 15_000)
const COLLATERAL_RATIO_MIN_BPS: u64 = 15_000;

/// Liquidation threshold: 120 % (12_000 bps)
const LIQUIDATION_RATIO_BPS: u64 = 12_000;

/// Annual stability fee: 2 % (200 bps)
const STABILITY_FEE_RATE_BPS: u64 = 200;

/// Liquidation discount: 5 % — liquidator gets collateral at 95 % of price
const LIQUIDATION_DISCOUNT_BPS: u64 = 500;

/// Fixed-point price decimals (oracle provides 8 decimals)
const PRICE_DECIMALS: u64 = 100_000_000;

/// Seconds in a year (365.25 days) for fee accrual
const SECONDS_PER_YEAR: u64 = 31_557_600;

/// Seed for the global state PDA
const GLOBAL_STATE_SEED: &[u8] = b"scusd_global";

/// Seed for vault PDAs
const VAULT_SEED: &[u8] = b"scusd_vault";

// ---------------------------------------------------------------------------
// State accounts
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GlobalState {
    /// Authority that initialized the program
    pub authority: Pubkey,
    /// Mint address of the SCUSD token
    pub scusd_mint: Pubkey,
    /// Mint address of the SCLONE collateral token
    pub sclone_mint: Pubkey,
    /// Program-owned token account holding collateral
    pub collateral_pool: Pubkey,
    /// Total SCLONE collateral locked across all vaults
    pub total_collateral: u64,
    /// Total SCUSD debt outstanding
    pub total_debt: u64,
    /// Minimum collateral ratio in basis points (default 15_000 = 150 %)
    pub collateral_ratio_min: u64,
    /// Liquidation ratio in basis points (default 12_000 = 120 %)
    pub liquidation_ratio: u64,
    /// Stability fee rate in basis points (default 200 = 2 %)
    pub stability_fee_rate: u64,
    /// Bump seed for this PDA
    pub bump: u8,
    /// Whether the system has been initialized
    pub is_initialized: bool,
}

impl GlobalState {
    pub const LEN: usize = 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1 + 1; // 170 bytes
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Vault {
    /// Owner of this vault
    pub owner: Pubkey,
    /// Amount of SCLONE deposited as collateral
    pub collateral_amount: u64,
    /// Amount of SCUSD debt minted against this vault
    pub debt_amount: u64,
    /// Current collateral ratio in basis points
    pub collateral_ratio: u64,
    /// Unix timestamp of the last update (for stability fee accrual)
    pub last_update: i64,
    /// Bump seed for this PDA
    pub bump: u8,
    /// Whether this vault is active
    pub is_initialized: bool,
}

impl Vault {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 1 + 1; // 66 bytes
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum ScusdInstruction {
    /// Initialize the SCUSD system.
    /// Accounts:
    ///   0. `[signer]`  Authority
    ///   1. `[writable]` Global state PDA
    ///   2. `[writable]` SCUSD mint (to be created)
    ///   3. `[]`         SCLONE mint
    ///   4. `[writable]` Collateral pool token account
    ///   5. `[]`         Token program
    ///   6. `[]`         System program
    ///   7. `[]`         Rent sysvar
    Initialize,

    /// Open a new collateral vault.
    /// Accounts:
    ///   0. `[signer]`  Vault owner
    ///   1. `[writable]` Vault PDA
    ///   2. `[]`         Global state PDA
    ///   3. `[]`         System program
    OpenVault,

    /// Deposit SCLONE collateral into a vault.
    /// Accounts:
    ///   0. `[signer]`  Vault owner
    ///   1. `[writable]` Vault PDA
    ///   2. `[writable]` Global state PDA
    ///   3. `[writable]` Owner's SCLONE token account
    ///   4. `[writable]` Collateral pool token account
    ///   5. `[]`         Token program
    ///   6. `[]`         Clock sysvar
    DepositCollateral { amount: u64 },

    /// Mint SCUSD against deposited collateral (must maintain >= 150 % ratio).
    /// Accounts:
    ///   0. `[signer]`  Vault owner
    ///   1. `[writable]` Vault PDA
    ///   2. `[writable]` Global state PDA
    ///   3. `[writable]` SCUSD mint
    ///   4. `[writable]` Owner's SCUSD token account
    ///   5. `[]`         Oracle price feed account
    ///   6. `[]`         Token program
    ///   7. `[]`         Clock sysvar
    MintScusd { amount: u64 },

    /// Burn SCUSD to repay vault debt.
    /// Accounts:
    ///   0. `[signer]`  Vault owner
    ///   1. `[writable]` Vault PDA
    ///   2. `[writable]` Global state PDA
    ///   3. `[writable]` SCUSD mint
    ///   4. `[writable]` Owner's SCUSD token account
    ///   5. `[]`         Token program
    ///   6. `[]`         Clock sysvar
    RepayScusd { amount: u64 },

    /// Withdraw excess collateral (must stay above 150 % ratio).
    /// Accounts:
    ///   0. `[signer]`  Vault owner
    ///   1. `[writable]` Vault PDA
    ///   2. `[writable]` Global state PDA
    ///   3. `[writable]` Collateral pool token account
    ///   4. `[writable]` Owner's SCLONE token account
    ///   5. `[]`         Oracle price feed account
    ///   6. `[]`         Token program
    ///   7. `[]`         Clock sysvar
    WithdrawCollateral { amount: u64 },

    /// Liquidate an undercollateralized vault (< 120 %).
    /// Liquidator repays the vault's debt and receives collateral at a 5 % discount.
    /// Accounts:
    ///   0. `[signer]`  Liquidator
    ///   1. `[writable]` Vault PDA (target)
    ///   2. `[writable]` Global state PDA
    ///   3. `[writable]` SCUSD mint
    ///   4. `[writable]` Liquidator's SCUSD token account
    ///   5. `[writable]` Liquidator's SCLONE token account
    ///   6. `[writable]` Collateral pool token account
    ///   7. `[]`         Oracle price feed account
    ///   8. `[]`         Token program
    ///   9. `[]`         Clock sysvar
    LiquidateVault,
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
    let instruction = ScusdInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        ScusdInstruction::Initialize => process_initialize(program_id, accounts),
        ScusdInstruction::OpenVault => process_open_vault(program_id, accounts),
        ScusdInstruction::DepositCollateral { amount } => {
            process_deposit_collateral(program_id, accounts, amount)
        }
        ScusdInstruction::MintScusd { amount } => process_mint_scusd(program_id, accounts, amount),
        ScusdInstruction::RepayScusd { amount } => {
            process_repay_scusd(program_id, accounts, amount)
        }
        ScusdInstruction::WithdrawCollateral { amount } => {
            process_withdraw_collateral(program_id, accounts, amount)
        }
        ScusdInstruction::LiquidateVault => process_liquidate_vault(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Instruction processors
// ---------------------------------------------------------------------------

fn process_initialize(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let scusd_mint_info = next_account_info(account_iter)?;
    let sclone_mint_info = next_account_info(account_iter)?;
    let collateral_pool_info = next_account_info(account_iter)?;
    let _token_program = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive and verify global state PDA
    let (global_pda, bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], program_id);
    if *global_state_info.key != global_pda {
        msg!("Error: invalid global state PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Allocate the global state account
    let rent = Rent::get()?;
    let space = GlobalState::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            global_state_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            global_state_info.clone(),
            system_program.clone(),
        ],
        &[&[GLOBAL_STATE_SEED, &[bump]]],
    )?;

    // Store initial global state
    let state = GlobalState {
        authority: *authority.key,
        scusd_mint: *scusd_mint_info.key,
        sclone_mint: *sclone_mint_info.key,
        collateral_pool: *collateral_pool_info.key,
        total_collateral: 0,
        total_debt: 0,
        collateral_ratio_min: COLLATERAL_RATIO_MIN_BPS,
        liquidation_ratio: LIQUIDATION_RATIO_BPS,
        stability_fee_rate: STABILITY_FEE_RATE_BPS,
        bump,
        is_initialized: true,
    };

    state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("SCUSD system initialized");
    Ok(())
}

fn process_open_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify global state is initialized
    let global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;
    if !global_state.is_initialized {
        msg!("Error: system not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Derive vault PDA for this owner
    let (vault_pda, vault_bump) =
        Pubkey::find_program_address(&[VAULT_SEED, owner.key.as_ref()], program_id);
    if *vault_info.key != vault_pda {
        msg!("Error: invalid vault PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Allocate vault account
    let rent = Rent::get()?;
    let space = Vault::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            owner.key,
            vault_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[owner.clone(), vault_info.clone(), system_program.clone()],
        &[&[VAULT_SEED, owner.key.as_ref(), &[vault_bump]]],
    )?;

    let clock = Clock::get()?;

    let vault = Vault {
        owner: *owner.key,
        collateral_amount: 0,
        debt_amount: 0,
        collateral_ratio: 0,
        last_update: clock.unix_timestamp,
        bump: vault_bump,
        is_initialized: true,
    };

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;

    msg!("Vault opened for {}", owner.key);
    Ok(())
}

fn process_deposit_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let owner_sclone_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::try_from_slice(&vault_info.data.borrow())?;
    if !vault.is_initialized || vault.owner != *owner.key {
        msg!("Error: vault not found or not owned by signer");
        return Err(ProgramError::InvalidArgument);
    }

    let mut global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;

    // Accrue stability fee before state change
    let clock = Clock::get()?;
    accrue_stability_fee(&mut vault, &global_state, clock.unix_timestamp);

    // Transfer SCLONE from owner to collateral pool
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            owner_sclone_account.key,
            collateral_pool.key,
            owner.key,
            &[],
            amount,
        )?,
        &[
            owner_sclone_account.clone(),
            collateral_pool.clone(),
            owner.clone(),
            token_program.clone(),
        ],
    )?;

    vault.collateral_amount = vault.collateral_amount.checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.last_update = clock.unix_timestamp;

    global_state.total_collateral = global_state.total_collateral.checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("Deposited {} SCLONE collateral", amount);
    Ok(())
}

fn process_mint_scusd(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let scusd_mint = next_account_info(account_iter)?;
    let owner_scusd_account = next_account_info(account_iter)?;
    let oracle_feed = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::try_from_slice(&vault_info.data.borrow())?;
    if !vault.is_initialized || vault.owner != *owner.key {
        return Err(ProgramError::InvalidArgument);
    }

    let mut global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;

    let clock = Clock::get()?;
    accrue_stability_fee(&mut vault, &global_state, clock.unix_timestamp);

    // Read SCLONE/USD price from oracle
    let sclone_price = read_oracle_price(oracle_feed)?;

    // Check that minting `amount` keeps the vault above 150 %
    let new_debt = vault.debt_amount.checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let ratio = calculate_collateral_ratio(vault.collateral_amount, new_debt, sclone_price);
    if ratio < global_state.collateral_ratio_min {
        msg!(
            "Error: collateral ratio {} bps would fall below minimum {} bps",
            ratio,
            global_state.collateral_ratio_min
        );
        return Err(ProgramError::InsufficientFunds);
    }

    // Mint SCUSD to the owner — signed by global state PDA (mint authority)
    invoke_signed(
        &spl_token::instruction::mint_to(
            token_program.key,
            scusd_mint.key,
            owner_scusd_account.key,
            global_state_info.key,
            &[],
            amount,
        )?,
        &[
            scusd_mint.clone(),
            owner_scusd_account.clone(),
            global_state_info.clone(),
            token_program.clone(),
        ],
        &[&[GLOBAL_STATE_SEED, &[global_state.bump]]],
    )?;

    vault.debt_amount = new_debt;
    vault.collateral_ratio = ratio;
    vault.last_update = clock.unix_timestamp;

    global_state.total_debt = global_state.total_debt.checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("Minted {} SCUSD (ratio: {} bps)", amount, ratio);
    Ok(())
}

fn process_repay_scusd(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let scusd_mint = next_account_info(account_iter)?;
    let owner_scusd_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::try_from_slice(&vault_info.data.borrow())?;
    if !vault.is_initialized || vault.owner != *owner.key {
        return Err(ProgramError::InvalidArgument);
    }

    let mut global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;

    let clock = Clock::get()?;
    accrue_stability_fee(&mut vault, &global_state, clock.unix_timestamp);

    let repay_amount = amount.min(vault.debt_amount);

    // Burn SCUSD from the owner
    invoke(
        &spl_token::instruction::burn(
            token_program.key,
            owner_scusd_account.key,
            scusd_mint.key,
            owner.key,
            &[],
            repay_amount,
        )?,
        &[
            owner_scusd_account.clone(),
            scusd_mint.clone(),
            owner.clone(),
            token_program.clone(),
        ],
    )?;

    vault.debt_amount = vault.debt_amount.checked_sub(repay_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.last_update = clock.unix_timestamp;

    global_state.total_debt = global_state.total_debt.checked_sub(repay_amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("Repaid {} SCUSD", repay_amount);
    Ok(())
}

fn process_withdraw_collateral(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let owner_sclone_account = next_account_info(account_iter)?;
    let oracle_feed = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !owner.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::try_from_slice(&vault_info.data.borrow())?;
    if !vault.is_initialized || vault.owner != *owner.key {
        return Err(ProgramError::InvalidArgument);
    }

    let mut global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;

    let clock = Clock::get()?;
    accrue_stability_fee(&mut vault, &global_state, clock.unix_timestamp);

    if amount > vault.collateral_amount {
        msg!("Error: withdraw amount exceeds collateral");
        return Err(ProgramError::InsufficientFunds);
    }

    let new_collateral = vault.collateral_amount.checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // If there is outstanding debt, ensure ratio stays above 150 %
    if vault.debt_amount > 0 {
        let sclone_price = read_oracle_price(oracle_feed)?;
        let ratio = calculate_collateral_ratio(new_collateral, vault.debt_amount, sclone_price);
        if ratio < global_state.collateral_ratio_min {
            msg!(
                "Error: withdrawal would drop ratio to {} bps (min {})",
                ratio,
                global_state.collateral_ratio_min
            );
            return Err(ProgramError::InsufficientFunds);
        }
        vault.collateral_ratio = ratio;
    }

    // Transfer SCLONE from pool back to owner — signed by global state PDA
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            collateral_pool.key,
            owner_sclone_account.key,
            global_state_info.key,
            &[],
            amount,
        )?,
        &[
            collateral_pool.clone(),
            owner_sclone_account.clone(),
            global_state_info.clone(),
            token_program.clone(),
        ],
        &[&[GLOBAL_STATE_SEED, &[global_state.bump]]],
    )?;

    vault.collateral_amount = new_collateral;
    vault.last_update = clock.unix_timestamp;

    global_state.total_collateral = global_state.total_collateral.checked_sub(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!("Withdrew {} SCLONE collateral", amount);
    Ok(())
}

fn process_liquidate_vault(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let liquidator = next_account_info(account_iter)?;
    let vault_info = next_account_info(account_iter)?;
    let global_state_info = next_account_info(account_iter)?;
    let scusd_mint = next_account_info(account_iter)?;
    let liquidator_scusd_account = next_account_info(account_iter)?;
    let liquidator_sclone_account = next_account_info(account_iter)?;
    let collateral_pool = next_account_info(account_iter)?;
    let oracle_feed = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !liquidator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut vault = Vault::try_from_slice(&vault_info.data.borrow())?;
    if !vault.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    let mut global_state = GlobalState::try_from_slice(&global_state_info.data.borrow())?;

    let clock = Clock::get()?;
    accrue_stability_fee(&mut vault, &global_state, clock.unix_timestamp);

    let sclone_price = read_oracle_price(oracle_feed)?;

    // Check the vault is undercollateralized (< 120 %)
    let ratio = calculate_collateral_ratio(vault.collateral_amount, vault.debt_amount, sclone_price);
    if ratio >= global_state.liquidation_ratio {
        msg!(
            "Error: vault ratio {} bps is above liquidation threshold {} bps",
            ratio,
            global_state.liquidation_ratio
        );
        return Err(ProgramError::InvalidArgument);
    }

    let debt_to_repay = vault.debt_amount;

    // Collateral the liquidator receives = debt_value / (price * (1 - discount))
    // With 5 % discount the liquidator pays debt_value for collateral worth debt_value / 0.95
    // collateral_tokens = (debt_to_repay * PRICE_DECIMALS) / (sclone_price * 9500 / 10000)
    let discounted_price = sclone_price
        .checked_mul(10_000 - LIQUIDATION_DISCOUNT_BPS)
        .ok_or(ProgramError::ArithmeticOverflow)?
        / 10_000;

    let collateral_to_liquidate = debt_to_repay
        .checked_mul(PRICE_DECIMALS)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .checked_div(discounted_price)
        .ok_or(ProgramError::ArithmeticOverflow)?
        .min(vault.collateral_amount);

    // Liquidator burns SCUSD to repay the vault's debt
    invoke(
        &spl_token::instruction::burn(
            token_program.key,
            liquidator_scusd_account.key,
            scusd_mint.key,
            liquidator.key,
            &[],
            debt_to_repay,
        )?,
        &[
            liquidator_scusd_account.clone(),
            scusd_mint.clone(),
            liquidator.clone(),
            token_program.clone(),
        ],
    )?;

    // Transfer collateral from pool to liquidator — signed by global state PDA
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            collateral_pool.key,
            liquidator_sclone_account.key,
            global_state_info.key,
            &[],
            collateral_to_liquidate,
        )?,
        &[
            collateral_pool.clone(),
            liquidator_sclone_account.clone(),
            global_state_info.clone(),
            token_program.clone(),
        ],
        &[&[GLOBAL_STATE_SEED, &[global_state.bump]]],
    )?;

    // Update vault — clear debt, reduce collateral
    vault.debt_amount = 0;
    vault.collateral_amount = vault.collateral_amount.checked_sub(collateral_to_liquidate)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    vault.collateral_ratio = 0;
    vault.last_update = clock.unix_timestamp;

    global_state.total_debt = global_state.total_debt.checked_sub(debt_to_repay)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    global_state.total_collateral = global_state.total_collateral.checked_sub(collateral_to_liquidate)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    vault.serialize(&mut &mut vault_info.data.borrow_mut()[..])?;
    global_state.serialize(&mut &mut global_state_info.data.borrow_mut()[..])?;

    msg!(
        "Liquidated vault: repaid {} SCUSD, seized {} SCLONE collateral",
        debt_to_repay,
        collateral_to_liquidate
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate the collateral ratio in basis points.
///
/// ratio_bps = (collateral_amount * sclone_price * 10_000) / (debt_amount * PRICE_DECIMALS)
///
/// A result of 15_000 means 150 %.
pub fn calculate_collateral_ratio(collateral: u64, debt: u64, sclone_price: u64) -> u64 {
    if debt == 0 {
        return u64::MAX; // infinite ratio when no debt
    }
    let collateral_value = (collateral as u128)
        .checked_mul(sclone_price as u128)
        .unwrap_or(0);
    let debt_value = (debt as u128)
        .checked_mul(PRICE_DECIMALS as u128)
        .unwrap_or(1);

    let ratio = collateral_value
        .checked_mul(10_000)
        .unwrap_or(0)
        / debt_value;

    ratio as u64
}

/// Maximum SCUSD that can be minted given collateral at the minimum ratio.
///
/// max_mint = (collateral * sclone_price * 10_000) / (PRICE_DECIMALS * collateral_ratio_min) - current_debt
pub fn calculate_max_mint(
    collateral: u64,
    current_debt: u64,
    sclone_price: u64,
    min_ratio_bps: u64,
) -> u64 {
    let collateral_value = (collateral as u128)
        .checked_mul(sclone_price as u128)
        .unwrap_or(0);

    let max_debt = collateral_value
        .checked_mul(10_000)
        .unwrap_or(0)
        / ((PRICE_DECIMALS as u128) * (min_ratio_bps as u128));

    let max_debt = max_debt as u64;
    max_debt.saturating_sub(current_debt)
}

/// Accrue the annual stability fee onto the vault's debt.
///
/// fee = debt * rate_bps / 10_000 * elapsed_seconds / SECONDS_PER_YEAR
pub fn calculate_stability_fee(debt: u64, rate_bps: u64, elapsed_seconds: u64) -> u64 {
    if debt == 0 || elapsed_seconds == 0 {
        return 0;
    }
    let fee = (debt as u128)
        .checked_mul(rate_bps as u128)
        .unwrap_or(0)
        .checked_mul(elapsed_seconds as u128)
        .unwrap_or(0)
        / (10_000u128 * SECONDS_PER_YEAR as u128);

    fee as u64
}

/// Apply accrued stability fee to vault debt.
fn accrue_stability_fee(vault: &mut Vault, global: &GlobalState, now: i64) {
    if vault.debt_amount == 0 || vault.last_update >= now {
        return;
    }
    let elapsed = (now - vault.last_update) as u64;
    let fee = calculate_stability_fee(vault.debt_amount, global.stability_fee_rate, elapsed);
    if fee > 0 {
        vault.debt_amount = vault.debt_amount.saturating_add(fee);
        msg!("Accrued stability fee: {} (debt now {})", fee, vault.debt_amount);
    }
}

/// Read a u64 price from the oracle price-feed account.
/// Expected layout: 64-byte header + price at offset 64 (u64 LE, 8 decimals).
fn read_oracle_price(oracle_account: &AccountInfo) -> Result<u64, ProgramError> {
    let data = oracle_account.data.borrow();
    if data.len() < 72 {
        msg!("Error: oracle account data too small");
        return Err(ProgramError::InvalidAccountData);
    }
    let price_bytes: [u8; 8] = data[64..72]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let price = u64::from_le_bytes(price_bytes);
    if price == 0 {
        msg!("Error: oracle price is zero");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(price)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collateral_ratio() {
        // 1000 SCLONE at $2.00 (2_0000_0000 fixed-point) backing 1000 SCUSD
        let ratio = calculate_collateral_ratio(1_000, 1_000, 200_000_000);
        assert_eq!(ratio, 20_000); // 200 %
    }

    #[test]
    fn test_collateral_ratio_no_debt() {
        let ratio = calculate_collateral_ratio(1_000, 0, 200_000_000);
        assert_eq!(ratio, u64::MAX);
    }

    #[test]
    fn test_max_mint() {
        // 1500 SCLONE at $1.00, min ratio 150 %
        let max = calculate_max_mint(1_500, 0, 100_000_000, 15_000);
        assert_eq!(max, 1_000);
    }

    #[test]
    fn test_stability_fee() {
        // 10_000 debt, 2 % annual, 1 year elapsed
        let fee = calculate_stability_fee(10_000, 200, SECONDS_PER_YEAR);
        assert_eq!(fee, 200); // 2 % of 10_000
    }

    #[test]
    fn test_stability_fee_partial_year() {
        // 10_000 debt, 2 % annual, half year
        let fee = calculate_stability_fee(10_000, 200, SECONDS_PER_YEAR / 2);
        assert_eq!(fee, 100); // 1 %
    }
}
