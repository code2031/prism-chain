use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
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

solana_program::declare_id!("SoLMaRtNFTMktP1aceXXXXXXXXXXXXXXXXXXXXXXXX");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Copy, Clone)]
pub enum MarketplaceError {
    #[error("Listing is not active")]
    ListingNotActive,
    #[error("Insufficient payment amount")]
    InsufficientPayment,
    #[error("Auction has not ended yet")]
    AuctionNotEnded,
    #[error("Auction has already ended")]
    AuctionAlreadyEnded,
    #[error("Bid is too low")]
    BidTooLow,
    #[error("Reserve price not met")]
    ReservePriceNotMet,
    #[error("Offer has expired")]
    OfferExpired,
    #[error("Offer is still active")]
    OfferStillActive,
    #[error("Unauthorized signer")]
    Unauthorized,
    #[error("Invalid collection")]
    InvalidCollection,
    #[error("Royalty basis points exceed maximum (10000)")]
    RoyaltyTooHigh,
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,
    #[error("Invalid account owner")]
    InvalidAccountOwner,
    #[error("Account already initialized")]
    AlreadyInitialized,
}

impl From<MarketplaceError> for ProgramError {
    fn from(e: MarketplaceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State — Listing
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Listing {
    /// Discriminator tag
    pub tag: u8,
    /// Mint address of the NFT
    pub nft_mint: Pubkey,
    /// Seller wallet
    pub seller: Pubkey,
    /// Price in lamports
    pub price: u64,
    /// Whether the listing is currently active
    pub is_active: bool,
    /// Unix timestamp of listing creation
    pub created_at: i64,
    /// Optional collection address (Pubkey::default() if none)
    pub collection: Pubkey,
}

impl Listing {
    pub const TAG: u8 = 1;
    pub const LEN: usize = 1 + 32 + 32 + 8 + 1 + 8 + 32; // 114 bytes
}

// ---------------------------------------------------------------------------
// State — Auction
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Auction {
    pub tag: u8,
    /// Mint address of the NFT
    pub nft_mint: Pubkey,
    /// Seller wallet
    pub seller: Pubkey,
    /// Starting price in lamports
    pub starting_price: u64,
    /// Current highest bid in lamports
    pub current_bid: u64,
    /// Current highest bidder (Pubkey::default() if no bids)
    pub current_bidder: Pubkey,
    /// Auction end time (unix timestamp)
    pub end_time: i64,
    /// Minimum price the seller will accept
    pub reserve_price: u64,
}

impl Auction {
    pub const TAG: u8 = 2;
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8 + 32 + 8 + 8; // 129 bytes
}

// ---------------------------------------------------------------------------
// State — Offer
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Offer {
    pub tag: u8,
    /// Mint address of the NFT the offer targets
    pub nft_mint: Pubkey,
    /// Buyer making the offer
    pub buyer: Pubkey,
    /// Offered price in lamports (escrowed)
    pub price: u64,
    /// Offer expiry (unix timestamp)
    pub expiry: i64,
}

impl Offer {
    pub const TAG: u8 = 3;
    pub const LEN: usize = 1 + 32 + 32 + 8 + 8; // 81 bytes
}

// ---------------------------------------------------------------------------
// State — Collection
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Collection {
    pub tag: u8,
    /// Collection name (max 32 bytes, padded)
    pub name: [u8; 32],
    /// Symbol (max 10 bytes, padded)
    pub symbol: [u8; 10],
    /// Creator / authority wallet
    pub creator: Pubkey,
    /// Royalty in basis points (e.g. 500 = 5%)
    pub royalty_bps: u16,
    /// Whether the collection is verified by the marketplace
    pub verified: bool,
}

impl Collection {
    pub const TAG: u8 = 4;
    pub const LEN: usize = 1 + 32 + 10 + 32 + 2 + 1; // 78 bytes
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum MarketplaceInstruction {
    /// 0 — Register a new NFT collection.
    /// Accounts: [signer] creator, [writable] collection_pda, [readable] system_program
    /// Data: name (32 bytes), symbol (10 bytes), royalty_bps (u16)
    CreateCollection {
        name: [u8; 32],
        symbol: [u8; 10],
        royalty_bps: u16,
    },

    /// 1 — List an NFT for fixed-price sale.
    /// Accounts: [signer] seller, [writable] listing_pda, [readable] nft_mint,
    ///           [writable] seller_token_account, [writable] escrow_token_account,
    ///           [readable] token_program, [readable] system_program
    /// Data: price (u64), collection (Pubkey)
    ListNft {
        price: u64,
        collection: Pubkey,
    },

    /// 2 — Remove an active listing.
    /// Accounts: [signer] seller, [writable] listing_pda, [writable] escrow_token_account,
    ///           [writable] seller_token_account, [readable] token_program
    DelistNft,

    /// 3 — Purchase a listed NFT.
    /// Accounts: [signer] buyer, [writable] listing_pda, [writable] seller,
    ///           [writable] escrow_token_account, [writable] buyer_token_account,
    ///           [writable] collection_pda (for royalty lookup),
    ///           [writable] royalty_recipient, [readable] token_program,
    ///           [readable] system_program
    BuyNft,

    /// 4 — Create a timed auction.
    /// Accounts: [signer] seller, [writable] auction_pda, [readable] nft_mint,
    ///           [writable] seller_token_account, [writable] escrow_token_account,
    ///           [readable] token_program, [readable] system_program
    /// Data: starting_price (u64), duration (i64 seconds), reserve_price (u64)
    CreateAuction {
        starting_price: u64,
        duration: i64,
        reserve_price: u64,
    },

    /// 5 — Place a bid on an active auction.
    /// Accounts: [signer] bidder, [writable] auction_pda, [writable] previous_bidder,
    ///           [readable] system_program
    /// Data: bid_amount (u64)
    PlaceBid {
        bid_amount: u64,
    },

    /// 6 — Settle a completed auction.
    /// Accounts: [signer] anyone, [writable] auction_pda, [writable] seller,
    ///           [writable] winner, [writable] escrow_token_account,
    ///           [writable] winner_token_account, [writable] collection_pda,
    ///           [writable] royalty_recipient, [readable] token_program,
    ///           [readable] system_program
    SettleAuction,

    /// 7 — Make an offer on any NFT (SOL escrowed in offer PDA).
    /// Accounts: [signer] buyer, [writable] offer_pda, [readable] nft_mint,
    ///           [readable] system_program
    /// Data: price (u64), expiry (i64)
    MakeOffer {
        price: u64,
        expiry: i64,
    },

    /// 8 — NFT owner accepts an offer.
    /// Accounts: [signer] seller, [writable] offer_pda, [writable] buyer,
    ///           [writable] seller_token_account, [writable] buyer_token_account,
    ///           [writable] collection_pda, [writable] royalty_recipient,
    ///           [readable] token_program, [readable] system_program
    AcceptOffer,

    /// 9 — Buyer cancels their offer and reclaims SOL.
    /// Accounts: [signer] buyer, [writable] offer_pda, [readable] system_program
    CancelOffer,
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
    let instruction = MarketplaceInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        MarketplaceInstruction::CreateCollection {
            name,
            symbol,
            royalty_bps,
        } => process_create_collection(program_id, accounts, name, symbol, royalty_bps),
        MarketplaceInstruction::ListNft { price, collection } => {
            process_list_nft(program_id, accounts, price, collection)
        }
        MarketplaceInstruction::DelistNft => process_delist_nft(program_id, accounts),
        MarketplaceInstruction::BuyNft => process_buy_nft(program_id, accounts),
        MarketplaceInstruction::CreateAuction {
            starting_price,
            duration,
            reserve_price,
        } => process_create_auction(program_id, accounts, starting_price, duration, reserve_price),
        MarketplaceInstruction::PlaceBid { bid_amount } => {
            process_place_bid(program_id, accounts, bid_amount)
        }
        MarketplaceInstruction::SettleAuction => process_settle_auction(program_id, accounts),
        MarketplaceInstruction::MakeOffer { price, expiry } => {
            process_make_offer(program_id, accounts, price, expiry)
        }
        MarketplaceInstruction::AcceptOffer => process_accept_offer(program_id, accounts),
        MarketplaceInstruction::CancelOffer => process_cancel_offer(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Calculate royalty amount from a sale price given basis points.
/// Returns (royalty_amount, seller_proceeds).
pub fn calculate_royalty(sale_price: u64, royalty_bps: u16) -> Result<(u64, u64), MarketplaceError> {
    let royalty = (sale_price as u128)
        .checked_mul(royalty_bps as u128)
        .ok_or(MarketplaceError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(MarketplaceError::ArithmeticOverflow)? as u64;
    let proceeds = sale_price
        .checked_sub(royalty)
        .ok_or(MarketplaceError::ArithmeticOverflow)?;
    Ok((royalty, proceeds))
}

/// Validate that a collection account is properly initialized and belongs to
/// this program.
pub fn validate_collection(
    collection_info: &AccountInfo,
    program_id: &Pubkey,
) -> Result<Collection, MarketplaceError> {
    if collection_info.owner != program_id {
        return Err(MarketplaceError::InvalidAccountOwner);
    }
    let data = collection_info.try_borrow_data().map_err(|_| MarketplaceError::InvalidCollection)?;
    let collection =
        Collection::try_from_slice(&data).map_err(|_| MarketplaceError::InvalidCollection)?;
    if collection.tag != Collection::TAG {
        return Err(MarketplaceError::InvalidCollection);
    }
    Ok(collection)
}

/// Derive a PDA and verify it matches the expected account.
fn assert_pda(
    program_id: &Pubkey,
    seeds: &[&[u8]],
    expected: &Pubkey,
) -> Result<u8, ProgramError> {
    let (pda, bump) = Pubkey::find_program_address(seeds, program_id);
    if pda != *expected {
        msg!("PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }
    Ok(bump)
}

/// Transfer lamports from a signer to a destination via system_program.
fn transfer_lamports<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    invoke(
        &system_instruction::transfer(from.key, to.key, amount),
        &[from.clone(), to.clone(), system_program.clone()],
    )
}

/// Transfer lamports from a PDA (signed) to a destination.
fn transfer_lamports_signed<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    amount: u64,
) -> ProgramResult {
    **from.try_borrow_mut_lamports()? -= amount;
    **to.try_borrow_mut_lamports()? += amount;
    Ok(())
}

// ---------------------------------------------------------------------------
// Instruction Processors
// ---------------------------------------------------------------------------

fn process_create_collection(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: [u8; 32],
    symbol: [u8; 10],
    royalty_bps: u16,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let creator = next_account_info(iter)?;
    let collection_pda = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !creator.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }
    if royalty_bps > 10_000 {
        return Err(MarketplaceError::RoyaltyTooHigh.into());
    }

    // Derive PDA: seeds = ["collection", creator, name]
    let bump = assert_pda(
        program_id,
        &[b"collection", creator.key.as_ref(), &name],
        collection_pda.key,
    )?;

    let rent = Rent::get()?;
    let space = Collection::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            creator.key,
            collection_pda.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[creator.clone(), collection_pda.clone(), system_program.clone()],
        &[&[b"collection", creator.key.as_ref(), &name, &[bump]]],
    )?;

    let collection = Collection {
        tag: Collection::TAG,
        name,
        symbol,
        creator: *creator.key,
        royalty_bps,
        verified: false,
    };
    collection.serialize(&mut *collection_pda.try_borrow_mut_data()?)?;

    msg!("SolMart: Collection created");
    Ok(())
}

fn process_list_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64,
    collection: Pubkey,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let seller = next_account_info(iter)?;
    let listing_pda = next_account_info(iter)?;
    let nft_mint = next_account_info(iter)?;
    let seller_token_account = next_account_info(iter)?;
    let escrow_token_account = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !seller.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    // Derive listing PDA: seeds = ["listing", nft_mint, seller]
    let bump = assert_pda(
        program_id,
        &[b"listing", nft_mint.key.as_ref(), seller.key.as_ref()],
        listing_pda.key,
    )?;

    let rent = Rent::get()?;
    let space = Listing::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            seller.key,
            listing_pda.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[seller.clone(), listing_pda.clone(), system_program.clone()],
        &[&[b"listing", nft_mint.key.as_ref(), seller.key.as_ref(), &[bump]]],
    )?;

    // Transfer NFT from seller to escrow
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            seller_token_account.key,
            escrow_token_account.key,
            seller.key,
            &[],
            1,
        )?,
        &[
            seller_token_account.clone(),
            escrow_token_account.clone(),
            seller.clone(),
            token_program.clone(),
        ],
    )?;

