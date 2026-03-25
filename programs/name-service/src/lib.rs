use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    hash::hash,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Seed for name record PDA.
pub const NAME_SEED: &[u8] = b"name";

/// Seed for reverse lookup PDA (address -> name).
pub const REVERSE_SEED: &[u8] = b"reverse";

/// Seed for subdomain PDA.
pub const SUBDOMAIN_SEED: &[u8] = b"subdomain";

/// Seed for name service registry/config PDA.
pub const REGISTRY_SEED: &[u8] = b"ns_registry";

/// The `.prism` top-level domain suffix.
pub const TLD: &str = ".prism";

/// Maximum name length (excluding TLD).
pub const MAX_NAME_LEN: usize = 64;

/// Minimum name length.
pub const MIN_NAME_LEN: usize = 1;

/// Maximum subdomain label length.
pub const MAX_SUBDOMAIN_LEN: usize = 32;

/// Maximum number of records per name.
pub const MAX_RECORDS: usize = 16;

/// Maximum single record value length in bytes.
pub const MAX_RECORD_VALUE_LEN: usize = 256;

/// Registration period: 1 year in seconds.
pub const REGISTRATION_PERIOD_SECS: i64 = 365 * 24 * 60 * 60;

/// Grace period after expiry before the name can be claimed by someone else.
pub const GRACE_PERIOD_SECS: i64 = 30 * 24 * 60 * 60; // 30 days

/// Base registration fee in lamports (for names >= 5 chars).
pub const BASE_FEE_LAMPORTS: u64 = 100_000_000; // 0.1 SOL

/// Premium multiplier for short names (3-4 chars).
pub const SHORT_NAME_MULTIPLIER: u64 = 10;

/// Premium multiplier for single/two-char names.
pub const ULTRA_SHORT_NAME_MULTIPLIER: u64 = 100;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum NameServiceError {
    #[error("Name already registered")]
    NameAlreadyRegistered,
    #[error("Name has expired")]
    NameExpired,
    #[error("Name has not expired (still within grace period)")]
    NameNotExpired,
    #[error("Not the owner of this name")]
    NotOwner,
    #[error("Name too long")]
    NameTooLong,
    #[error("Name too short")]
    NameTooShort,
    #[error("Invalid name characters (only a-z, 0-9, hyphen allowed)")]
    InvalidNameChars,
    #[error("Name cannot start or end with a hyphen")]
    InvalidHyphenPlacement,
    #[error("Too many records")]
    TooManyRecords,
    #[error("Record value too long")]
    RecordValueTooLong,
    #[error("Subdomain label too long")]
    SubdomainTooLong,
    #[error("Parent name does not exist or is expired")]
    InvalidParentName,
    #[error("Insufficient registration fee")]
    InsufficientFee,
    #[error("Registry already initialized")]
    AlreadyInitialized,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Record key not found")]
    RecordNotFound,
    #[error("Reverse record already set")]
    ReverseRecordExists,
    #[error("Name not found")]
    NameNotFound,
}

impl From<NameServiceError> for ProgramError {
    fn from(e: NameServiceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Well-known record keys.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RecordKey {
    /// Wallet address (Prism/Solana pubkey).
    Wallet,
    /// IPFS content hash.
    Ipfs,
    /// Arweave URL.
    Arweave,
    /// Twitter/X handle.
    Twitter,
    /// Discord handle.
    Discord,
    /// Telegram handle.
    Telegram,
    /// GitHub username.
    Github,
    /// Email address.
    Email,
    /// Website URL.
    Url,
    /// Avatar URL or IPFS hash.
    Avatar,
    /// Bitcoin address.
    Btc,
    /// Ethereum address.
    Eth,
    /// Arbitrary key (custom record).
    Custom { key: Vec<u8> },
}

/// A single key-value record attached to a name.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NameRecord {
    pub key: RecordKey,
    pub value: Vec<u8>,
}

/// The name service registry configuration.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NameRegistry {
    pub is_initialized: bool,
    /// Authority that can update fees and parameters.
    pub authority: Pubkey,
    /// Treasury address that receives registration fees.
    pub treasury: Pubkey,
    /// Total names registered.
    pub total_names: u64,
    /// PDA bump.
    pub bump: u8,
}

