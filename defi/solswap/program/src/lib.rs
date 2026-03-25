// =============================================================================
// SolSwap AMM DEX — On-chain Program
// =============================================================================
//
// A constant-product automated market maker (x * y = k) for the SolClone
// blockchain.  Supports pool creation, bilateral liquidity provision,
// proportional withdrawal, and fee-aware token swaps with slippage protection.
//
// Fee schedule
// ------------
//   Trade fee   : 25 / 10 000  =  0.25 %   (paid by the swapper)
//   Protocol fee:  5 / 10 000  =  0.05 %   (subset of trade fee, sent to treasury)
//
// LP token minting
// ----------------
//   First deposit : lp_amount = sqrt(amount_a * amount_b)
//   Subsequent    : lp_amount = min(a * supply / reserve_a,
//                                    b * supply / reserve_b)
//
// Swap formula (constant-product with fee)
// ----------------------------------------
//   effective_input = input_amount * (fee_denominator - fee_numerator)
//   output          = effective_input * reserve_out
//                     / (reserve_in * fee_denominator + effective_input)
//
// =============================================================================

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// ---------------------------------------------------------------------------
// Program ID
// ---------------------------------------------------------------------------

solana_program::declare_id!("So1Swap1111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SwapInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        SwapInstruction::InitializePool { fee_numerator, fee_denominator, protocol_fee_numerator } => {
            process_initialize_pool(program_id, accounts, fee_numerator, fee_denominator, protocol_fee_numerator)
        }
        SwapInstruction::AddLiquidity { amount_a, amount_b, min_lp_tokens } => {
            process_add_liquidity(program_id, accounts, amount_a, amount_b, min_lp_tokens)
        }
        SwapInstruction::RemoveLiquidity { lp_amount, min_amount_a, min_amount_b } => {
            process_remove_liquidity(program_id, accounts, lp_amount, min_amount_a, min_amount_b)
        }
        SwapInstruction::Swap { amount_in, minimum_amount_out, swap_direction } => {
            process_swap(program_id, accounts, amount_in, minimum_amount_out, swap_direction)
        }
    }
}

// =============================================================================
// Instruction Enum
// =============================================================================

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum SwapInstruction {
    /// Create a new liquidity pool for a token pair.
    ///
    /// Accounts expected:
    ///   0. `[writable]` Pool state account (PDA)
    ///   1. `[]`         Token A mint
    ///   2. `[]`         Token B mint
    ///   3. `[writable]` Token A reserve vault
    ///   4. `[writable]` Token B reserve vault
    ///   5. `[writable]` LP token mint
    ///   6. `[signer]`   Pool authority / creator
    ///   7. `[]`         System program
    ///   8. `[]`         Token program
    ///   9. `[]`         Rent sysvar
    InitializePool {
        fee_numerator: u64,
        fee_denominator: u64,
        protocol_fee_numerator: u64,
    },

    /// Deposit both tokens into the pool and receive LP tokens.
    ///
    /// Accounts expected:
    ///   0. `[writable]` Pool state account
    ///   1. `[writable]` Token A reserve vault
    ///   2. `[writable]` Token B reserve vault
    ///   3. `[writable]` LP token mint
    ///   4. `[writable]` User token A account
    ///   5. `[writable]` User token B account
    ///   6. `[writable]` User LP token account
    ///   7. `[signer]`   User authority
    ///   8. `[]`         Token program
    AddLiquidity {
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    },

    /// Burn LP tokens and withdraw proportional tokens from the pool.
    ///
    /// Accounts expected:
    ///   0. `[writable]` Pool state account
    ///   1. `[writable]` Token A reserve vault
    ///   2. `[writable]` Token B reserve vault
    ///   3. `[writable]` LP token mint
    ///   4. `[writable]` User token A account
    ///   5. `[writable]` User token B account
    ///   6. `[writable]` User LP token account
    ///   7. `[signer]`   User authority
    ///   8. `[]`         Token program
    RemoveLiquidity {
        lp_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    },

    /// Swap one token for another using the constant-product formula.
    ///
    /// Accounts expected:
    ///   0. `[writable]` Pool state account
    ///   1. `[writable]` Source token reserve vault
    ///   2. `[writable]` Destination token reserve vault
    ///   3. `[writable]` User source token account
    ///   4. `[writable]` User destination token account
    ///   5. `[writable]` Protocol fee destination account
    ///   6. `[signer]`   User authority
    ///   7. `[]`         Token program
    Swap {
        amount_in: u64,
        minimum_amount_out: u64,
        swap_direction: SwapDirection,
    },
}