    let clock = Clock::get()?;
    let listing = Listing {
        tag: Listing::TAG,
        nft_mint: *nft_mint.key,
        seller: *seller.key,
        price,
        is_active: true,
        created_at: clock.unix_timestamp,
        collection,
    };
    listing.serialize(&mut *listing_pda.try_borrow_mut_data()?)?;

    msg!("SolMart: NFT listed for {} lamports", price);
    Ok(())
}

fn process_delist_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let seller = next_account_info(iter)?;
    let listing_pda = next_account_info(iter)?;
    let escrow_token_account = next_account_info(iter)?;
    let seller_token_account = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;

    if !seller.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let mut listing = Listing::try_from_slice(&listing_pda.try_borrow_data()?)?;
    if listing.seller != *seller.key {
        return Err(MarketplaceError::Unauthorized.into());
    }
    if !listing.is_active {
        return Err(MarketplaceError::ListingNotActive.into());
    }

    // Derive PDA bump for signing the escrow transfer back
    let (_, bump) = Pubkey::find_program_address(
        &[b"listing", listing.nft_mint.as_ref(), seller.key.as_ref()],
        program_id,
    );

    // Transfer NFT back from escrow to seller
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            escrow_token_account.key,
            seller_token_account.key,
            listing_pda.key,
            &[],
            1,
        )?,
        &[
            escrow_token_account.clone(),
            seller_token_account.clone(),
            listing_pda.clone(),
            token_program.clone(),
        ],
        &[&[
            b"listing",
            listing.nft_mint.as_ref(),
            seller.key.as_ref(),
            &[bump],
        ]],
    )?;

    listing.is_active = false;
    listing.serialize(&mut *listing_pda.try_borrow_mut_data()?)?;

    msg!("SolMart: NFT delisted");
    Ok(())
}

