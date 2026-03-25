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

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Price values use 8 decimal places of precision.
/// e.g. $1.00 = 100_000_000, $45_123.99 = 4_512_399_000_000
pub const PRICE_DECIMALS: u64 = 100_000_000;

/// Maximum number of authorized publishers per feed
const MAX_PUBLISHERS: usize = 16;

/// Maximum staleness: prices older than 120 slots (~1 minute) are stale
const MAX_STALENESS_SLOTS: u64 = 120;

/// Seed prefix for price feed PDAs
const FEED_SEED: &[u8] = b"price_feed";

/// Seed prefix for publisher submission PDAs
const SUBMISSION_SEED: &[u8] = b"submission";

// ---------------------------------------------------------------------------
// Supported asset pairs
// ---------------------------------------------------------------------------

/// Well-known asset identifiers
pub mod assets {
    pub const SCLONE_USD: &str = "SCLONE/USD";
    pub const SCUSD_USD: &str = "SCUSD/USD";
    pub const BTC_USD: &str = "BTC/USD";
    pub const ETH_USD: &str = "ETH/USD";
}

// ---------------------------------------------------------------------------
// State accounts
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PriceFeed {
    /// Human-readable asset pair identifier (e.g. "SCLONE/USD")
    pub asset: String,
    /// Aggregated price with 8 decimal places of precision
    pub price: u64,
    /// Confidence interval (spread) in the same fixed-point format
    pub confidence: u64,
    /// Slot number of the last successful price update
    pub last_update_slot: u64,
    /// Unix timestamp of the last update
    pub last_update_timestamp: i64,
    /// Authority that created this feed and can manage publishers
    pub authority: Pubkey,
    /// List of authorized publisher public keys
    pub publishers: Vec<Pubkey>,
    /// Number of submissions used in the latest aggregation
    pub num_submissions: u32,
    /// PDA bump seed
    pub bump: u8,
    /// Whether this feed has been initialized
    pub is_initialized: bool,
}

impl PriceFeed {
    /// Conservative upper bound for serialized size
    pub const MAX_LEN: usize = 4 + 32   // asset string (length prefix + max 32 chars)
        + 8                              // price
        + 8                              // confidence
        + 8                              // last_update_slot
        + 8                              // last_update_timestamp
        + 32                             // authority
        + 4 + (32 * MAX_PUBLISHERS)      // publishers vec
        + 4                              // num_submissions
        + 1                              // bump
        + 1;                             // is_initialized
        // = 598 bytes
}

/// Individual price submission from a single publisher.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct PriceSubmission {
    /// Publisher who submitted this price
    pub publisher: Pubkey,
    /// The price feed this submission is for
    pub feed: Pubkey,
    /// Submitted price (8 decimals)
    pub price: u64,
    /// Confidence / uncertainty (8 decimals)
    pub confidence: u64,
    /// Slot when submitted
    pub slot: u64,
    /// Unix timestamp of submission
    pub timestamp: i64,
    /// PDA bump
    pub bump: u8,
    /// Whether this submission is active
    pub is_initialized: bool,
}