// =============================================================================
// Swap Direction
// =============================================================================

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapDirection {
    AtoB,
    BtoA,
}

// =============================================================================
// Pool State
// =============================================================================

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PoolState {
    /// Discriminator / magic byte to identify account type.
    pub is_initialized: bool,

    /// Bump seed used to derive the pool PDA.
    pub bump: u8,

    /// Mint address of token A.
    pub token_a_mint: Pubkey,

    /// Mint address of token B.
    pub token_b_mint: Pubkey,

    /// Current reserve of token A held by the pool vault.
    pub token_a_reserve: u64,

    /// Current reserve of token B held by the pool vault.
    pub token_b_reserve: u64,

    /// Mint address of the LP token issued to liquidity providers.
    pub lp_mint: Pubkey,

    /// Total supply of LP tokens currently outstanding.
    pub lp_supply: u64,

    /// Trade fee numerator (default 25 -> 0.25%).
    pub fee_numerator: u64,

    /// Trade fee denominator (default 10 000).
    pub fee_denominator: u64,

    /// Protocol fee numerator — fraction of the trade fee routed to treasury
    /// (default 5 -> 0.05%).
    pub protocol_fee_numerator: u64,

    /// Authority pubkey that can adjust parameters.
    pub authority: Pubkey,

    /// Cumulative volume of token A swapped through this pool (for analytics).
    pub cumulative_volume_a: u128,

    /// Cumulative volume of token B swapped through this pool (for analytics).
    pub cumulative_volume_b: u128,
}

impl PoolState {
    pub const LEN: usize = 1    // is_initialized
        + 1                     // bump
        + 32                    // token_a_mint
        + 32                    // token_b_mint
        + 8                     // token_a_reserve
        + 8                     // token_b_reserve
        + 32                    // lp_mint
        + 8                     // lp_supply
        + 8                     // fee_numerator
        + 8                     // fee_denominator
        + 8                     // protocol_fee_numerator
        + 32                    // authority
        + 16                    // cumulative_volume_a
        + 16;                   // cumulative_volume_b

    pub const DEFAULT_FEE_NUMERATOR: u64 = 25;
    pub const DEFAULT_FEE_DENOMINATOR: u64 = 10_000;
    pub const DEFAULT_PROTOCOL_FEE_NUMERATOR: u64 = 5;
}

// =============================================================================
// Error Enum
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapError {
    /// Pool has already been initialized.
    AlreadyInitialized,
    /// Pool has not been initialized yet.
    NotInitialized,
    /// Provided fee configuration is invalid.
    InvalidFeeConfig,
    /// Deposit amounts must both be greater than zero.
    ZeroDeposit,
    /// Swap input amount must be greater than zero.
    ZeroSwapInput,
    /// Computed output fell below the caller's minimum (slippage exceeded).
    SlippageExceeded,
    /// LP token amount for withdrawal must be greater than zero.
    ZeroLpBurn,
    /// LP burn amount exceeds the caller's balance.
    InsufficientLpBalance,
    /// Pool reserves are empty; cannot swap.
    EmptyReserves,
    /// Arithmetic overflow during calculation.
    MathOverflow,
    /// The two mints provided are identical.
    IdenticalMints,
    /// An account had an unexpected owner.
    InvalidAccountOwner,
    /// LP tokens minted would be below the user's minimum.
    InsufficientLpMinted,
}

impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl std::fmt::Display for SwapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyInitialized    => write!(f, "Pool already initialized"),
            Self::NotInitialized        => write!(f, "Pool not initialized"),
            Self::InvalidFeeConfig      => write!(f, "Invalid fee configuration"),
            Self::ZeroDeposit           => write!(f, "Deposit amounts must be > 0"),
            Self::ZeroSwapInput         => write!(f, "Swap input must be > 0"),
            Self::SlippageExceeded      => write!(f, "Slippage tolerance exceeded"),
            Self::ZeroLpBurn            => write!(f, "LP burn amount must be > 0"),
            Self::InsufficientLpBalance => write!(f, "Insufficient LP token balance"),
            Self::EmptyReserves         => write!(f, "Pool reserves are empty"),
            Self::MathOverflow          => write!(f, "Math overflow"),
            Self::IdenticalMints        => write!(f, "Token mints must differ"),
            Self::InvalidAccountOwner   => write!(f, "Invalid account owner"),
            Self::InsufficientLpMinted  => write!(f, "Insufficient LP tokens minted"),
        }
    }
}

// =============================================================================
// Event Logging
// =============================================================================

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PoolInitializedEvent {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LiquidityAddedEvent {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens_minted: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LiquidityRemovedEvent {
    pub pool: Pubkey,
    pub provider: Pubkey,
    pub lp_tokens_burned: u64,
    pub amount_a_returned: u64,
    pub amount_b_returned: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SwapEvent {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub direction: SwapDirection,
    pub amount_in: u64,
    pub amount_out: u64,
    pub protocol_fee: u64,
}

fn emit_event<T: BorshSerialize>(label: &str, event: &T) {
    if let Ok(data) = borsh::to_vec(event) {
        msg!("EVENT:{} {}", label, bs58_encode(&data));
    }
}

/// Minimal base-58 encoder for event payloads (avoids pulling in the bs58
/// crate).
fn bs58_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if bytes.is_empty() {
        return String::new();
    }

    // Count leading zeros.
    let leading_zeros = bytes.iter().take_while(|&&b| b == 0).count();

    // Work in big-endian u16 digits.
    let mut digits: Vec<u16> = vec![0];
    for &byte in bytes {
        let mut carry = byte as u16;
        for d in digits.iter_mut() {
            carry += (*d) << 8;
            *d = carry % 58;
            carry /= 58;
        }
        while carry > 0 {
            digits.push(carry % 58);
            carry /= 58;
        }
    }

    let mut result = String::with_capacity(leading_zeros + digits.len());
    for _ in 0..leading_zeros {
        result.push('1');
    }
    for &d in digits.iter().rev() {
        result.push(ALPHABET[d as usize] as char);
    }
    result
}

// =============================================================================
// Math Helpers
// =============================================================================

/// Integer square root via Newton's method (Babylonian).
pub fn sqrt(value: u128) -> u64 {
    if value == 0 {
        return 0;
    }
    let mut x = value;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + value / x) / 2;
    }
    x as u64
}

/// Compute the output amount for a constant-product swap **after** fees.
///
/// ```text
/// effective_in = amount_in * (fee_denominator - fee_numerator)
/// amount_out   = effective_in * reserve_out
///                / (reserve_in * fee_denominator + effective_in)
/// ```
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<(u64, u64), SwapError> {
    if amount_in == 0 {
        return Err(SwapError::ZeroSwapInput);
    }
    if reserve_in == 0 || reserve_out == 0 {
        return Err(SwapError::EmptyReserves);
    }

    let amount_in = amount_in as u128;
    let reserve_in = reserve_in as u128;
    let reserve_out = reserve_out as u128;
    let fee_num = fee_numerator as u128;
    let fee_den = fee_denominator as u128;

    let effective_in = amount_in
        .checked_mul(fee_den.checked_sub(fee_num).ok_or(SwapError::MathOverflow)?)
        .ok_or(SwapError::MathOverflow)?;

    let numerator = effective_in
        .checked_mul(reserve_out)
        .ok_or(SwapError::MathOverflow)?;

    let denominator = reserve_in
        .checked_mul(fee_den)
        .ok_or(SwapError::MathOverflow)?
        .checked_add(effective_in)
        .ok_or(SwapError::MathOverflow)?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(SwapError::MathOverflow)? as u64;

    // Protocol fee is a portion of the overall fee.
    let total_fee = amount_in
        .checked_mul(fee_num)
        .ok_or(SwapError::MathOverflow)?
        .checked_div(fee_den)
        .ok_or(SwapError::MathOverflow)? as u64;

    Ok((amount_out, total_fee))
}