fn process_buy_nft(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let buyer = next_account_info(iter)?;
    let listing_pda = next_account_info(iter)?;
    let seller = next_account_info(iter)?;
    let escrow_token_account = next_account_info(iter)?;
    let buyer_token_account = next_account_info(iter)?;
    let collection_pda = next_account_info(iter)?;
    let royalty_recipient = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !buyer.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let mut listing = Listing::try_from_slice(&listing_pda.try_borrow_data()?)?;
    if !listing.is_active {
        return Err(MarketplaceError::ListingNotActive.into());
    }

    // Determine royalty split
    let (royalty_amount, seller_proceeds) =
        if listing.collection != Pubkey::default() && collection_pda.owner == program_id {
            let collection = validate_collection(collection_pda, program_id)?;
            calculate_royalty(listing.price, collection.royalty_bps)?
        } else {
            (0u64, listing.price)
        };

    // Transfer SOL: buyer -> seller (proceeds)
    transfer_lamports(buyer, seller, system_program, seller_proceeds)?;

    // Transfer SOL: buyer -> royalty_recipient (royalty)
    if royalty_amount > 0 {
        transfer_lamports(buyer, royalty_recipient, system_program, royalty_amount)?;
    }

    // Transfer NFT: escrow -> buyer
    let (_, bump) = Pubkey::find_program_address(
        &[
            b"listing",
            listing.nft_mint.as_ref(),
            listing.seller.as_ref(),
        ],
        program_id,
    );

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            escrow_token_account.key,
            buyer_token_account.key,
            listing_pda.key,
            &[],
            1,
        )?,
        &[
            escrow_token_account.clone(),
            buyer_token_account.clone(),
            listing_pda.clone(),
            token_program.clone(),
        ],
        &[&[
            b"listing",
            listing.nft_mint.as_ref(),
            listing.seller.as_ref(),
            &[bump],
        ]],
    )?;

    listing.is_active = false;
    listing.serialize(&mut *listing_pda.try_borrow_mut_data()?)?;

    msg!(
        "SolMart: NFT sold — seller receives {}, royalty {}",
        seller_proceeds,
        royalty_amount
    );
    Ok(())
}