impl NameRegistry {
    pub const MAX_SIZE: usize = 1 + 32 + 32 + 8 + 1;
}

/// A registered name (e.g., "alice.prism").
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Name {
    pub is_initialized: bool,
    /// The name string (without .prism TLD).
    pub name: Vec<u8>,
    /// The name hash (used for PDA derivation).
    pub name_hash: [u8; 32],
    /// Current owner.
    pub owner: Pubkey,
    /// Expiry unix timestamp.
    pub expires_at: i64,
    /// When the name was first registered.
    pub registered_at: i64,
    /// Key-value records.
    pub records: Vec<NameRecord>,
    /// Whether this name has a reverse lookup set.
    pub has_reverse: bool,
    /// PDA bump.
    pub bump: u8,
}

impl Name {
    pub const MAX_SIZE: usize = 1
        + (4 + MAX_NAME_LEN)
        + 32
        + 32
        + 8
        + 8
        + (4 + MAX_RECORDS * (64 + 4 + MAX_RECORD_VALUE_LEN))
        + 1
        + 1;
}

/// A subdomain record (e.g., "sub.alice.prism").
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Subdomain {
    pub is_initialized: bool,
    /// Subdomain label (the part before the parent name).
    pub label: Vec<u8>,
    /// Parent name hash.
    pub parent_name_hash: [u8; 32],
    /// Subdomain owner (defaults to parent owner, can be different).
    pub owner: Pubkey,
    /// Records for this subdomain.
    pub records: Vec<NameRecord>,
    /// PDA bump.
    pub bump: u8,
}

impl Subdomain {
    pub const MAX_SIZE: usize = 1
        + (4 + MAX_SUBDOMAIN_LEN)
        + 32
        + 32
        + (4 + MAX_RECORDS * (64 + 4 + MAX_RECORD_VALUE_LEN))
        + 1;
}

/// Reverse lookup: address -> name hash.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ReverseLookup {
    pub is_initialized: bool,
    /// The address this reverse record is for.
    pub address: Pubkey,
    /// The name hash it resolves to.
    pub name_hash: [u8; 32],
    /// PDA bump.
    pub bump: u8,
}