/// Calculate LP tokens to mint for a deposit.
///
/// First deposit : `sqrt(amount_a * amount_b)`
/// Subsequent    : `min(a * supply / reserve_a, b * supply / reserve_b)`
pub fn calculate_lp_tokens(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    lp_supply: u64,
) -> Result<u64, SwapError> {
    if amount_a == 0 || amount_b == 0 {
        return Err(SwapError::ZeroDeposit);
    }

    if lp_supply == 0 {
        // First deposit — geometric mean.
        let product = (amount_a as u128)
            .checked_mul(amount_b as u128)
            .ok_or(SwapError::MathOverflow)?;
        let lp = sqrt(product);
        if lp == 0 {
            return Err(SwapError::MathOverflow);
        }
        Ok(lp)
    } else {
        // Proportional deposit.
        let supply = lp_supply as u128;
        let lp_a = (amount_a as u128)
            .checked_mul(supply)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(reserve_a as u128)
            .ok_or(SwapError::MathOverflow)?;
        let lp_b = (amount_b as u128)
            .checked_mul(supply)
            .ok_or(SwapError::MathOverflow)?
            .checked_div(reserve_b as u128)
            .ok_or(SwapError::MathOverflow)?;
        let lp = std::cmp::min(lp_a, lp_b) as u64;
        if lp == 0 {
            return Err(SwapError::MathOverflow);
        }
        Ok(lp)
    }
}

/// Calculate the amount of each underlying token a liquidity provider receives
/// when burning `lp_amount` LP tokens.
pub fn calculate_withdrawal(
    lp_amount: u64,
    lp_supply: u64,
    reserve_a: u64,
    reserve_b: u64,
) -> Result<(u64, u64), SwapError> {
    if lp_amount == 0 {
        return Err(SwapError::ZeroLpBurn);
    }
    let lp = lp_amount as u128;
    let supply = lp_supply as u128;

    let amount_a = lp
        .checked_mul(reserve_a as u128)
        .ok_or(SwapError::MathOverflow)?
        .checked_div(supply)
        .ok_or(SwapError::MathOverflow)? as u64;

    let amount_b = lp
        .checked_mul(reserve_b as u128)
        .ok_or(SwapError::MathOverflow)?
        .checked_div(supply)
        .ok_or(SwapError::MathOverflow)? as u64;

    Ok((amount_a, amount_b))
}

/// Calculate price impact of a swap as basis points (hundredths of a percent).
///
/// ```text
/// spot_price    = reserve_out / reserve_in
/// exec_price    = amount_out  / amount_in
/// impact_bps    = (1 - exec_price / spot_price) * 10 000
/// ```
pub fn calculate_price_impact_bps(
    amount_in: u64,
    amount_out: u64,
    reserve_in: u64,
    reserve_out: u64,
) -> u64 {
    if amount_in == 0 || reserve_in == 0 || reserve_out == 0 {
        return 0;
    }
    // Use u128 to prevent overflow.
    let spot_numerator = (reserve_out as u128) * (amount_in as u128);
    let exec_numerator = (amount_out as u128) * (reserve_in as u128);

    if exec_numerator >= spot_numerator {
        return 0; // No negative impact.
    }

    let diff = spot_numerator - exec_numerator;
    ((diff * 10_000) / spot_numerator) as u64
}

// =============================================================================
// Instruction Processors
// =============================================================================