fn process_create_auction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    starting_price: u64,
    duration: i64,
    reserve_price: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let seller = next_account_info(iter)?;
    let auction_pda = next_account_info(iter)?;
    let nft_mint = next_account_info(iter)?;
    let seller_token_account = next_account_info(iter)?;
    let escrow_token_account = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !seller.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let bump = assert_pda(
        program_id,
        &[b"auction", nft_mint.key.as_ref(), seller.key.as_ref()],
        auction_pda.key,
    )?;

    let rent = Rent::get()?;
    let space = Auction::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            seller.key,
            auction_pda.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[seller.clone(), auction_pda.clone(), system_program.clone()],
        &[&[b"auction", nft_mint.key.as_ref(), seller.key.as_ref(), &[bump]]],
    )?;

    // Transfer NFT into escrow
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            seller_token_account.key,
            escrow_token_account.key,
            seller.key,
            &[],
            1,
        )?,
        &[
            seller_token_account.clone(),
            escrow_token_account.clone(),
            seller.clone(),
            token_program.clone(),
        ],
    )?;

    let clock = Clock::get()?;
    let auction = Auction {
        tag: Auction::TAG,
        nft_mint: *nft_mint.key,
        seller: *seller.key,
        starting_price,
        current_bid: 0,
        current_bidder: Pubkey::default(),
        end_time: clock.unix_timestamp + duration,
        reserve_price,
    };
    auction.serialize(&mut *auction_pda.try_borrow_mut_data()?)?;

    msg!(
        "SolMart: Auction created — starts at {} lamports, ends at {}",
        starting_price,
        auction.end_time
    );
    Ok(())
}

