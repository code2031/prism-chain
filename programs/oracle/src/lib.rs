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

solana_program::declare_id!("Orac1e111111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of authorized oracle data sources per feed.
const MAX_ORACLES: usize = 16;

/// Maximum staleness: reject price data older than 60 seconds.
const MAX_STALENESS_SECONDS: i64 = 60;

/// Default staleness threshold if not overridden per feed.
const DEFAULT_STALENESS: i64 = 30;

/// Minimum number of oracle submissions required for a valid aggregate.
const MIN_SUBMISSIONS: usize = 3;

/// Maximum number of historical price rounds to store per feed.
const MAX_HISTORY: usize = 32;

/// Seed for price feed PDA.
const FEED_SEED: &[u8] = b"price_feed";

/// Seed for oracle config PDA.
const CONFIG_SEED: &[u8] = b"oracle_config";

/// Price precision: prices are stored with 8 decimal places.
/// e.g., $42,000.12345678 = 4_200_012_345_678
const PRICE_DECIMALS: u8 = 8;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum OracleError {
    #[error("Feed has already been initialized")]
    AlreadyInitialized,

    #[error("Feed is not initialized")]
    NotInitialized,

    #[error("Oracle is not authorized to submit to this feed")]
    UnauthorizedOracle,

    #[error("Price data is stale (exceeds max age)")]
    StaleData,

    #[error("Not enough oracle submissions to compute aggregate")]
    InsufficientSubmissions,

    #[error("Price must be greater than zero")]
    InvalidPrice,

    #[error("Confidence interval is invalid")]
    InvalidConfidence,

    #[error("Maximum number of oracles reached for this feed")]
    MaxOraclesReached,

    #[error("Oracle already registered for this feed")]
    OracleAlreadyRegistered,

    #[error("Unauthorized: only admin can perform this action")]
    Unauthorized,

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Invalid PDA derivation")]
    InvalidPDA,

    #[error("Invalid asset class")]
    InvalidAssetClass,
}

impl From<OracleError> for ProgramError {
    fn from(e: OracleError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Asset class for categorizing price feeds.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum AssetClass {
    /// Cryptocurrency (BTC, ETH, PRISM, etc.)
    Crypto = 0,
    /// Foreign exchange (EUR/USD, GBP/USD, etc.)
    Forex = 1,
    /// Commodities (Gold, Silver, Oil, etc.)
    Commodity = 2,
    /// Equity/Stock indices
    Equity = 3,
}

/// A single oracle's price submission for a given round.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct OracleSubmission {
    /// Oracle that submitted this price.
    pub oracle: Pubkey,
    /// Price in fixed-point with PRICE_DECIMALS decimals.
    pub price: i128,
    /// Confidence interval (half-width) in same units as price.
    /// e.g., if price = 42000_00000000 and confidence = 50_00000000,
    /// the price is $42,000 +/- $50.
    pub confidence: u64,
    /// Unix timestamp of the observation (from the oracle's data source).
    pub timestamp: i64,
    /// Unix timestamp when submitted on-chain.
    pub slot_timestamp: i64,
}

/// A completed price round with aggregated data.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PriceRound {
    /// Round number (monotonically increasing).
    pub round_id: u64,
    /// Aggregated median price.
    pub price: i128,
    /// Aggregated confidence interval.
    pub confidence: u64,
    /// Number of oracle submissions in this round.
    pub num_submissions: u32,
    /// Timestamp of this round's aggregation.
    pub timestamp: i64,
}

/// On-chain state for a price feed.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PriceFeed {
    /// Whether this feed has been initialized.
    pub is_initialized: bool,
    /// Feed admin who can add/remove oracles.
    pub admin: Pubkey,
    /// Human-readable symbol (e.g., "PRISM/USD"). Max 32 bytes.
    pub symbol: [u8; 32],
    /// Asset class.
    pub asset_class: AssetClass,
    /// Number of decimal places in the price.
    pub decimals: u8,
    /// Maximum allowed staleness in seconds.
    pub max_staleness: i64,
    /// Minimum number of submissions required for aggregation.
    pub min_submissions: u32,
    /// Current round number.
    pub current_round: u64,
    /// Latest aggregated price.
    pub latest_price: i128,
    /// Latest aggregated confidence.
    pub latest_confidence: u64,
    /// Timestamp of the latest aggregation.
    pub latest_timestamp: i64,
    /// Whether the feed is active.
    pub is_active: bool,
    /// PDA bump seed.
    pub bump: u8,
    /// Number of registered oracles.
    pub oracle_count: u32,
    /// Authorized oracle pubkeys.
    pub oracles: Vec<Pubkey>,
    /// Current round's submissions (cleared each round).
    pub current_submissions: Vec<OracleSubmission>,
    /// Historical price rounds (circular buffer).
    pub history: Vec<PriceRound>,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum OracleInstruction {
    /// Initialize a new price feed.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Admin (payer)
    ///   1. `[writable]`         PriceFeed PDA
    ///   2. `[]`                 System program
    InitializeFeed {
        /// Symbol bytes (e.g., b"PRISM/USD" padded to 32).
        symbol: [u8; 32],
        /// Asset class.
        asset_class: AssetClass,
        /// Max staleness in seconds (0 = use default).
        max_staleness: i64,
        /// Minimum oracle submissions for valid aggregation.
        min_submissions: u32,
    },

    /// Register an authorized oracle for a feed.
    ///
    /// Accounts:
    ///   0. `[signer]`           Admin
    ///   1. `[writable]`         PriceFeed PDA
    AddOracle {
        oracle: Pubkey,
    },

    /// Remove an oracle from a feed.
    ///
    /// Accounts:
    ///   0. `[signer]`           Admin
    ///   1. `[writable]`         PriceFeed PDA
    RemoveOracle {
        oracle: Pubkey,
    },

    /// Submit a price observation. If enough submissions are collected,
    /// automatically triggers aggregation.
    ///
    /// Accounts:
    ///   0. `[signer]`           Oracle
    ///   1. `[writable]`         PriceFeed PDA
    SubmitPrice {
        /// Price value (fixed-point with PRICE_DECIMALS decimals).
        price: i128,
        /// Confidence half-width in same units.
        confidence: u64,
        /// Observation timestamp from data source.
        timestamp: i64,
    },

    /// Force aggregation of current submissions (admin only).
    ///
    /// Accounts:
    ///   0. `[signer]`           Admin
    ///   1. `[writable]`         PriceFeed PDA
    ForceAggregate,

    /// Read the latest price (convenience instruction that just logs).
    ///
    /// Accounts:
    ///   0. `[]`                 PriceFeed PDA
    ReadPrice,
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
    let instruction = OracleInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        OracleInstruction::InitializeFeed {
            symbol,
            asset_class,
            max_staleness,
            min_submissions,
        } => process_initialize_feed(
            program_id,
            accounts,
            symbol,
            asset_class,
            max_staleness,
            min_submissions,
        ),
        OracleInstruction::AddOracle { oracle } => {
            process_add_oracle(program_id, accounts, oracle)
        }
        OracleInstruction::RemoveOracle { oracle } => {
            process_remove_oracle(program_id, accounts, oracle)
        }
        OracleInstruction::SubmitPrice {
            price,
            confidence,
            timestamp,
        } => process_submit_price(program_id, accounts, price, confidence, timestamp),
        OracleInstruction::ForceAggregate => {
            process_force_aggregate(program_id, accounts)
        }
        OracleInstruction::ReadPrice => process_read_price(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the median of a slice of i128 values.
/// The slice must be non-empty. For even-length slices, returns the lower median.
fn median(values: &mut [i128]) -> i128 {
    values.sort();
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        // Average of two middle values
        (values[mid - 1] + values[mid]) / 2
    } else {
        values[mid]
    }
}

/// Compute a weighted confidence interval from individual oracle confidences.
/// Uses the maximum confidence as a conservative estimate, then adds
/// a spread component based on the range of submitted prices.
fn aggregate_confidence(submissions: &[OracleSubmission], median_price: i128) -> u64 {
    if submissions.is_empty() {
        return 0;
    }

    // Base: max individual confidence
    let max_confidence = submissions
        .iter()
        .map(|s| s.confidence)
        .max()
        .unwrap_or(0);

    // Spread: maximum absolute deviation from median
    let max_deviation = submissions
        .iter()
        .map(|s| {
            let diff = s.price - median_price;
            (if diff < 0 { -diff } else { diff }) as u64
        })
        .max()
        .unwrap_or(0);

    // Combined confidence: max(individual confidence, price spread)
    max_confidence.max(max_deviation)
}

/// Perform aggregation on the current submissions.
fn aggregate_prices(feed: &mut PriceFeed, clock_timestamp: i64) -> ProgramResult {
    let submission_count = feed.current_submissions.len();
    if submission_count < feed.min_submissions as usize {
        return Err(OracleError::InsufficientSubmissions.into());
    }

    // Extract prices and compute median
    let mut prices: Vec<i128> = feed
        .current_submissions
        .iter()
        .map(|s| s.price)
        .collect();
    let median_price = median(&mut prices);

    // Compute aggregate confidence
    let confidence = aggregate_confidence(&feed.current_submissions, median_price);

    // Advance round
    feed.current_round = feed.current_round.saturating_add(1);

    // Store in history (circular buffer)
    let round = PriceRound {
        round_id: feed.current_round,
        price: median_price,
        confidence,
        num_submissions: submission_count as u32,
        timestamp: clock_timestamp,
    };

    if feed.history.len() >= MAX_HISTORY {
        // Overwrite oldest entry
        let idx = (feed.current_round as usize - 1) % MAX_HISTORY;
        feed.history[idx] = round;
    } else {
        feed.history.push(round);
    }

    // Update latest values
    feed.latest_price = median_price;
    feed.latest_confidence = confidence;
    feed.latest_timestamp = clock_timestamp;

    // Clear submissions for next round
    feed.current_submissions.clear();

    msg!(
        "Price aggregated: round={}, price={}, confidence={}, sources={}",
        feed.current_round,
        median_price,
        confidence,
        submission_count
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_feed(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    symbol: [u8; 32],
    asset_class: AssetClass,
    max_staleness: i64,
    min_submissions: u32,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let feed_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive feed PDA from symbol
    let (feed_pda, feed_bump) = Pubkey::find_program_address(
        &[FEED_SEED, &symbol],
        program_id,
    );
    if feed_pda != *feed_account.key {
        return Err(OracleError::InvalidPDA.into());
    }

    let staleness = if max_staleness <= 0 {
        DEFAULT_STALENESS
    } else {
        max_staleness.min(MAX_STALENESS_SECONDS)
    };

    let min_subs = if min_submissions == 0 {
        MIN_SUBMISSIONS as u32
    } else {
        min_submissions
    };

    // Allocate space for the feed
    let rent = Rent::get()?;
    // Conservative estimate: base fields + MAX_ORACLES pubkeys
    // + MAX_ORACLES submissions + MAX_HISTORY rounds
    let space: usize = 2048 + (MAX_ORACLES * 32) + (MAX_ORACLES * 128) + (MAX_HISTORY * 64);
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            admin.key,
            feed_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[admin.clone(), feed_account.clone(), system_program.clone()],
        &[&[FEED_SEED, &symbol, &[feed_bump]]],
    )?;

    let feed = PriceFeed {
        is_initialized: true,
        admin: *admin.key,
        symbol,
        asset_class,
        decimals: PRICE_DECIMALS,
        max_staleness: staleness,
        min_submissions: min_subs,
        current_round: 0,
        latest_price: 0,
        latest_confidence: 0,
        latest_timestamp: 0,
        is_active: true,
        bump: feed_bump,
        oracle_count: 0,
        oracles: Vec::new(),
        current_submissions: Vec::new(),
        history: Vec::new(),
    };

    feed.serialize(&mut &mut feed_account.data.borrow_mut()[..])?;

    // Log the symbol as a string for debugging
    let symbol_str = core::str::from_utf8(&symbol)
        .unwrap_or("???")
        .trim_end_matches('\0');
    msg!("Price feed initialized: {}", symbol_str);

    Ok(())
}

fn process_add_oracle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    oracle: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let feed_account = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if feed_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_account.data.borrow())?;
    if !feed.is_initialized {
        return Err(OracleError::NotInitialized.into());
    }
    if feed.admin != *admin.key {
        return Err(OracleError::Unauthorized.into());
    }
    if feed.oracles.len() >= MAX_ORACLES {
        return Err(OracleError::MaxOraclesReached.into());
    }
    if feed.oracles.contains(&oracle) {
        return Err(OracleError::OracleAlreadyRegistered.into());
    }

    feed.oracles.push(oracle);
    feed.oracle_count = feed.oracles.len() as u32;

    feed.serialize(&mut &mut feed_account.data.borrow_mut()[..])?;

    msg!("Oracle added to feed: {}", oracle);

    Ok(())
}

fn process_remove_oracle(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    oracle: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let feed_account = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if feed_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_account.data.borrow())?;
    if !feed.is_initialized {
        return Err(OracleError::NotInitialized.into());
    }
    if feed.admin != *admin.key {
        return Err(OracleError::Unauthorized.into());
    }

    let initial_len = feed.oracles.len();
    feed.oracles.retain(|o| *o != oracle);
    if feed.oracles.len() == initial_len {
        return Err(OracleError::UnauthorizedOracle.into());
    }
    feed.oracle_count = feed.oracles.len() as u32;

    // Also remove any pending submissions from this oracle
    feed.current_submissions.retain(|s| s.oracle != oracle);

    feed.serialize(&mut &mut feed_account.data.borrow_mut()[..])?;

    msg!("Oracle removed from feed: {}", oracle);

    Ok(())
}