fn process_initialize_pool(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    fee_numerator: u64,
    fee_denominator: u64,
    protocol_fee_numerator: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let pool_account    = next_account_info(account_iter)?;
    let token_a_mint    = next_account_info(account_iter)?;
    let token_b_mint    = next_account_info(account_iter)?;
    let _vault_a        = next_account_info(account_iter)?;
    let _vault_b        = next_account_info(account_iter)?;
    let lp_mint         = next_account_info(account_iter)?;
    let authority       = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _token_program  = next_account_info(account_iter)?;
    let _rent           = next_account_info(account_iter)?;

    // --- Validations ---

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Fee denominator must be positive and >= numerator.
    if fee_denominator == 0 || fee_numerator > fee_denominator {
        return Err(SwapError::InvalidFeeConfig.into());
    }
    if protocol_fee_numerator > fee_numerator {
        return Err(SwapError::InvalidFeeConfig.into());
    }

    // Mints must differ.
    if token_a_mint.key == token_b_mint.key {
        return Err(SwapError::IdenticalMints.into());
    }

    // Check not already initialized.
    let mut pool_data = pool_account.try_borrow_mut_data()?;
    if pool_data.len() < PoolState::LEN {
        return Err(ProgramError::AccountDataTooSmall);
    }
    if pool_data[0] != 0 {
        return Err(SwapError::AlreadyInitialized.into());
    }

    // --- Write state ---

    let pool = PoolState {
        is_initialized: true,
        bump: 0, // Set by the client when deriving the PDA.
        token_a_mint: *token_a_mint.key,
        token_b_mint: *token_b_mint.key,
        token_a_reserve: 0,
        token_b_reserve: 0,
        lp_mint: *lp_mint.key,
        lp_supply: 0,
        fee_numerator,
        fee_denominator,
        protocol_fee_numerator,
        authority: *authority.key,
        cumulative_volume_a: 0,
        cumulative_volume_b: 0,
    };

    let encoded = borsh::to_vec(&pool).map_err(|_| ProgramError::InvalidAccountData)?;
    pool_data[..encoded.len()].copy_from_slice(&encoded);

    // --- Emit event ---

    emit_event("PoolInitialized", &PoolInitializedEvent {
        pool: *pool_account.key,
        token_a_mint: *token_a_mint.key,
        token_b_mint: *token_b_mint.key,
        fee_numerator,
        fee_denominator,
    });

    msg!(
        "SolSwap: Pool initialized  token_a={} token_b={} fee={}/{}",
        token_a_mint.key,
        token_b_mint.key,
        fee_numerator,
        fee_denominator
    );

    Ok(())
}