fn process_place_bid(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    bid_amount: u64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let bidder = next_account_info(iter)?;
    let auction_pda = next_account_info(iter)?;
    let previous_bidder = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !bidder.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let mut auction = Auction::try_from_slice(&auction_pda.try_borrow_data()?)?;

    let clock = Clock::get()?;
    if clock.unix_timestamp >= auction.end_time {
        return Err(MarketplaceError::AuctionAlreadyEnded.into());
    }

    // Bid must exceed both the starting price and the current bid
    if bid_amount <= auction.current_bid || bid_amount < auction.starting_price {
        return Err(MarketplaceError::BidTooLow.into());
    }

    // Refund previous bidder if one exists
    if auction.current_bid > 0 && auction.current_bidder != Pubkey::default() {
        transfer_lamports_signed(auction_pda, previous_bidder, auction.current_bid)?;
    }

    // Escrow new bid into the auction PDA
    transfer_lamports(bidder, auction_pda, system_program, bid_amount)?;

    auction.current_bid = bid_amount;
    auction.current_bidder = *bidder.key;
    auction.serialize(&mut *auction_pda.try_borrow_mut_data()?)?;

    msg!("SolMart: New bid of {} lamports", bid_amount);
    Ok(())
}

fn process_settle_auction(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let _caller = next_account_info(iter)?;
    let auction_pda = next_account_info(iter)?;
    let seller = next_account_info(iter)?;
    let winner = next_account_info(iter)?;
    let escrow_token_account = next_account_info(iter)?;
    let winner_token_account = next_account_info(iter)?;
    let collection_pda = next_account_info(iter)?;
    let royalty_recipient = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let _system_program = next_account_info(iter)?;

    let auction = Auction::try_from_slice(&auction_pda.try_borrow_data()?)?;

    let clock = Clock::get()?;
    if clock.unix_timestamp < auction.end_time {
        return Err(MarketplaceError::AuctionNotEnded.into());
    }

    if auction.current_bid < auction.reserve_price {
        return Err(MarketplaceError::ReservePriceNotMet.into());
    }

    // Calculate royalty
    let (royalty_amount, seller_proceeds) =
        if collection_pda.owner == program_id && collection_pda.data_len() >= Collection::LEN {
            let collection = validate_collection(collection_pda, program_id)?;
            calculate_royalty(auction.current_bid, collection.royalty_bps)?
        } else {
            (0u64, auction.current_bid)
        };

    // Transfer SOL from auction PDA to seller
    transfer_lamports_signed(auction_pda, seller, seller_proceeds)?;

    // Transfer royalty
    if royalty_amount > 0 {
        transfer_lamports_signed(auction_pda, royalty_recipient, royalty_amount)?;
    }

    // Transfer NFT from escrow to winner
    let (_, bump) = Pubkey::find_program_address(
        &[
            b"auction",
            auction.nft_mint.as_ref(),
            auction.seller.as_ref(),
        ],
        program_id,
    );

    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            escrow_token_account.key,
            winner_token_account.key,
            auction_pda.key,
            &[],
            1,
        )?,
        &[
            escrow_token_account.clone(),
            winner_token_account.clone(),
            auction_pda.clone(),
            token_program.clone(),
        ],
        &[&[
            b"auction",
            auction.nft_mint.as_ref(),
            auction.seller.as_ref(),
            &[bump],
        ]],
    )?;

    msg!(
        "SolMart: Auction settled — winner receives NFT, seller receives {} lamports",
        seller_proceeds
    );
    Ok(())
}