impl PriceSubmission {
    pub const LEN: usize = 32 + 32 + 8 + 8 + 8 + 8 + 1 + 1; // 98 bytes
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum OracleInstruction {
    /// Create a new price feed for an asset pair.
    ///
    /// Accounts:
    ///   0. `[signer]`   Authority / payer
    ///   1. `[writable]` Price feed PDA
    ///   2. `[]`         System program
    CreateFeed {
        asset: String,
        publishers: Vec<Pubkey>,
    },

    /// Submit a price update from an authorized publisher.
    ///
    /// Accounts:
    ///   0. `[signer]`   Publisher
    ///   1. `[writable]` Price feed PDA
    ///   2. `[writable]` Submission PDA (publisher + feed)
    ///   3. `[]`         System program
    UpdatePrice {
        price: u64,
        confidence: u64,
    },

    /// Aggregate submitted prices from multiple publishers using the median.
    /// Anyone can call this to trigger aggregation.
    ///
    /// Accounts:
    ///   0. `[signer]`   Caller / payer
    ///   1. `[writable]` Price feed PDA
    ///   2..N `[]`       Submission PDAs to aggregate (one per publisher)
    AggregatePrices,

    /// Add a publisher to an existing feed (authority only).
    ///
    /// Accounts:
    ///   0. `[signer]`   Authority
    ///   1. `[writable]` Price feed PDA
    AddPublisher {
        publisher: Pubkey,
    },

    /// Remove a publisher from a feed (authority only).
    ///
    /// Accounts:
    ///   0. `[signer]`   Authority
    ///   1. `[writable]` Price feed PDA
    RemovePublisher {
        publisher: Pubkey,
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
    let instruction = OracleInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        OracleInstruction::CreateFeed { asset, publishers } => {
            process_create_feed(program_id, accounts, asset, publishers)
        }
        OracleInstruction::UpdatePrice { price, confidence } => {
            process_update_price(program_id, accounts, price, confidence)
        }
        OracleInstruction::AggregatePrices => {
            process_aggregate_prices(program_id, accounts)
        }
        OracleInstruction::AddPublisher { publisher } => {
            process_add_publisher(program_id, accounts, publisher)
        }
        OracleInstruction::RemovePublisher { publisher } => {
            process_remove_publisher(program_id, accounts, publisher)
        }
    }
}

// ---------------------------------------------------------------------------
// Instruction processors
// ---------------------------------------------------------------------------

fn process_create_feed(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    asset: String,
    publishers: Vec<Pubkey>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let feed_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if asset.len() > 32 {
        msg!("Error: asset name too long (max 32 chars)");
        return Err(ProgramError::InvalidArgument);
    }

    if publishers.len() > MAX_PUBLISHERS {
        msg!("Error: too many publishers (max {})", MAX_PUBLISHERS);
        return Err(ProgramError::InvalidArgument);
    }

    // Derive PDA from the asset name
    let (feed_pda, bump) =
        Pubkey::find_program_address(&[FEED_SEED, asset.as_bytes()], program_id);
    if *feed_info.key != feed_pda {
        msg!("Error: invalid feed PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Allocate the feed account
    let rent = Rent::get()?;
    let space = PriceFeed::MAX_LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            feed_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            feed_info.clone(),
            system_program.clone(),
        ],
        &[&[FEED_SEED, asset.as_bytes(), &[bump]]],
    )?;

    let feed = PriceFeed {
        asset: asset.clone(),
        price: 0,
        confidence: 0,
        last_update_slot: 0,
        last_update_timestamp: 0,
        authority: *authority.key,
        publishers,
        num_submissions: 0,
        bump,
        is_initialized: true,
    };

    feed.serialize(&mut &mut feed_info.data.borrow_mut()[..])?;

    msg!("Created price feed for {}", asset);
    Ok(())
}

fn process_update_price(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64,
    confidence: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let publisher = next_account_info(account_iter)?;
    let feed_info = next_account_info(account_iter)?;
    let submission_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !publisher.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if price == 0 {
        msg!("Error: price cannot be zero");
        return Err(ProgramError::InvalidArgument);
    }

    // Verify feed exists and publisher is authorized
    let feed = PriceFeed::try_from_slice(&feed_info.data.borrow())?;
    if !feed.is_initialized {
        msg!("Error: feed not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    if !feed.publishers.contains(publisher.key) {
        msg!("Error: publisher {} is not authorized for this feed", publisher.key);
        return Err(ProgramError::InvalidArgument);
    }

    // Derive submission PDA
    let (submission_pda, sub_bump) = Pubkey::find_program_address(
        &[SUBMISSION_SEED, feed_info.key.as_ref(), publisher.key.as_ref()],
        program_id,
    );
    if *submission_info.key != submission_pda {
        msg!("Error: invalid submission PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;

    // Create submission account if it doesn't exist, otherwise just update
    if submission_info.data_is_empty() {
        let rent = Rent::get()?;
        let space = PriceSubmission::LEN;
        let lamports = rent.minimum_balance(space);

        invoke_signed(
            &system_instruction::create_account(
                publisher.key,
                submission_info.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                publisher.clone(),
                submission_info.clone(),
                system_program.clone(),
            ],
            &[&[
                SUBMISSION_SEED,
                feed_info.key.as_ref(),
                publisher.key.as_ref(),
                &[sub_bump],
            ]],
        )?;
    }

    let submission = PriceSubmission {
        publisher: *publisher.key,
        feed: *feed_info.key,
        price,
        confidence,
        slot: clock.slot,
        timestamp: clock.unix_timestamp,
        bump: sub_bump,
        is_initialized: true,
    };

    submission.serialize(&mut &mut submission_info.data.borrow_mut()[..])?;

    msg!(
        "Publisher {} submitted price {} (confidence {}) for {}",
        publisher.key,
        price,
        confidence,
        feed.asset
    );
    Ok(())
}

fn process_aggregate_prices(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let caller = next_account_info(account_iter)?;
    let feed_info = next_account_info(account_iter)?;

    if !caller.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_info.data.borrow())?;
    if !feed.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    let clock = Clock::get()?;
    let current_slot = clock.slot;

    // Collect valid submissions from remaining accounts
    let mut prices: Vec<u64> = Vec::new();
    let mut confidences: Vec<u64> = Vec::new();

    for submission_info in account_iter {
        if submission_info.data_is_empty() {
            continue;
        }

        let submission = match PriceSubmission::try_from_slice(&submission_info.data.borrow()) {
            Ok(s) => s,
            Err(_) => continue,
        };

        if !submission.is_initialized {
            continue;
        }

        // Verify the submission belongs to this feed
        if submission.feed != *feed_info.key {
            continue;
        }

        // Verify the publisher is authorized
        if !feed.publishers.contains(&submission.publisher) {
            continue;
        }

        // Check staleness — reject submissions older than MAX_STALENESS_SLOTS
        if current_slot.saturating_sub(submission.slot) > MAX_STALENESS_SLOTS {
            msg!(
                "Skipping stale submission from {} (slot {} vs current {})",
                submission.publisher,
                submission.slot,
                current_slot
            );
            continue;
        }

        prices.push(submission.price);
        confidences.push(submission.confidence);
    }

    if prices.is_empty() {
        msg!("Error: no valid submissions to aggregate");
        return Err(ProgramError::InvalidArgument);
    }

    // Compute median price
    let median_price = compute_median(&mut prices);
    let median_confidence = compute_median(&mut confidences);

    feed.price = median_price;
    feed.confidence = median_confidence;
    feed.last_update_slot = current_slot;
    feed.last_update_timestamp = clock.unix_timestamp;
    feed.num_submissions = prices.len() as u32;

    feed.serialize(&mut &mut feed_info.data.borrow_mut()[..])?;

    msg!(
        "Aggregated {} submissions for {}: price={}, confidence={}",
        prices.len(),
        feed.asset,
        median_price,
        median_confidence
    );
    Ok(())
}

fn process_add_publisher(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    publisher: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let feed_info = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_info.data.borrow())?;
    if !feed.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if feed.authority != *authority.key {
        msg!("Error: only the feed authority can add publishers");
        return Err(ProgramError::InvalidArgument);
    }

    if feed.publishers.len() >= MAX_PUBLISHERS {
        msg!("Error: maximum publishers reached");
        return Err(ProgramError::InvalidArgument);
    }

    if feed.publishers.contains(&publisher) {
        msg!("Error: publisher already authorized");
        return Err(ProgramError::InvalidArgument);
    }

    feed.publishers.push(publisher);
    feed.serialize(&mut &mut feed_info.data.borrow_mut()[..])?;

    msg!("Added publisher {} to {}", publisher, feed.asset);
    Ok(())
}

fn process_remove_publisher(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    publisher: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let feed_info = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut feed = PriceFeed::try_from_slice(&feed_info.data.borrow())?;
    if !feed.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if feed.authority != *authority.key {
        msg!("Error: only the feed authority can remove publishers");
        return Err(ProgramError::InvalidArgument);
    }

    let initial_len = feed.publishers.len();
    feed.publishers.retain(|p| *p != publisher);

    if feed.publishers.len() == initial_len {
        msg!("Error: publisher not found");
        return Err(ProgramError::InvalidArgument);
    }

    feed.serialize(&mut &mut feed_info.data.borrow_mut()[..])?;

    msg!("Removed publisher {} from {}", publisher, feed.asset);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the median of a mutable slice of u64 values.
/// The slice is sorted in place. For an even number of elements the lower-median
/// is returned (to avoid rounding issues with integer arithmetic).
fn compute_median(values: &mut [u64]) -> u64 {
    assert!(!values.is_empty(), "cannot compute median of empty slice");
    values.sort_unstable();
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        // Average of the two middle values
        (values[mid - 1] / 2) + (values[mid] / 2) + ((values[mid - 1] % 2 + values[mid] % 2) / 2)
    } else {
        values[mid]
    }
}

/// Check whether a price feed is stale relative to the current slot.
pub fn is_price_stale(feed: &PriceFeed, current_slot: u64) -> bool {
    current_slot.saturating_sub(feed.last_update_slot) > MAX_STALENESS_SLOTS
}

/// Format a fixed-point price for display (informational only).
pub fn format_price(price: u64) -> String {
    let whole = price / PRICE_DECIMALS;
    let frac = price % PRICE_DECIMALS;
    format!("{}.{:08}", whole, frac)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_median_odd() {
        let mut prices = vec![300, 100, 200];
        assert_eq!(compute_median(&mut prices), 200);
    }

    #[test]
    fn test_median_even() {
        let mut prices = vec![100, 200, 300, 400];
        assert_eq!(compute_median(&mut prices), 250);
    }

    #[test]
    fn test_median_single() {
        let mut prices = vec![42];
        assert_eq!(compute_median(&mut prices), 42);
    }

    #[test]
    fn test_median_duplicates() {
        let mut prices = vec![100, 100, 200, 200, 300];
        assert_eq!(compute_median(&mut prices), 200);
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(100_000_000), "1.00000000");
        assert_eq!(format_price(4_512_399_000_000), "45123.99000000");
        assert_eq!(format_price(50_000), "0.00050000");
    }

    #[test]
    fn test_is_price_stale() {
        let feed = PriceFeed {
            asset: "TEST/USD".to_string(),
            price: 100_000_000,
            confidence: 1_000_000,
            last_update_slot: 1000,
            last_update_timestamp: 0,
            authority: Pubkey::default(),
            publishers: vec![],
            num_submissions: 1,
            bump: 0,
            is_initialized: true,
        };

        // Not stale: within 120 slots
        assert!(!is_price_stale(&feed, 1100));
        // Stale: more than 120 slots
        assert!(is_price_stale(&feed, 1200));
    }
}
