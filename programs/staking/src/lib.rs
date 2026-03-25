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
use thiserror::Error;

// ---------------------------------------------------------------------------
// Program ID
// ---------------------------------------------------------------------------

solana_program::declare_id!("PrismStake1111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for the staking pool PDA.
pub const POOL_SEED: &[u8] = b"prism_staking_pool";
/// Seed for individual stake account PDAs.
pub const STAKE_SEED: &[u8] = b"prism_stake";
/// Seed for the liquid staking mint authority PDA.
pub const ST_PRISM_AUTHORITY_SEED: &[u8] = b"st_prism_authority";
/// Seed for the slashing insurance pool PDA.
pub const INSURANCE_SEED: &[u8] = b"prism_insurance";

/// Basis-point denominator (100%).
pub const BPS_DENOMINATOR: u64 = 10_000;
/// Insurance cut taken from every reward distribution (2%).
pub const INSURANCE_BPS: u64 = 200;
/// Seconds per day, used for lock-duration arithmetic.
pub const SECONDS_PER_DAY: i64 = 86_400;

// ---------------------------------------------------------------------------
// Lock tiers
// ---------------------------------------------------------------------------

/// The lock tier determines the reward multiplier a staker receives.
#[derive(BorshSerialize, BorshDeserialize, Clone, Copy, Debug, PartialEq)]
pub enum LockTier {
    /// No lock-up, base 1x rewards.
    Flexible,
    /// 30-day lock, 1.5x rewards.
    Locked30,
    /// 90-day lock, 2x rewards.
    Locked90,
    /// 180-day lock, 3x rewards.
    Locked180,
    /// 365-day lock, 5x rewards.
    Locked365,
}

impl LockTier {
    /// Reward multiplier expressed in basis points (1x = 10 000).
    pub fn reward_multiplier_bps(&self) -> u64 {
        match self {
            LockTier::Flexible => 10_000,
            LockTier::Locked30 => 15_000,
            LockTier::Locked90 => 20_000,
            LockTier::Locked180 => 30_000,
            LockTier::Locked365 => 50_000,
        }
    }

    /// Lock duration in seconds.  Flexible has zero lock.
    pub fn lock_duration_seconds(&self) -> i64 {
        match self {
            LockTier::Flexible => 0,
            LockTier::Locked30 => 30 * SECONDS_PER_DAY,
            LockTier::Locked90 => 90 * SECONDS_PER_DAY,
            LockTier::Locked180 => 180 * SECONDS_PER_DAY,
            LockTier::Locked365 => 365 * SECONDS_PER_DAY,
        }
    }
}

// ---------------------------------------------------------------------------
// Account state
// ---------------------------------------------------------------------------

/// Global staking pool state (one per program instance).
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakingPool {
    /// Whether the pool has been initialized.
    pub is_initialized: bool,
    /// Authority that can update pool parameters.
    pub authority: Pubkey,
    /// SPL Token mint for stPRISM (liquid staking receipt).
    pub st_prism_mint: Pubkey,
    /// Total lamports staked across all users.
    pub total_staked: u64,
    /// Accumulated reward per token, scaled by 1e12 for precision.
    pub accumulated_reward_per_token: u128,
    /// Total lamports burned via slashing events, tracked for transparency.
    pub total_slashed: u64,
    /// Insurance pool balance (2% of all distributed rewards).
    pub insurance_balance: u64,
    /// Base annual reward rate in basis points (e.g., 800 = 8%).
    pub base_reward_rate_bps: u64,
    /// Last slot at which rewards were updated.
    pub last_update_slot: u64,
    /// Bump seed for the pool PDA.
    pub bump: u8,
}

impl StakingPool {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 16 + 8 + 8 + 8 + 8 + 1;
}

/// Per-user stake account.
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct StakeAccount {
    /// Whether this account has been initialized.
    pub is_initialized: bool,
    /// Owner of this stake position.
    pub owner: Pubkey,
    /// Validator this stake is delegated to (Pubkey::default() if none).
    pub delegated_validator: Pubkey,
    /// Amount of lamports staked.
    pub staked_amount: u64,
    /// Reward debt used in the masterchef-style accounting.
    pub reward_debt: u128,
    /// Pending (unclaimed) rewards.
    pub pending_rewards: u64,
    /// Lock tier chosen at stake time.
    pub lock_tier: LockTier,
    /// Unix timestamp when the stake was created.
    pub stake_timestamp: i64,
    /// Unix timestamp when the lock expires (0 for Flexible).
    pub unlock_timestamp: i64,
    /// Whether auto-compound is enabled.
    pub auto_compound: bool,
    /// Whether liquid staking (stPRISM minting) is enabled.
    pub liquid_staking: bool,
    /// Amount of stPRISM minted for this position.
    pub st_prism_minted: u64,
    /// Commission rate of the delegated validator in bps at time of delegation.
    pub validator_commission_bps: u64,
    /// Bump seed for this stake PDA.
    pub bump: u8,
}

impl StakeAccount {
    pub const LEN: usize = 1 + 32 + 32 + 8 + 16 + 8 + 1 + 8 + 8 + 1 + 1 + 8 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum StakingInstruction {
    /// Initialize the global staking pool.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Pool authority (payer)
    ///   1. `[writable]`         Pool PDA
    ///   2. `[]`                 stPRISM mint
    ///   3. `[]`                 System program
    InitializePool {
        base_reward_rate_bps: u64,
    },

    /// Stake PRISM tokens.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Staker
    ///   1. `[writable]`         Stake account PDA
    ///   2. `[writable]`         Pool PDA
    ///   3. `[]`                 Clock sysvar
    ///   4. `[]`                 System program
    Stake {
        amount: u64,
        lock_tier: LockTier,
        auto_compound: bool,
        liquid_staking: bool,
    },

    /// Unstake PRISM tokens.  Fails if lock has not expired.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Staker
    ///   1. `[writable]`         Stake account PDA
    ///   2. `[writable]`         Pool PDA
    ///   3. `[]`                 Clock sysvar
    ///   4. `[]`                 System program
    Unstake {
        amount: u64,
    },

    /// Claim accumulated rewards.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Staker
    ///   1. `[writable]`         Stake account PDA
    ///   2. `[writable]`         Pool PDA
    ///   3. `[]`                 Clock sysvar
    ///   4. `[]`                 System program
    ClaimRewards,

    /// Delegate stake to a validator.
    ///
    /// Accounts:
    ///   0. `[signer]`    Staker
    ///   1. `[writable]`  Stake account PDA
    ///   2. `[]`          Validator identity account
    Delegate {
        validator: Pubkey,
        commission_bps: u64,
    },

    /// Toggle auto-compound on an existing stake.
    ///
    /// Accounts:
    ///   0. `[signer]`   Staker
    ///   1. `[writable]` Stake account PDA
    ToggleAutoCompound,

    /// Mint stPRISM liquid staking receipt tokens.
    ///
    /// Accounts:
    ///   0. `[signer]`           Staker
    ///   1. `[writable]`         Stake account PDA
    ///   2. `[writable]`         stPRISM mint
    ///   3. `[writable]`         Staker's stPRISM token account
    ///   4. `[]`                 Mint authority PDA
    ///   5. `[]`                 SPL Token program
    MintStPrism,

    /// Burn stPRISM tokens to unlock the underlying stake.
    ///
    /// Accounts:
    ///   0. `[signer]`           Staker
    ///   1. `[writable]`         Stake account PDA
    ///   2. `[writable]`         stPRISM mint
    ///   3. `[writable]`         Staker's stPRISM token account
    ///   4. `[]`                 SPL Token program
    BurnStPrism {
        amount: u64,
    },

    /// Slash a validator's delegated stake (authority only).
    ///
    /// Accounts:
    ///   0. `[signer]`   Pool authority
    ///   1. `[writable]` Stake account PDA
    ///   2. `[writable]` Pool PDA
    Slash {
        amount: u64,
    },

    /// Update the pool's reward accumulator (permissionless crank).
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[]`         Clock sysvar
    UpdateRewards,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum StakingError {
    #[error("Pool already initialized")]
    AlreadyInitialized,
    #[error("Pool not initialized")]
    NotInitialized,
    #[error("Stake account already initialized")]
    StakeAlreadyInitialized,
    #[error("Stake account not initialized")]
    StakeNotInitialized,
    #[error("Lock period has not expired")]
    LockNotExpired,
    #[error("Insufficient staked balance")]
    InsufficientStake,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid lock tier")]
    InvalidLockTier,
    #[error("Liquid staking not enabled for this position")]
    LiquidStakingNotEnabled,
    #[error("No rewards to claim")]
    NoRewards,
    #[error("Invalid account owner")]
    InvalidAccountOwner,
    #[error("Insufficient stPRISM balance")]
    InsufficientStPrism,
}

impl From<StakingError> for ProgramError {
    fn from(e: StakingError) -> Self {
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
    let instruction = StakingInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        StakingInstruction::InitializePool { base_reward_rate_bps } => {
            process_initialize_pool(program_id, accounts, base_reward_rate_bps)
        }
        StakingInstruction::Stake {
            amount,
            lock_tier,
            auto_compound,
            liquid_staking,
        } => process_stake(program_id, accounts, amount, lock_tier, auto_compound, liquid_staking),
        StakingInstruction::Unstake { amount } => {
            process_unstake(program_id, accounts, amount)
        }
        StakingInstruction::ClaimRewards => process_claim_rewards(program_id, accounts),
        StakingInstruction::Delegate {
            validator,
            commission_bps,
        } => process_delegate(program_id, accounts, validator, commission_bps),
        StakingInstruction::ToggleAutoCompound => {
            process_toggle_auto_compound(program_id, accounts)
        }
        StakingInstruction::MintStPrism => process_mint_st_prism(program_id, accounts),
        StakingInstruction::BurnStPrism { amount } => {
            process_burn_st_prism(program_id, accounts, amount)
        }
        StakingInstruction::Slash { amount } => process_slash(program_id, accounts, amount),
        StakingInstruction::UpdateRewards => process_update_rewards(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    base_reward_rate_bps: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let st_prism_mint = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (pool_pda, bump) = Pubkey::find_program_address(&[POOL_SEED], program_id);
    if pool_pda != *pool_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Create the pool account via CPI.
    let rent = Rent::get()?;
    let space = StakingPool::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            pool_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[authority.clone(), pool_account.clone(), system_program.clone()],
        &[&[POOL_SEED, &[bump]]],
    )?;

    let pool = StakingPool {
        is_initialized: true,
        authority: *authority.key,
        st_prism_mint: *st_prism_mint.key,
        total_staked: 0,
        accumulated_reward_per_token: 0,
        total_slashed: 0,
        insurance_balance: 0,
        base_reward_rate_bps,
        last_update_slot: 0,
        bump,
    };

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Prism staking pool initialized. Base reward rate: {} bps", base_reward_rate_bps);
    Ok(())
}

fn process_stake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    lock_tier: LockTier,
    auto_compound: bool,
    liquid_staking: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Derive the stake PDA.
    let (stake_pda, stake_bump) =
        Pubkey::find_program_address(&[STAKE_SEED, staker.key.as_ref()], program_id);
    if stake_pda != *stake_account.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Load or create stake account.
    let mut pool = StakingPool::try_from_slice(&pool_account.data.borrow())?;
    if !pool.is_initialized {
        return Err(StakingError::NotInitialized.into());
    }

    // If the stake account doesn't exist yet, create it.
    if stake_account.data_len() == 0 {
        let rent = Rent::get()?;
        let space = StakeAccount::LEN;
        let lamports = rent.minimum_balance(space);

        invoke_signed(
            &system_instruction::create_account(
                staker.key,
                stake_account.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[staker.clone(), stake_account.clone(), system_program.clone()],
            &[&[STAKE_SEED, staker.key.as_ref(), &[stake_bump]]],
        )?;

        let unlock_ts = if lock_tier.lock_duration_seconds() > 0 {
            now.checked_add(lock_tier.lock_duration_seconds())
                .ok_or(StakingError::Overflow)?
        } else {
            0
        };

        let stake = StakeAccount {
            is_initialized: true,
            owner: *staker.key,
            delegated_validator: Pubkey::default(),
            staked_amount: amount,
            reward_debt: 0,
            pending_rewards: 0,
            lock_tier,
            stake_timestamp: now,
            unlock_timestamp: unlock_ts,
            auto_compound,
            liquid_staking,
            st_prism_minted: 0,
            validator_commission_bps: 0,
            bump: stake_bump,
        };

        stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;
    } else {
        let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
        if !stake.is_initialized {
            return Err(StakingError::StakeNotInitialized.into());
        }
        if stake.owner != *staker.key {
            return Err(StakingError::Unauthorized.into());
        }

        // Settle pending rewards before modifying the position.
        let pending = calculate_pending_rewards(&stake, &pool)?;
        stake.pending_rewards = stake
            .pending_rewards
            .checked_add(pending)
            .ok_or(StakingError::Overflow)?;

        stake.staked_amount = stake
            .staked_amount
            .checked_add(amount)
            .ok_or(StakingError::Overflow)?;

        // Recalculate reward debt.
        stake.reward_debt = (stake.staked_amount as u128)
            .checked_mul(pool.accumulated_reward_per_token)
            .ok_or(StakingError::Overflow)?;

        stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;
    }

    // Transfer lamports from staker to pool.
    invoke(
        &system_instruction::transfer(staker.key, pool_account.key, amount),
        &[staker.clone(), pool_account.clone(), system_program.clone()],
    )?;

    pool.total_staked = pool
        .total_staked
        .checked_add(amount)
        .ok_or(StakingError::Overflow)?;

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!(
        "Staked {} lamports with tier {:?} (multiplier {}x). Auto-compound: {}, Liquid: {}",
        amount,
        lock_tier,
        lock_tier.reward_multiplier_bps() as f64 / BPS_DENOMINATOR as f64,
        auto_compound,
        liquid_staking,
    );

    Ok(())
}

fn process_unstake(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }

    // Check lock expiry.
    if stake.unlock_timestamp > 0 && now < stake.unlock_timestamp {
        msg!(
            "Lock not expired. Current: {}, Unlock: {}",
            now,
            stake.unlock_timestamp
        );
        return Err(StakingError::LockNotExpired.into());
    }

    if amount > stake.staked_amount {
        return Err(StakingError::InsufficientStake.into());
    }

    let mut pool = StakingPool::try_from_slice(&pool_account.data.borrow())?;

    // Settle pending rewards.
    let pending = calculate_pending_rewards(&stake, &pool)?;
    stake.pending_rewards = stake
        .pending_rewards
        .checked_add(pending)
        .ok_or(StakingError::Overflow)?;

    stake.staked_amount = stake
        .staked_amount
        .checked_sub(amount)
        .ok_or(StakingError::Overflow)?;

    stake.reward_debt = (stake.staked_amount as u128)
        .checked_mul(pool.accumulated_reward_per_token)
        .ok_or(StakingError::Overflow)?;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    // Transfer lamports back from pool PDA to staker.
    **pool_account.try_borrow_mut_lamports()? -= amount;
    **staker.try_borrow_mut_lamports()? += amount;

    pool.total_staked = pool
        .total_staked
        .checked_sub(amount)
        .ok_or(StakingError::Overflow)?;

    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Unstaked {} lamports", amount);
    Ok(())
}

fn process_claim_rewards(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }

    let mut pool = StakingPool::try_from_slice(&pool_account.data.borrow())?;

    // Calculate pending.
    let pending = calculate_pending_rewards(&stake, &pool)?;
    let total_claimable = stake
        .pending_rewards
        .checked_add(pending)
        .ok_or(StakingError::Overflow)?;

    if total_claimable == 0 {
        return Err(StakingError::NoRewards.into());
    }

    // Deduct insurance cut (2% of rewards go to slashing protection pool).
    let insurance_cut = total_claimable
        .checked_mul(INSURANCE_BPS)
        .ok_or(StakingError::Overflow)?
        / BPS_DENOMINATOR;
    let net_reward = total_claimable
        .checked_sub(insurance_cut)
        .ok_or(StakingError::Overflow)?;

    pool.insurance_balance = pool
        .insurance_balance
        .checked_add(insurance_cut)
        .ok_or(StakingError::Overflow)?;

    // If auto-compound is enabled, restake the reward; otherwise pay out.
    if stake.auto_compound {
        stake.staked_amount = stake
            .staked_amount
            .checked_add(net_reward)
            .ok_or(StakingError::Overflow)?;
        pool.total_staked = pool
            .total_staked
            .checked_add(net_reward)
            .ok_or(StakingError::Overflow)?;
        msg!("Auto-compounded {} lamports back into stake", net_reward);
    } else {
        // Transfer lamports from pool to staker.
        **pool_account.try_borrow_mut_lamports()? -= net_reward;
        **staker.try_borrow_mut_lamports()? += net_reward;
        msg!("Claimed {} lamports in rewards", net_reward);
    }

    // Reset reward accounting.
    stake.pending_rewards = 0;
    stake.reward_debt = (stake.staked_amount as u128)
        .checked_mul(pool.accumulated_reward_per_token)
        .ok_or(StakingError::Overflow)?;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;
    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Insurance pool balance: {} lamports", pool.insurance_balance);
    Ok(())
}

fn process_delegate(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    validator: Pubkey,
    commission_bps: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let _validator_account = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }

    stake.delegated_validator = validator;
    stake.validator_commission_bps = commission_bps;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!(
        "Delegated stake to validator {} with commission {} bps",
        validator,
        commission_bps
    );
    Ok(())
}

fn process_toggle_auto_compound(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }

    stake.auto_compound = !stake.auto_compound;
    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Auto-compound toggled to {}", stake.auto_compound);
    Ok(())
}

fn process_mint_st_prism(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let st_prism_mint = next_account_info(account_iter)?;
    let staker_token_account = next_account_info(account_iter)?;
    let mint_authority = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }
    if !stake.liquid_staking {
        return Err(StakingError::LiquidStakingNotEnabled.into());
    }

    // The amount of stPRISM to mint equals staked amount minus already minted.
    let mintable = stake
        .staked_amount
        .checked_sub(stake.st_prism_minted)
        .ok_or(StakingError::Overflow)?;

    if mintable == 0 {
        msg!("No additional stPRISM to mint");
        return Ok(());
    }

    // Derive mint authority PDA.
    let (authority_pda, authority_bump) =
        Pubkey::find_program_address(&[ST_PRISM_AUTHORITY_SEED], program_id);
    if authority_pda != *mint_authority.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // CPI: mint stPRISM tokens.
    let mint_ix = spl_token_mint_to(
        token_program.key,
        st_prism_mint.key,
        staker_token_account.key,
        &authority_pda,
        mintable,
    )?;

    invoke_signed(
        &mint_ix,
        &[
            st_prism_mint.clone(),
            staker_token_account.clone(),
            mint_authority.clone(),
            token_program.clone(),
        ],
        &[&[ST_PRISM_AUTHORITY_SEED, &[authority_bump]]],
    )?;

    stake.st_prism_minted = stake
        .st_prism_minted
        .checked_add(mintable)
        .ok_or(StakingError::Overflow)?;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Minted {} stPRISM liquid staking tokens", mintable);
    Ok(())
}

fn process_burn_st_prism(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let staker = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let st_prism_mint = next_account_info(account_iter)?;
    let staker_token_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;

    if !staker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }
    if stake.owner != *staker.key {
        return Err(StakingError::Unauthorized.into());
    }

    if amount > stake.st_prism_minted {
        return Err(StakingError::InsufficientStPrism.into());
    }

    // CPI: burn stPRISM tokens.
    let burn_ix = spl_token_burn(
        token_program.key,
        staker_token_account.key,
        st_prism_mint.key,
        staker.key,
        amount,
    )?;

    invoke(
        &burn_ix,
        &[
            staker_token_account.clone(),
            st_prism_mint.clone(),
            staker.clone(),
            token_program.clone(),
        ],
    )?;

    stake.st_prism_minted = stake
        .st_prism_minted
        .checked_sub(amount)
        .ok_or(StakingError::Overflow)?;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;

    msg!("Burned {} stPRISM tokens", amount);
    Ok(())
}

fn process_slash(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let stake_account = next_account_info(account_iter)?;
    let pool_account = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool = StakingPool::try_from_slice(&pool_account.data.borrow())?;
    if pool.authority != *authority.key {
        return Err(StakingError::Unauthorized.into());
    }

    let mut stake = StakeAccount::try_from_slice(&stake_account.data.borrow())?;
    if !stake.is_initialized {
        return Err(StakingError::StakeNotInitialized.into());
    }

    let slash_amount = std::cmp::min(amount, stake.staked_amount);

    // First, try to cover from insurance pool.
    let insurance_cover = std::cmp::min(slash_amount, pool.insurance_balance);
    let actual_slash = slash_amount
        .checked_sub(insurance_cover)
        .ok_or(StakingError::Overflow)?;

    pool.insurance_balance = pool
        .insurance_balance
        .checked_sub(insurance_cover)
        .ok_or(StakingError::Overflow)?;

    stake.staked_amount = stake
        .staked_amount
        .checked_sub(actual_slash)
        .ok_or(StakingError::Overflow)?;

    pool.total_staked = pool
        .total_staked
        .checked_sub(actual_slash)
        .ok_or(StakingError::Overflow)?;

    pool.total_slashed = pool
        .total_slashed
        .checked_add(slash_amount)
        .ok_or(StakingError::Overflow)?;

    stake.serialize(&mut &mut stake_account.data.borrow_mut()[..])?;
    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!(
        "Slashed {} lamports (insurance covered {}, staker lost {})",
        slash_amount,
        insurance_cover,
        actual_slash
    );
    Ok(())
}

fn process_update_rewards(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_account = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    let clock = Clock::from_account_info(clock_sysvar)?;
    let current_slot = clock.slot;

    let mut pool = StakingPool::try_from_slice(&pool_account.data.borrow())?;
    if !pool.is_initialized {
        return Err(StakingError::NotInitialized.into());
    }

    if pool.total_staked == 0 {
        pool.last_update_slot = current_slot;
        pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;
        return Ok(());
    }

    let slots_elapsed = current_slot.saturating_sub(pool.last_update_slot);
    if slots_elapsed == 0 {
        return Ok(());
    }

    // Reward calculation:
    //   reward_per_slot = (total_staked * base_rate_bps) / (BPS_DENOMINATOR * slots_per_year)
    //   We approximate slots_per_year as 63_072_000 (2 slots/sec * 86400 * 365).
    const SLOTS_PER_YEAR: u128 = 63_072_000;
    let total_reward = (pool.total_staked as u128)
        .checked_mul(pool.base_reward_rate_bps as u128)
        .ok_or(StakingError::Overflow)?
        .checked_mul(slots_elapsed as u128)
        .ok_or(StakingError::Overflow)?
        / BPS_DENOMINATOR as u128
        / SLOTS_PER_YEAR;

    if total_reward > 0 {
        // Scale by 1e12 for precision.
        let reward_per_token = total_reward
            .checked_mul(1_000_000_000_000)
            .ok_or(StakingError::Overflow)?
            / pool.total_staked as u128;

        pool.accumulated_reward_per_token = pool
            .accumulated_reward_per_token
            .checked_add(reward_per_token)
            .ok_or(StakingError::Overflow)?;
    }

    pool.last_update_slot = current_slot;
    pool.serialize(&mut &mut pool_account.data.borrow_mut()[..])?;

    msg!("Rewards updated at slot {}. Total reward emitted: {} lamports", current_slot, total_reward);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate pending rewards for a stake position using masterchef-style accounting,
/// scaled by the lock tier's reward multiplier.
fn calculate_pending_rewards(
    stake: &StakeAccount,
    pool: &StakingPool,
) -> Result<u64, ProgramError> {
    if stake.staked_amount == 0 {
        return Ok(0);
    }

    let accumulated = (stake.staked_amount as u128)
        .checked_mul(pool.accumulated_reward_per_token)
        .ok_or(StakingError::Overflow)?;

    let raw_pending = accumulated
        .checked_sub(stake.reward_debt)
        .ok_or(StakingError::Overflow)?
        / 1_000_000_000_000; // undo the 1e12 scaling

    // Apply lock tier multiplier.
    let multiplied = raw_pending
        .checked_mul(stake.lock_tier.reward_multiplier_bps() as u128)
        .ok_or(StakingError::Overflow)?
        / BPS_DENOMINATOR as u128;

    // Deduct validator commission if delegated.
    let after_commission = if stake.delegated_validator != Pubkey::default()
        && stake.validator_commission_bps > 0
    {
        let commission = multiplied
            .checked_mul(stake.validator_commission_bps as u128)
            .ok_or(StakingError::Overflow)?
            / BPS_DENOMINATOR as u128;
        multiplied
            .checked_sub(commission)
            .ok_or(StakingError::Overflow)?
    } else {
        multiplied
    };

    Ok(after_commission as u64)
}

// ---------------------------------------------------------------------------
// SPL Token CPI instruction builders
// ---------------------------------------------------------------------------

/// Build an SPL Token `MintTo` instruction manually to avoid a direct
/// `spl-token` crate dependency (keeps the program lean).
fn spl_token_mint_to(
    token_program_id: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let data = {
        let mut buf = Vec::with_capacity(9);
        buf.push(7); // MintTo instruction index
        buf.extend_from_slice(&amount.to_le_bytes());
        buf
    };
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*mint, false),
            solana_program::instruction::AccountMeta::new(*destination, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}

/// Build an SPL Token `Burn` instruction manually.
fn spl_token_burn(
    token_program_id: &Pubkey,
    account: &Pubkey,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Result<solana_program::instruction::Instruction, ProgramError> {
    let data = {
        let mut buf = Vec::with_capacity(9);
        buf.push(8); // Burn instruction index
        buf.extend_from_slice(&amount.to_le_bytes());
        buf
    };
    Ok(solana_program::instruction::Instruction {
        program_id: *token_program_id,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*account, false),
            solana_program::instruction::AccountMeta::new(*mint, false),
            solana_program::instruction::AccountMeta::new_readonly(*authority, true),
        ],
        data,
    })
}