fn process_make_offer(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    price: u64,
    expiry: i64,
) -> ProgramResult {
    let iter = &mut accounts.iter();
    let buyer = next_account_info(iter)?;
    let offer_pda = next_account_info(iter)?;
    let nft_mint = next_account_info(iter)?;
    let system_program = next_account_info(iter)?;

    if !buyer.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let bump = assert_pda(
        program_id,
        &[b"offer", nft_mint.key.as_ref(), buyer.key.as_ref()],
        offer_pda.key,
    )?;

    let rent = Rent::get()?;
    let space = Offer::LEN;
    let lamports = rent.minimum_balance(space);

    // Create the offer PDA (rent-exempt) + escrow the offered SOL
    let total_lamports = lamports + price;

    invoke_signed(
        &system_instruction::create_account(
            buyer.key,
            offer_pda.key,
            total_lamports,
            space as u64,
            program_id,
        ),
        &[buyer.clone(), offer_pda.clone(), system_program.clone()],
        &[&[b"offer", nft_mint.key.as_ref(), buyer.key.as_ref(), &[bump]]],
    )?;

    let offer = Offer {
        tag: Offer::TAG,
        nft_mint: *nft_mint.key,
        buyer: *buyer.key,
        price,
        expiry,
    };
    offer.serialize(&mut *offer_pda.try_borrow_mut_data()?)?;

    msg!("SolMart: Offer of {} lamports placed", price);
    Ok(())
}