impl ReverseLookup {
    pub const MAX_SIZE: usize = 1 + 32 + 32 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum NameServiceInstruction {
    /// Initialize the name service registry.
    ///
    /// Accounts:
    ///   0. `[writable]` Registry PDA
    ///   1. `[signer]`   Authority
    ///   2. `[]`         Treasury
    ///   3. `[]`         System program
    ///   4. `[]`         Rent sysvar
    InitializeRegistry,

    /// Register a new name.
    ///
    /// Accounts:
    ///   0. `[]`         Registry PDA
    ///   1. `[writable]` Name PDA
    ///   2. `[signer]`   Registrant (becomes owner)
    ///   3. `[writable]` Treasury (receives fee)
    ///   4. `[]`         Clock sysvar
    ///   5. `[]`         System program
    ///   6. `[]`         Rent sysvar
    RegisterName {
        name: Vec<u8>,
    },

    /// Renew an existing name for another year.
    ///
    /// Accounts:
    ///   0. `[writable]` Name PDA
    ///   1. `[signer]`   Owner (or anyone during grace period)
    ///   2. `[writable]` Treasury
    ///   3. `[]`         Clock sysvar
    RenewName,

    /// Transfer name ownership.
    ///
    /// Accounts:
    ///   0. `[writable]` Name PDA
    ///   1. `[signer]`   Current owner
    ///   2. `[]`         New owner
    TransferName {
        new_owner: Pubkey,
    },

    /// Set or update a record on a name.
    ///
    /// Accounts:
    ///   0. `[writable]` Name PDA
    ///   1. `[signer]`   Owner
    ///   2. `[]`         Clock sysvar
    SetRecord {
        key: RecordKey,
        value: Vec<u8>,
    },

    /// Remove a record from a name.
    ///
    /// Accounts:
    ///   0. `[writable]` Name PDA
    ///   1. `[signer]`   Owner
    DeleteRecord {
        key: RecordKey,
    },

    /// Create a subdomain under a registered name.
    ///
    /// Accounts:
    ///   0. `[]`         Parent name PDA
    ///   1. `[writable]` Subdomain PDA
    ///   2. `[signer]`   Parent name owner
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    CreateSubdomain {
        label: Vec<u8>,
        subdomain_owner: Pubkey,
    },

    /// Delete a subdomain.
    ///
    /// Accounts:
    ///   0. `[]`         Parent name PDA
    ///   1. `[writable]` Subdomain PDA
    ///   2. `[signer]`   Parent name owner
    DeleteSubdomain,

    /// Set the reverse lookup (address -> name).
    ///
    /// Accounts:
    ///   0. `[]`         Name PDA (must be owned by signer)
    ///   1. `[writable]` Reverse lookup PDA
    ///   2. `[signer]`   Owner
    ///   3. `[]`         System program
    ///   4. `[]`         Rent sysvar
    SetReverseLookup,

    /// Remove the reverse lookup.
    ///
    /// Accounts:
    ///   0. `[writable]` Reverse lookup PDA
    ///   1. `[signer]`   Owner
    RemoveReverseLookup,

    /// Claim an expired name (after grace period).
    ///
    /// Accounts:
    ///   0. `[writable]` Name PDA
    ///   1. `[]`         Registry PDA
    ///   2. `[signer]`   New claimant
    ///   3. `[writable]` Treasury
    ///   4. `[]`         Clock sysvar
    ClaimExpiredName,
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
    let instruction = NameServiceInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        NameServiceInstruction::InitializeRegistry => {
            process_initialize_registry(program_id, accounts)
        }
        NameServiceInstruction::RegisterName { name } => {
            process_register_name(program_id, accounts, name)
        }
        NameServiceInstruction::RenewName => process_renew_name(program_id, accounts),
        NameServiceInstruction::TransferName { new_owner } => {
            process_transfer_name(program_id, accounts, new_owner)
        }
        NameServiceInstruction::SetRecord { key, value } => {
            process_set_record(program_id, accounts, key, value)
        }
        NameServiceInstruction::DeleteRecord { key } => {
            process_delete_record(program_id, accounts, key)
        }
        NameServiceInstruction::CreateSubdomain {
            label,
            subdomain_owner,
        } => process_create_subdomain(program_id, accounts, label, subdomain_owner),
        NameServiceInstruction::DeleteSubdomain => {
            process_delete_subdomain(program_id, accounts)
        }
        NameServiceInstruction::SetReverseLookup => {
            process_set_reverse_lookup(program_id, accounts)
        }
        NameServiceInstruction::RemoveReverseLookup => {
            process_remove_reverse_lookup(program_id, accounts)
        }
        NameServiceInstruction::ClaimExpiredName => {
            process_claim_expired_name(program_id, accounts)
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute the hash of a name (used for PDA derivation).
pub fn compute_name_hash(name: &[u8]) -> [u8; 32] {
    hash(name).to_bytes()
}

/// Validate a name string (lowercase alphanumeric + hyphens).
pub fn validate_name(name: &[u8]) -> Result<(), NameServiceError> {
    if name.len() > MAX_NAME_LEN {
        return Err(NameServiceError::NameTooLong);
    }
    if name.len() < MIN_NAME_LEN {
        return Err(NameServiceError::NameTooShort);
    }

    // Only a-z, 0-9, hyphen.
    for &b in name {
        if !((b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b) || b == b'-') {
            return Err(NameServiceError::InvalidNameChars);
        }
    }

    // Cannot start or end with hyphen.
    if name[0] == b'-' || name[name.len() - 1] == b'-' {
        return Err(NameServiceError::InvalidHyphenPlacement);
    }

    Ok(())
}

/// Calculate the registration fee based on name length.
pub fn calculate_fee(name_len: usize) -> u64 {
    if name_len <= 2 {
        BASE_FEE_LAMPORTS * ULTRA_SHORT_NAME_MULTIPLIER
    } else if name_len <= 4 {
        BASE_FEE_LAMPORTS * SHORT_NAME_MULTIPLIER
    } else {
        BASE_FEE_LAMPORTS
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_registry(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let registry_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let treasury_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_pda, bump) =
        Pubkey::find_program_address(&[REGISTRY_SEED], program_id);
    if registry_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    if registry_info.data_len() > 0 {
        let existing = NameRegistry::try_from_slice(&registry_info.data.borrow());
        if let Ok(reg) = existing {
            if reg.is_initialized {
                return Err(NameServiceError::AlreadyInitialized.into());
            }
        }
    }

    let registry = NameRegistry {
        is_initialized: true,
        authority: *authority_info.key,
        treasury: *treasury_info.key,
        total_names: 0,
        bump,
    };

    let data = registry.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    registry_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Name service registry initialized");
    Ok(())
}

fn process_register_name(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    name: Vec<u8>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let registry_info = next_account_info(account_iter)?;
    let name_info = next_account_info(account_iter)?;
    let registrant_info = next_account_info(account_iter)?;
    let _treasury_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !registrant_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate name.
    validate_name(&name)?;

    let name_hash = compute_name_hash(&name);
    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Verify name PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[NAME_SEED, &name_hash],
        program_id,
    );
    if name_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check if already registered and not expired.
    if name_info.data_len() > 0 {
        let existing = Name::try_from_slice(&name_info.data.borrow());
        if let Ok(n) = existing {
            if n.is_initialized {
                let grace_deadline = n
                    .expires_at
                    .checked_add(GRACE_PERIOD_SECS)
                    .ok_or(NameServiceError::Overflow)?;
                if now < grace_deadline {
                    return Err(NameServiceError::NameAlreadyRegistered.into());
                }
            }
        }
    }

    // Calculate fee.
    let _fee = calculate_fee(name.len());
    // In production: transfer fee from registrant to treasury via CPI.

    let expires_at = now
        .checked_add(REGISTRATION_PERIOD_SECS)
        .ok_or(NameServiceError::Overflow)?;

    let name_record = Name {
        is_initialized: true,
        name: name.clone(),
        name_hash,
        owner: *registrant_info.key,
        expires_at,
        registered_at: now,
        records: vec![],
        has_reverse: false,
        bump,
    };

    // Update registry counter.
    let mut registry = NameRegistry::try_from_slice(&registry_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;
    registry.total_names = registry
        .total_names
        .checked_add(1)
        .ok_or(NameServiceError::Overflow)?;

    let reg_data = registry.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    registry_info.data.borrow_mut()[..reg_data.len()].copy_from_slice(&reg_data);

    let name_data = name_record.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..name_data.len()].copy_from_slice(&name_data);

    let display_name = String::from_utf8_lossy(&name);
    msg!(
        "Name registered: {}{} by {} (expires {})",
        display_name,
        TLD,
        registrant_info.key,
        expires_at
    );
    Ok(())
}

fn process_renew_name(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let _treasury_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if !name.is_initialized {
        return Err(NameServiceError::NameNotFound.into());
    }

    // Only owner can renew (before or during grace period).
    if name.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // If already expired past grace period, cannot renew (must re-register).
    let grace_deadline = name
        .expires_at
        .checked_add(GRACE_PERIOD_SECS)
        .ok_or(NameServiceError::Overflow)?;
    if now > grace_deadline {
        return Err(NameServiceError::NameExpired.into());
    }

    // Extend from current expiry (or from now if already expired).
    let base_time = if now > name.expires_at { now } else { name.expires_at };
    name.expires_at = base_time
        .checked_add(REGISTRATION_PERIOD_SECS)
        .ok_or(NameServiceError::Overflow)?;

    // In production: collect renewal fee via CPI.

    let data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    let display_name = String::from_utf8_lossy(&name.name);
    msg!(
        "Name renewed: {}{} (new expiry: {})",
        display_name,
        TLD,
        name.expires_at
    );
    Ok(())
}

fn process_transfer_name(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_owner: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let _new_owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if name.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    let old_owner = name.owner;
    name.owner = new_owner;
    // Clear reverse lookup flag on transfer.
    name.has_reverse = false;

    let data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    let display_name = String::from_utf8_lossy(&name.name);
    msg!(
        "Name transferred: {}{} from {} to {}",
        display_name,
        TLD,
        old_owner,
        new_owner
    );
    Ok(())
}

fn process_set_record(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    key: RecordKey,
    value: Vec<u8>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if value.len() > MAX_RECORD_VALUE_LEN {
        return Err(NameServiceError::RecordValueTooLong.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if name.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    // Check name hasn't expired.
    if clock.unix_timestamp > name.expires_at {
        return Err(NameServiceError::NameExpired.into());
    }

    // Update existing record or add new one.
    let existing_pos = name.records.iter().position(|r| r.key == key);
    match existing_pos {
        Some(pos) => {
            name.records[pos].value = value;
            msg!("Record updated on {}{}", String::from_utf8_lossy(&name.name), TLD);
        }
        None => {
            if name.records.len() >= MAX_RECORDS {
                return Err(NameServiceError::TooManyRecords.into());
            }
            name.records.push(NameRecord { key, value });
            msg!("Record added to {}{}", String::from_utf8_lossy(&name.name), TLD);
        }
    }

    let data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    Ok(())
}

fn process_delete_record(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    key: RecordKey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if name.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    let pos = name
        .records
        .iter()
        .position(|r| r.key == key)
        .ok_or(NameServiceError::RecordNotFound)?;

    name.records.remove(pos);

    let data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Record deleted from {}{}", String::from_utf8_lossy(&name.name), TLD);
    Ok(())
}

fn process_create_subdomain(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    label: Vec<u8>,
    subdomain_owner: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let parent_info = next_account_info(account_iter)?;
    let subdomain_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if label.len() > MAX_SUBDOMAIN_LEN {
        return Err(NameServiceError::SubdomainTooLong.into());
    }

    // Validate subdomain label characters.
    for &b in &label {
        if !((b'a'..=b'z').contains(&b) || (b'0'..=b'9').contains(&b) || b == b'-') {
            return Err(NameServiceError::InvalidNameChars.into());
        }
    }

    let parent = Name::try_from_slice(&parent_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if !parent.is_initialized {
        return Err(NameServiceError::InvalidParentName.into());
    }

    if parent.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    if clock.unix_timestamp > parent.expires_at {
        return Err(NameServiceError::InvalidParentName.into());
    }

    // Derive subdomain PDA from label + parent hash.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[SUBDOMAIN_SEED, &label, &parent.name_hash],
        program_id,
    );
    if subdomain_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let subdomain = Subdomain {
        is_initialized: true,
        label: label.clone(),
        parent_name_hash: parent.name_hash,
        owner: subdomain_owner,
        records: vec![],
        bump,
    };

    let data = subdomain.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    subdomain_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Subdomain created: {}.{}{} owned by {}",
        String::from_utf8_lossy(&label),
        String::from_utf8_lossy(&parent.name),
        TLD,
        subdomain_owner
    );
    Ok(())
}

fn process_delete_subdomain(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let parent_info = next_account_info(account_iter)?;
    let subdomain_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let parent = Name::try_from_slice(&parent_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    // Parent owner can always delete subdomains.
    if parent.owner != *owner_info.key {
        // Subdomain owner can also delete their own subdomain.
        let subdomain = Subdomain::try_from_slice(&subdomain_info.data.borrow())
            .map_err(|_| NameServiceError::InvalidAccountData)?;
        if subdomain.owner != *owner_info.key {
            return Err(NameServiceError::NotOwner.into());
        }
    }

    // Mark as uninitialized (effectively deletes).
    let mut subdomain = Subdomain::try_from_slice(&subdomain_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;
    subdomain.is_initialized = false;
    subdomain.records.clear();

    let data = subdomain.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    subdomain_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Subdomain deleted");
    Ok(())
}

fn process_set_reverse_lookup(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let reverse_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if name.owner != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    // Verify reverse PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[REVERSE_SEED, owner_info.key.as_ref()],
        program_id,
    );
    if reverse_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let reverse = ReverseLookup {
        is_initialized: true,
        address: *owner_info.key,
        name_hash: name.name_hash,
        bump,
    };

    name.has_reverse = true;

    let reverse_data = reverse.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    reverse_info.data.borrow_mut()[..reverse_data.len()].copy_from_slice(&reverse_data);

    let name_data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..name_data.len()].copy_from_slice(&name_data);

    msg!(
        "Reverse lookup set: {} -> {}{}",
        owner_info.key,
        String::from_utf8_lossy(&name.name),
        TLD
    );
    Ok(())
}

fn process_remove_reverse_lookup(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let reverse_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;

    if !owner_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut reverse = ReverseLookup::try_from_slice(&reverse_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if reverse.address != *owner_info.key {
        return Err(NameServiceError::NotOwner.into());
    }

    reverse.is_initialized = false;

    let data = reverse.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    reverse_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Reverse lookup removed for {}", owner_info.key);
    Ok(())
}

fn process_claim_expired_name(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let name_info = next_account_info(account_iter)?;
    let _registry_info = next_account_info(account_iter)?;
    let claimant_info = next_account_info(account_iter)?;
    let _treasury_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !claimant_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let mut name = Name::try_from_slice(&name_info.data.borrow())
        .map_err(|_| NameServiceError::InvalidAccountData)?;

    if !name.is_initialized {
        return Err(NameServiceError::NameNotFound.into());
    }

    // Must be past expiry + grace period.
    let grace_deadline = name
        .expires_at
        .checked_add(GRACE_PERIOD_SECS)
        .ok_or(NameServiceError::Overflow)?;

    if now < grace_deadline {
        return Err(NameServiceError::NameNotExpired.into());
    }

    // In production: collect registration fee from claimant.

    // Reset the name for the new owner.
    let old_owner = name.owner;
    name.owner = *claimant_info.key;
    name.expires_at = now
        .checked_add(REGISTRATION_PERIOD_SECS)
        .ok_or(NameServiceError::Overflow)?;
    name.registered_at = now;
    name.records.clear();
    name.has_reverse = false;

    let data = name.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    name_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    let display_name = String::from_utf8_lossy(&name.name);
    msg!(
        "Expired name claimed: {}{} by {} (prev owner: {})",
        display_name,
        TLD,
        claimant_info.key,
        old_owner
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_validation_valid() {
        assert!(validate_name(b"alice").is_ok());
        assert!(validate_name(b"bob-wallet").is_ok());
        assert!(validate_name(b"dao-treasury-2024").is_ok());
        assert!(validate_name(b"a").is_ok());
        assert!(validate_name(b"x".repeat(MAX_NAME_LEN).as_slice()).is_ok());
    }

    #[test]
    fn test_name_validation_invalid() {
        // Too long.
        assert!(matches!(
            validate_name(&vec![b'a'; MAX_NAME_LEN + 1]),
            Err(NameServiceError::NameTooLong)
        ));

        // Too short.
        assert!(matches!(
            validate_name(b""),
            Err(NameServiceError::NameTooShort)
        ));

        // Invalid chars.
        assert!(matches!(
            validate_name(b"Alice"),
            Err(NameServiceError::InvalidNameChars)
        ));
        assert!(matches!(
            validate_name(b"alice.bob"),
            Err(NameServiceError::InvalidNameChars)
        ));
        assert!(matches!(
            validate_name(b"alice bob"),
            Err(NameServiceError::InvalidNameChars)
        ));

        // Starts/ends with hyphen.
        assert!(matches!(
            validate_name(b"-alice"),
            Err(NameServiceError::InvalidHyphenPlacement)
        ));
        assert!(matches!(
            validate_name(b"alice-"),
            Err(NameServiceError::InvalidHyphenPlacement)
        ));
    }

    #[test]
    fn test_fee_calculation() {
        // 1-2 chars: 100x base.
        assert_eq!(calculate_fee(1), BASE_FEE_LAMPORTS * ULTRA_SHORT_NAME_MULTIPLIER);
        assert_eq!(calculate_fee(2), BASE_FEE_LAMPORTS * ULTRA_SHORT_NAME_MULTIPLIER);

        // 3-4 chars: 10x base.
        assert_eq!(calculate_fee(3), BASE_FEE_LAMPORTS * SHORT_NAME_MULTIPLIER);
        assert_eq!(calculate_fee(4), BASE_FEE_LAMPORTS * SHORT_NAME_MULTIPLIER);

        // 5+ chars: base fee.
        assert_eq!(calculate_fee(5), BASE_FEE_LAMPORTS);
        assert_eq!(calculate_fee(10), BASE_FEE_LAMPORTS);
        assert_eq!(calculate_fee(64), BASE_FEE_LAMPORTS);
    }

    #[test]
    fn test_name_hash() {
        let h1 = compute_name_hash(b"alice");
        let h2 = compute_name_hash(b"alice");
        assert_eq!(h1, h2);

        let h3 = compute_name_hash(b"bob");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_name_serialization() {
        let name = Name {
            is_initialized: true,
            name: b"alice".to_vec(),
            name_hash: compute_name_hash(b"alice"),
            owner: Pubkey::new_unique(),
            expires_at: 1_700_000_000 + REGISTRATION_PERIOD_SECS,
            registered_at: 1_700_000_000,
            records: vec![
                NameRecord {
                    key: RecordKey::Wallet,
                    value: Pubkey::new_unique().to_bytes().to_vec(),
                },
                NameRecord {
                    key: RecordKey::Twitter,
                    value: b"@alice".to_vec(),
                },
            ],
            has_reverse: true,
            bump: 255,
        };
        let data = name.try_to_vec().unwrap();
        let decoded = Name::try_from_slice(&data).unwrap();
        assert_eq!(decoded.name, b"alice");
        assert_eq!(decoded.records.len(), 2);
        assert!(decoded.has_reverse);
    }

    #[test]
    fn test_subdomain_serialization() {
        let sub = Subdomain {
            is_initialized: true,
            label: b"mail".to_vec(),
            parent_name_hash: compute_name_hash(b"alice"),
            owner: Pubkey::new_unique(),
            records: vec![],
            bump: 254,
        };
        let data = sub.try_to_vec().unwrap();
        let decoded = Subdomain::try_from_slice(&data).unwrap();
        assert_eq!(decoded.label, b"mail");
    }

    #[test]
    fn test_reverse_lookup_serialization() {
        let reverse = ReverseLookup {
            is_initialized: true,
            address: Pubkey::new_unique(),
            name_hash: compute_name_hash(b"alice"),
            bump: 253,
        };
        let data = reverse.try_to_vec().unwrap();
        let decoded = ReverseLookup::try_from_slice(&data).unwrap();
        assert_eq!(decoded.name_hash, compute_name_hash(b"alice"));
    }

    #[test]
    fn test_registration_period() {
        assert_eq!(REGISTRATION_PERIOD_SECS, 31_536_000); // 365 days
        assert_eq!(GRACE_PERIOD_SECS, 2_592_000); // 30 days
    }

    #[test]
    fn test_record_keys() {
        let wallet = RecordKey::Wallet;
        let custom = RecordKey::Custom {
            key: b"custom-field".to_vec(),
        };
        let r1 = NameRecord {
            key: wallet.clone(),
            value: vec![1, 2, 3],
        };
        let r2 = NameRecord {
            key: custom,
            value: vec![4, 5, 6],
        };
        assert_ne!(
            r1.key.try_to_vec().unwrap(),
            r2.key.try_to_vec().unwrap()
        );
    }
}