fn process_submit_price(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: i128,
    confidence: u64,
    timestamp: i64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let oracle = next_account_info(account_iter)?;
    let feed_account = next_account_info(account_iter)?;

    if !oracle.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if feed_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_account.data.borrow())?;
    if !feed.is_initialized || !feed.is_active {
        return Err(OracleError::NotInitialized.into());
    }

    // Verify oracle is authorized
    if !feed.oracles.contains(oracle.key) {
        return Err(OracleError::UnauthorizedOracle.into());
    }

    // Validate price
    if price <= 0 {
        return Err(OracleError::InvalidPrice.into());
    }

    // Staleness check: reject data that is too old
    let clock = Clock::get()?;
    let age = clock.unix_timestamp.saturating_sub(timestamp);
    if age > feed.max_staleness {
        return Err(OracleError::StaleData.into());
    }

    // Check for duplicate submission in current round
    // (an oracle can only submit once per round)
    if feed.current_submissions.iter().any(|s| s.oracle == *oracle.key) {
        // Replace the existing submission
        for sub in feed.current_submissions.iter_mut() {
            if sub.oracle == *oracle.key {
                sub.price = price;
                sub.confidence = confidence;
                sub.timestamp = timestamp;
                sub.slot_timestamp = clock.unix_timestamp;
                break;
            }
        }
    } else {
        feed.current_submissions.push(OracleSubmission {
            oracle: *oracle.key,
            price,
            confidence,
            timestamp,
            slot_timestamp: clock.unix_timestamp,
        });
    }

    msg!(
        "Price submitted: oracle={}, price={}, confidence={}, age={}s",
        oracle.key,
        price,
        confidence,
        age
    );

    // Auto-aggregate if we have enough submissions
    if feed.current_submissions.len() >= feed.min_submissions as usize {
        aggregate_prices(&mut feed, clock.unix_timestamp)?;
    }

    feed.serialize(&mut &mut feed_account.data.borrow_mut()[..])?;

    Ok(())
}