fn process_accept_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let seller = next_account_info(iter)?;
    let offer_pda = next_account_info(iter)?;
    let buyer = next_account_info(iter)?;
    let seller_token_account = next_account_info(iter)?;
    let buyer_token_account = next_account_info(iter)?;
    let collection_pda = next_account_info(iter)?;
    let royalty_recipient = next_account_info(iter)?;
    let token_program = next_account_info(iter)?;
    let _system_program = next_account_info(iter)?;

    if !seller.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let offer = Offer::try_from_slice(&offer_pda.try_borrow_data()?)?;

    let clock = Clock::get()?;
    if clock.unix_timestamp > offer.expiry {
        return Err(MarketplaceError::OfferExpired.into());
    }

    // Calculate royalty
    let (royalty_amount, seller_proceeds) =
        if collection_pda.owner == program_id && collection_pda.data_len() >= Collection::LEN {
            let collection = validate_collection(collection_pda, program_id)?;
            calculate_royalty(offer.price, collection.royalty_bps)?
        } else {
            (0u64, offer.price)
        };

    // Transfer escrowed SOL from offer PDA -> seller
    transfer_lamports_signed(offer_pda, seller, seller_proceeds)?;

    // Royalty
    if royalty_amount > 0 {
        transfer_lamports_signed(offer_pda, royalty_recipient, royalty_amount)?;
    }

    // Transfer NFT from seller to buyer
    invoke(
        &spl_token::instruction::transfer(
            token_program.key,
            seller_token_account.key,
            buyer_token_account.key,
            seller.key,
            &[],
            1,
        )?,
        &[
            seller_token_account.clone(),
            buyer_token_account.clone(),
            seller.clone(),
            token_program.clone(),
        ],
    )?;

    // Close the offer PDA — return remaining rent to buyer
    let remaining = offer_pda.lamports();
    transfer_lamports_signed(offer_pda, buyer, remaining)?;

    msg!(
        "SolMart: Offer accepted — seller receives {}, royalty {}",
        seller_proceeds,
        royalty_amount
    );
    Ok(())
}

fn process_cancel_offer(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let iter = &mut accounts.iter();
    let buyer = next_account_info(iter)?;
    let offer_pda = next_account_info(iter)?;
    let _system_program = next_account_info(iter)?;

    if !buyer.is_signer {
        return Err(MarketplaceError::Unauthorized.into());
    }

    let offer = Offer::try_from_slice(&offer_pda.try_borrow_data()?)?;
    if offer.buyer != *buyer.key {
        return Err(MarketplaceError::Unauthorized.into());
    }

    // Return all lamports (rent + escrowed SOL) to buyer
    let lamports = offer_pda.lamports();
    transfer_lamports_signed(offer_pda, buyer, lamports)?;

    msg!("SolMart: Offer cancelled — {} lamports refunded", lamports);
    Ok(())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_royalty() {
        // 5% royalty on 1 SOL
        let (royalty, proceeds) = calculate_royalty(1_000_000_000, 500).unwrap();
        assert_eq!(royalty, 50_000_000);
        assert_eq!(proceeds, 950_000_000);
    }

    #[test]
    fn test_calculate_royalty_zero() {
        let (royalty, proceeds) = calculate_royalty(1_000_000_000, 0).unwrap();
        assert_eq!(royalty, 0);
        assert_eq!(proceeds, 1_000_000_000);
    }

    #[test]
    fn test_calculate_royalty_max() {
        // 100% — edge case
        let (royalty, proceeds) = calculate_royalty(1_000_000_000, 10_000).unwrap();
        assert_eq!(royalty, 1_000_000_000);
        assert_eq!(proceeds, 0);
    }

    #[test]
    fn test_listing_size() {
        assert_eq!(Listing::LEN, 114);
    }

    #[test]
    fn test_auction_size() {
        assert_eq!(Auction::LEN, 129);
    }

    #[test]
    fn test_offer_size() {
        assert_eq!(Offer::LEN, 81);
    }

    #[test]
    fn test_collection_size() {
        assert_eq!(Collection::LEN, 78);
    }
}