fn process_add_liquidity(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_a: u64,
    amount_b: u64,
    min_lp_tokens: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let pool_account = next_account_info(account_iter)?;
    let _vault_a     = next_account_info(account_iter)?;
    let _vault_b     = next_account_info(account_iter)?;
    let _lp_mint     = next_account_info(account_iter)?;
    let _user_a      = next_account_info(account_iter)?;
    let _user_b      = next_account_info(account_iter)?;
    let _user_lp     = next_account_info(account_iter)?;
    let authority     = next_account_info(account_iter)?;
    let _token_prog   = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // --- Deserialize pool ---

    let mut pool_data = pool_account.try_borrow_mut_data()?;
    let mut pool = PoolState::try_from_slice(&pool_data[..PoolState::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(SwapError::NotInitialized.into());
    }

    // --- Calculate LP tokens ---

    let lp_tokens = calculate_lp_tokens(
        amount_a,
        amount_b,
        pool.token_a_reserve,
        pool.token_b_reserve,
        pool.lp_supply,
    )?;

    if lp_tokens < min_lp_tokens {
        return Err(SwapError::InsufficientLpMinted.into());
    }

    // --- Update reserves ---

    pool.token_a_reserve = pool
        .token_a_reserve
        .checked_add(amount_a)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
    pool.token_b_reserve = pool
        .token_b_reserve
        .checked_add(amount_b)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
    pool.lp_supply = pool
        .lp_supply
        .checked_add(lp_tokens)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;

    let encoded = borsh::to_vec(&pool).map_err(|_| ProgramError::InvalidAccountData)?;
    pool_data[..encoded.len()].copy_from_slice(&encoded);

    // In a real implementation we would invoke spl_token::transfer CPI here to
    // move tokens from the user into the vault, and spl_token::mint_to to mint
    // LP tokens to the user.

    emit_event("LiquidityAdded", &LiquidityAddedEvent {
        pool: *pool_account.key,
        provider: *authority.key,
        amount_a,
        amount_b,
        lp_tokens_minted: lp_tokens,
    });

    msg!(
        "SolSwap: Liquidity added  a={} b={} lp_minted={}",
        amount_a,
        amount_b,
        lp_tokens
    );

    Ok(())
}

fn process_remove_liquidity(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    lp_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let pool_account = next_account_info(account_iter)?;
    let _vault_a     = next_account_info(account_iter)?;
    let _vault_b     = next_account_info(account_iter)?;
    let _lp_mint     = next_account_info(account_iter)?;
    let _user_a      = next_account_info(account_iter)?;
    let _user_b      = next_account_info(account_iter)?;
    let _user_lp     = next_account_info(account_iter)?;
    let authority     = next_account_info(account_iter)?;
    let _token_prog   = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool_data = pool_account.try_borrow_mut_data()?;
    let mut pool = PoolState::try_from_slice(&pool_data[..PoolState::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(SwapError::NotInitialized.into());
    }

    if lp_amount > pool.lp_supply {
        return Err(SwapError::InsufficientLpBalance.into());
    }

    let (amount_a, amount_b) = calculate_withdrawal(
        lp_amount,
        pool.lp_supply,
        pool.token_a_reserve,
        pool.token_b_reserve,
    )?;

    // Slippage checks.
    if amount_a < min_amount_a || amount_b < min_amount_b {
        return Err(SwapError::SlippageExceeded.into());
    }

    // --- Update state ---

    pool.token_a_reserve = pool
        .token_a_reserve
        .checked_sub(amount_a)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
    pool.token_b_reserve = pool
        .token_b_reserve
        .checked_sub(amount_b)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
    pool.lp_supply = pool
        .lp_supply
        .checked_sub(lp_amount)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;

    let encoded = borsh::to_vec(&pool).map_err(|_| ProgramError::InvalidAccountData)?;
    pool_data[..encoded.len()].copy_from_slice(&encoded);

    emit_event("LiquidityRemoved", &LiquidityRemovedEvent {
        pool: *pool_account.key,
        provider: *authority.key,
        lp_tokens_burned: lp_amount,
        amount_a_returned: amount_a,
        amount_b_returned: amount_b,
    });

    msg!(
        "SolSwap: Liquidity removed  lp_burned={} a={} b={}",
        lp_amount,
        amount_a,
        amount_b
    );

    Ok(())
}

fn process_swap(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount_in: u64,
    minimum_amount_out: u64,
    swap_direction: SwapDirection,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();

    let pool_account  = next_account_info(account_iter)?;
    let _vault_src    = next_account_info(account_iter)?;
    let _vault_dst    = next_account_info(account_iter)?;
    let _user_src     = next_account_info(account_iter)?;
    let _user_dst     = next_account_info(account_iter)?;
    let _protocol_dst = next_account_info(account_iter)?;
    let authority      = next_account_info(account_iter)?;
    let _token_prog    = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool_data = pool_account.try_borrow_mut_data()?;
    let mut pool = PoolState::try_from_slice(&pool_data[..PoolState::LEN])
        .map_err(|_| ProgramError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(SwapError::NotInitialized.into());
    }

    let (reserve_in, reserve_out) = match swap_direction {
        SwapDirection::AtoB => (pool.token_a_reserve, pool.token_b_reserve),
        SwapDirection::BtoA => (pool.token_b_reserve, pool.token_a_reserve),
    };

    // --- Calculate output ---

    let (amount_out, _total_fee) = calculate_swap_output(
        amount_in,
        reserve_in,
        reserve_out,
        pool.fee_numerator,
        pool.fee_denominator,
    )?;

    // Slippage guard.
    if amount_out < minimum_amount_out {
        return Err(SwapError::SlippageExceeded.into());
    }

    // Protocol fee (subset of the trade fee, retained in the reserves so it
    // accrues to protocol-owned LP tokens).
    let protocol_fee = (amount_in as u128)
        .checked_mul(pool.protocol_fee_numerator as u128)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())?
        .checked_div(pool.fee_denominator as u128)
        .ok_or::<ProgramError>(SwapError::MathOverflow.into())? as u64;

    // --- Update reserves ---

    match swap_direction {
        SwapDirection::AtoB => {
            pool.token_a_reserve = pool
                .token_a_reserve
                .checked_add(amount_in)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
            pool.token_b_reserve = pool
                .token_b_reserve
                .checked_sub(amount_out)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
            pool.cumulative_volume_a = pool
                .cumulative_volume_a
                .checked_add(amount_in as u128)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
        }
        SwapDirection::BtoA => {
            pool.token_b_reserve = pool
                .token_b_reserve
                .checked_add(amount_in)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
            pool.token_a_reserve = pool
                .token_a_reserve
                .checked_sub(amount_out)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
            pool.cumulative_volume_b = pool
                .cumulative_volume_b
                .checked_add(amount_in as u128)
                .ok_or::<ProgramError>(SwapError::MathOverflow.into())?;
        }
    }

    let encoded = borsh::to_vec(&pool).map_err(|_| ProgramError::InvalidAccountData)?;
    pool_data[..encoded.len()].copy_from_slice(&encoded);

    emit_event("Swap", &SwapEvent {
        pool: *pool_account.key,
        user: *authority.key,
        direction: swap_direction,
        amount_in,
        amount_out,
        protocol_fee,
    });

    msg!(
        "SolSwap: Swap {:?}  in={} out={} protocol_fee={}",
        swap_direction,
        amount_in,
        amount_out,
        protocol_fee
    );

    Ok(())
}

// =============================================================================
// Unit Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqrt() {
        assert_eq!(sqrt(0), 0);
        assert_eq!(sqrt(1), 1);
        assert_eq!(sqrt(4), 2);
        assert_eq!(sqrt(9), 3);
        assert_eq!(sqrt(100), 10);
        assert_eq!(sqrt(1_000_000), 1_000);
        // Non-perfect square floors correctly.
        assert_eq!(sqrt(2), 1);
        assert_eq!(sqrt(8), 2);
    }

    #[test]
    fn test_calculate_swap_output_basic() {
        // Pool: 1 000 A / 2 000 B, fee = 25/10000
        let (out, _fee) = calculate_swap_output(100, 1_000, 2_000, 25, 10_000).unwrap();
        // Without fee: 100 * 2000 / (1000 + 100) = 181.81 -> ~181
        // With fee:    effective = 100 * 9975 = 997500
        //              out = 997500 * 2000 / (1000 * 10000 + 997500) = 1995000000 / 10997500 = ~181
        assert!(out > 0 && out < 200);
    }

    #[test]
    fn test_calculate_swap_output_zero_input() {
        let result = calculate_swap_output(0, 1_000, 2_000, 25, 10_000);
        assert_eq!(result, Err(SwapError::ZeroSwapInput));
    }

    #[test]
    fn test_calculate_swap_output_empty_reserves() {
        let result = calculate_swap_output(100, 0, 2_000, 25, 10_000);
        assert_eq!(result, Err(SwapError::EmptyReserves));
    }

    #[test]
    fn test_calculate_lp_tokens_first_deposit() {
        let lp = calculate_lp_tokens(1_000, 4_000, 0, 0, 0).unwrap();
        // sqrt(1000 * 4000) = sqrt(4_000_000) = 2000
        assert_eq!(lp, 2_000);
    }

    #[test]
    fn test_calculate_lp_tokens_subsequent() {
        // Pool: 1000 A / 2000 B / 1000 LP supply
        let lp = calculate_lp_tokens(100, 200, 1_000, 2_000, 1_000).unwrap();
        // min(100*1000/1000, 200*1000/2000) = min(100, 100) = 100
        assert_eq!(lp, 100);
    }

    #[test]
    fn test_calculate_withdrawal() {
        // Burn 500 of 2000 LP. Reserves: 10 000 A / 20 000 B.
        let (a, b) = calculate_withdrawal(500, 2_000, 10_000, 20_000).unwrap();
        assert_eq!(a, 2_500);
        assert_eq!(b, 5_000);
    }

    #[test]
    fn test_price_impact() {
        let impact = calculate_price_impact_bps(100, 181, 1_000, 2_000);
        // Spot: 2.0, Exec: 1.81, impact ~9.5% = 950 bps
        assert!(impact > 0);
    }

    #[test]
    fn test_pool_state_size() {
        // Ensure our LEN constant is correct.
        assert_eq!(PoolState::LEN, 200);
    }
}