fn process_force_aggregate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let admin = next_account_info(account_iter)?;
    let feed_account = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    if feed_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_account.data.borrow())?;
    if !feed.is_initialized {
        return Err(OracleError::NotInitialized.into());
    }
    if feed.admin != *admin.key {
        return Err(OracleError::Unauthorized.into());
    }

    let clock = Clock::get()?;
    aggregate_prices(&mut feed, clock.unix_timestamp)?;

    feed.serialize(&mut &mut feed_account.data.borrow_mut()[..])?;

    Ok(())
}

fn process_read_price(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let feed_account = next_account_info(account_iter)?;

    if feed_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let feed = PriceFeed::try_from_slice(&feed_account.data.borrow())?;
    if !feed.is_initialized {
        return Err(OracleError::NotInitialized.into());
    }

    let symbol_str = core::str::from_utf8(&feed.symbol)
        .unwrap_or("???")
        .trim_end_matches('\0');

    // Staleness check on read
    let clock = Clock::get()?;
    let age = clock.unix_timestamp.saturating_sub(feed.latest_timestamp);

    if age > feed.max_staleness && feed.latest_timestamp > 0 {
        msg!(
            "WARNING: Price for {} is stale ({}s old, max {}s)",
            symbol_str,
            age,
            feed.max_staleness
        );
    }

    msg!(
        "Price feed {}: price={}, confidence={}, round={}, age={}s, sources={}",
        symbol_str,
        feed.latest_price,
        feed.latest_confidence,
        feed.current_round,
        age,
        feed.oracle_count
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
    fn test_median_odd() {
        let mut vals = vec![100i128, 300, 200];
        assert_eq!(median(&mut vals), 200);
    }

    #[test]
    fn test_median_even() {
        let mut vals = vec![100i128, 200, 300, 400];
        assert_eq!(median(&mut vals), 250);
    }

    #[test]
    fn test_median_single() {
        let mut vals = vec![42i128];
        assert_eq!(median(&mut vals), 42);
    }

    #[test]
    fn test_median_duplicates() {
        let mut vals = vec![100i128, 100, 100, 200, 300];
        assert_eq!(median(&mut vals), 100);
    }

    #[test]
    fn test_aggregate_confidence() {
        let submissions = vec![
            OracleSubmission {
                oracle: Pubkey::new_unique(),
                price: 42000_00000000,
                confidence: 50_00000000,
                timestamp: 1000,
                slot_timestamp: 1001,
            },
            OracleSubmission {
                oracle: Pubkey::new_unique(),
                price: 42010_00000000,
                confidence: 30_00000000,
                timestamp: 1000,
                slot_timestamp: 1001,
            },
            OracleSubmission {
                oracle: Pubkey::new_unique(),
                price: 41990_00000000,
                confidence: 40_00000000,
                timestamp: 1000,
                slot_timestamp: 1001,
            },
        ];

        let median_price = 42000_00000000i128;
        let conf = aggregate_confidence(&submissions, median_price);
        // max confidence = 50_00000000, max deviation = 10_00000000
        // result = max(50_00000000, 10_00000000) = 50_00000000
        assert_eq!(conf, 50_00000000);
    }

    #[test]
    fn test_asset_class_variants() {
        assert_ne!(AssetClass::Crypto as u8, AssetClass::Forex as u8);
        assert_ne!(AssetClass::Forex as u8, AssetClass::Commodity as u8);
        assert_ne!(AssetClass::Commodity as u8, AssetClass::Equity as u8);
    }
}
