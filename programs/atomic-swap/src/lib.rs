use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    hash::hash,
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

solana_program::declare_id!("Swap1111111111111111111111111111111111111111");

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum lock duration: 7 days in seconds.
const MAX_LOCK_DURATION: i64 = 7 * 24 * 60 * 60;

/// Minimum lock duration: 10 minutes in seconds.
const MIN_LOCK_DURATION: i64 = 10 * 60;

/// Size of the SHA-256 hash used as the hash lock.
const HASH_SIZE: usize = 32;

/// Size of the secret preimage.
const SECRET_SIZE: usize = 32;

/// Seed prefix for swap PDA accounts.
const SWAP_SEED: &[u8] = b"atomic_swap";

/// Seed prefix for swap vault PDA accounts.
const VAULT_SEED: &[u8] = b"swap_vault";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum SwapError {
    #[error("Swap has already been initialized")]
    AlreadyInitialized,

    #[error("Swap is not in the expected state")]
    InvalidState,

    #[error("Lock duration is out of valid range")]
    InvalidLockDuration,

    #[error("The provided secret does not match the hash lock")]
    InvalidSecret,

    #[error("The swap has not yet expired; cannot refund")]
    NotYetExpired,

    #[error("The swap has already expired; cannot claim")]
    AlreadyExpired,

    #[error("Insufficient funds for the swap")]
    InsufficientFunds,

    #[error("Unauthorized: signer does not match expected party")]
    Unauthorized,

    #[error("Invalid amount: must be greater than zero")]
    InvalidAmount,

    #[error("Arithmetic overflow")]
    Overflow,

    #[error("Invalid PDA derivation")]
    InvalidPDA,
}

impl From<SwapError> for ProgramError {
    fn from(e: SwapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Status of an atomic swap.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum SwapState {
    /// Swap has been created and funds are locked.
    Open = 0,
    /// Swap has been claimed by the counterparty with a valid secret.
    Claimed = 1,
    /// Swap has been refunded to the initiator after expiry.
    Refunded = 2,
}

/// Type of asset being swapped.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum AssetType {
    /// Native PRISM (lamports).
    NativePrism = 0,
    /// SPL Token.
    SplToken = 1,
}

/// On-chain state for a single atomic swap (HTLC).
///
/// Layout:
///   is_initialized (1) + state (1) + asset_type (1) + initiator (32)
///   + counterparty (32) + amount (8) + hash_lock (32) + secret (32)
///   + lock_duration (8) + created_at (8) + token_mint (32) + bump (1)
///   = 188 bytes
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct SwapAccount {
    /// Whether this account has been initialized.
    pub is_initialized: bool,
    /// Current state of the swap.
    pub state: SwapState,
    /// Type of asset being swapped.
    pub asset_type: AssetType,
    /// The party that created the swap and locked funds.
    pub initiator: Pubkey,
    /// The counterparty that can claim with the correct secret.
    pub counterparty: Pubkey,
    /// Amount of tokens or lamports locked.
    pub amount: u64,
    /// SHA-256 hash of the secret (the "hash lock").
    pub hash_lock: [u8; HASH_SIZE],
    /// The revealed secret (filled on claim, zeroed until then).
    pub secret: [u8; SECRET_SIZE],
    /// Lock duration in seconds from creation.
    pub lock_duration: i64,
    /// Unix timestamp when the swap was created.
    pub created_at: i64,
    /// SPL token mint (zero pubkey if native PRISM).
    pub token_mint: Pubkey,
    /// PDA bump seed.
    pub bump: u8,
}

impl SwapAccount {
    pub const SIZE: usize = 188;

    /// Returns true if the time lock has expired.
    pub fn is_expired(&self, current_time: i64) -> bool {
        current_time >= self.created_at.saturating_add(self.lock_duration)
    }

    /// Verifies that a given secret hashes to the stored hash lock.
    pub fn verify_secret(&self, secret: &[u8; SECRET_SIZE]) -> bool {
        let computed = hash(secret);
        computed.to_bytes() == self.hash_lock
    }
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum SwapInstruction {
    /// Create a new atomic swap (HTLC) and lock funds.
    ///
    /// Accounts:
    ///   0. `[writable, signer]` Initiator (payer)
    ///   1. `[writable]`         Swap PDA account
    ///   2. `[writable]`         Vault PDA (holds locked funds)
    ///   3. `[]`                 System program
    ///   4. `[]`                 (Optional) SPL Token program
    ///   5. `[]`                 (Optional) Initiator token account
    ///   6. `[]`                 (Optional) Vault token account
    CreateSwap {
        /// SHA-256 hash of the secret known only to the initiator.
        hash_lock: [u8; HASH_SIZE],
        /// Amount to lock (lamports or token smallest unit).
        amount: u64,
        /// Counterparty who can claim the swap.
        counterparty: Pubkey,
        /// Duration in seconds before initiator can reclaim.
        lock_duration: i64,
        /// Asset type: native PRISM or SPL token.
        asset_type: AssetType,
        /// Token mint address (ignored for native PRISM).
        token_mint: Pubkey,
    },

    /// Claim a swap by revealing the secret preimage.
    ///
    /// Accounts:
    ///   0. `[signer]`           Counterparty (claimer)
    ///   1. `[writable]`         Swap PDA account
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Counterparty receive account
    ///   4. `[]`                 System program
    ///   5. `[]`                 (Optional) SPL Token program
    ClaimSwap {
        /// The secret preimage whose SHA-256 hash matches hash_lock.
        secret: [u8; SECRET_SIZE],
    },

    /// Refund a swap after the time lock has expired.
    ///
    /// Accounts:
    ///   0. `[signer]`           Initiator
    ///   1. `[writable]`         Swap PDA account
    ///   2. `[writable]`         Vault PDA
    ///   3. `[writable]`         Initiator receive account
    ///   4. `[]`                 System program
    ///   5. `[]`                 (Optional) SPL Token program
    RefundSwap,
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
    let instruction = SwapInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        SwapInstruction::CreateSwap {
            hash_lock,
            amount,
            counterparty,
            lock_duration,
            asset_type,
            token_mint,
        } => process_create_swap(
            program_id,
            accounts,
            hash_lock,
            amount,
            counterparty,
            lock_duration,
            asset_type,
            token_mint,
        ),
        SwapInstruction::ClaimSwap { secret } => {
            process_claim_swap(program_id, accounts, secret)
        }
        SwapInstruction::RefundSwap => process_refund_swap(program_id, accounts),
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_create_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    hash_lock: [u8; HASH_SIZE],
    amount: u64,
    counterparty: Pubkey,
    lock_duration: i64,
    asset_type: AssetType,
    token_mint: Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let initiator = next_account_info(account_iter)?;
    let swap_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    // Validate signer
    if !initiator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate amount
    if amount == 0 {
        return Err(SwapError::InvalidAmount.into());
    }

    // Validate lock duration
    if lock_duration < MIN_LOCK_DURATION || lock_duration > MAX_LOCK_DURATION {
        return Err(SwapError::InvalidLockDuration.into());
    }

    // Derive swap PDA
    let (swap_pda, swap_bump) = Pubkey::find_program_address(
        &[SWAP_SEED, &hash_lock, initiator.key.as_ref()],
        program_id,
    );
    if swap_pda != *swap_account.key {
        return Err(SwapError::InvalidPDA.into());
    }

    // Derive vault PDA
    let (vault_pda, _vault_bump) = Pubkey::find_program_address(
        &[VAULT_SEED, swap_account.key.as_ref()],
        program_id,
    );
    if vault_pda != *vault_account.key {
        return Err(SwapError::InvalidPDA.into());
    }

    // Create the swap PDA account
    let rent = Rent::get()?;
    let space = SwapAccount::SIZE;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            initiator.key,
            swap_account.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[initiator.clone(), swap_account.clone(), system_program.clone()],
        &[&[SWAP_SEED, &hash_lock, initiator.key.as_ref(), &[swap_bump]]],
    )?;

    // Get the current clock
    let clock = Clock::get()?;

    // For native PRISM transfers, move lamports from initiator to vault
    if asset_type == AssetType::NativePrism {
        // Create vault account to hold locked funds
        let vault_space = 0;
        let vault_lamports = amount;

        let (_vault_pda, vault_bump) = Pubkey::find_program_address(
            &[VAULT_SEED, swap_account.key.as_ref()],
            program_id,
        );

        invoke_signed(
            &system_instruction::create_account(
                initiator.key,
                vault_account.key,
                vault_lamports,
                vault_space,
                program_id,
            ),
            &[initiator.clone(), vault_account.clone(), system_program.clone()],
            &[&[VAULT_SEED, swap_account.key.as_ref(), &[vault_bump]]],
        )?;
    }
    // For SPL tokens, the caller must set up the token accounts and transfer
    // through the SPL Token program. The instruction accounts 4-6 are used.

    // Initialize the swap state
    let swap = SwapAccount {
        is_initialized: true,
        state: SwapState::Open,
        asset_type,
        initiator: *initiator.key,
        counterparty,
        amount,
        hash_lock,
        secret: [0u8; SECRET_SIZE],
        lock_duration,
        created_at: clock.unix_timestamp,
        token_mint,
        bump: swap_bump,
    };

    swap.serialize(&mut &mut swap_account.data.borrow_mut()[..])?;

    msg!(
        "Atomic swap created: {} lamports/tokens locked for {} seconds",
        amount,
        lock_duration
    );
    msg!("Hash lock: {:?}", &hash_lock[..8]);

    Ok(())
}

fn process_claim_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    secret: [u8; SECRET_SIZE],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let claimer = next_account_info(account_iter)?;
    let swap_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let claimer_receive = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    // Validate signer
    if !claimer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate swap account ownership
    if swap_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize swap state
    let mut swap = SwapAccount::try_from_slice(&swap_account.data.borrow())?;

    // Validate state
    if !swap.is_initialized {
        return Err(SwapError::InvalidState.into());
    }
    if swap.state != SwapState::Open {
        return Err(SwapError::InvalidState.into());
    }

    // Validate claimer is the designated counterparty
    if *claimer.key != swap.counterparty {
        return Err(SwapError::Unauthorized.into());
    }

    // Check not expired
    let clock = Clock::get()?;
    if swap.is_expired(clock.unix_timestamp) {
        return Err(SwapError::AlreadyExpired.into());
    }

    // Verify the secret matches the hash lock
    if !swap.verify_secret(&secret) {
        return Err(SwapError::InvalidSecret.into());
    }

    // Transfer funds from vault to claimer
    if swap.asset_type == AssetType::NativePrism {
        // Transfer lamports from vault PDA to claimer's receive account
        let (_vault_pda, vault_bump) = Pubkey::find_program_address(
            &[VAULT_SEED, swap_account.key.as_ref()],
            program_id,
        );

        // Drain vault: move all lamports to claimer
        let vault_lamports = vault_account.lamports();
        **vault_account.try_borrow_mut_lamports()? = 0;
        **claimer_receive.try_borrow_mut_lamports()? = claimer_receive
            .lamports()
            .checked_add(vault_lamports)
            .ok_or(SwapError::Overflow)?;

        // Suppress unused variable warning -- vault_bump is needed for PDA
        // derivation in a production SPL token path.
        let _ = vault_bump;
    }
    // For SPL tokens, invoke the SPL Token transfer with PDA signer seeds.

    // Update swap state
    swap.state = SwapState::Claimed;
    swap.secret = secret;
    swap.serialize(&mut &mut swap_account.data.borrow_mut()[..])?;

    msg!("Atomic swap claimed successfully by counterparty");
    msg!("Secret revealed: {:?}", &secret[..8]);

    Ok(())
}

fn process_refund_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let initiator = next_account_info(account_iter)?;
    let swap_account = next_account_info(account_iter)?;
    let vault_account = next_account_info(account_iter)?;
    let initiator_receive = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    // Validate signer
    if !initiator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Validate swap account ownership
    if swap_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Deserialize swap state
    let mut swap = SwapAccount::try_from_slice(&swap_account.data.borrow())?;

    // Validate state
    if !swap.is_initialized {
        return Err(SwapError::InvalidState.into());
    }
    if swap.state != SwapState::Open {
        return Err(SwapError::InvalidState.into());
    }

    // Validate the initiator is the original creator
    if *initiator.key != swap.initiator {
        return Err(SwapError::Unauthorized.into());
    }

    // Ensure the time lock has expired
    let clock = Clock::get()?;
    if !swap.is_expired(clock.unix_timestamp) {
        return Err(SwapError::NotYetExpired.into());
    }

    // Refund: transfer funds from vault back to initiator
    if swap.asset_type == AssetType::NativePrism {
        let vault_lamports = vault_account.lamports();
        **vault_account.try_borrow_mut_lamports()? = 0;
        **initiator_receive.try_borrow_mut_lamports()? = initiator_receive
            .lamports()
            .checked_add(vault_lamports)
            .ok_or(SwapError::Overflow)?;
    }
    // For SPL tokens, invoke the SPL Token transfer with PDA signer seeds.

    // Update state
    swap.state = SwapState::Refunded;
    swap.serialize(&mut &mut swap_account.data.borrow_mut()[..])?;

    msg!("Atomic swap refunded to initiator after timeout");

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::hash::hash;

    #[test]
    fn test_secret_verification() {
        let secret = [42u8; SECRET_SIZE];
        let hash_lock = hash(&secret).to_bytes();

        let swap = SwapAccount {
            is_initialized: true,
            state: SwapState::Open,
            asset_type: AssetType::NativePrism,
            initiator: Pubkey::new_unique(),
            counterparty: Pubkey::new_unique(),
            amount: 1_000_000,
            hash_lock,
            secret: [0u8; SECRET_SIZE],
            lock_duration: 3600,
            created_at: 1000,
            token_mint: Pubkey::default(),
            bump: 255,
        };

        assert!(swap.verify_secret(&secret));
        assert!(!swap.verify_secret(&[0u8; SECRET_SIZE]));
    }

    #[test]
    fn test_expiry_check() {
        let swap = SwapAccount {
            is_initialized: true,
            state: SwapState::Open,
            asset_type: AssetType::NativePrism,
            initiator: Pubkey::new_unique(),
            counterparty: Pubkey::new_unique(),
            amount: 1_000_000,
            hash_lock: [0u8; HASH_SIZE],
            secret: [0u8; SECRET_SIZE],
            lock_duration: 3600,
            created_at: 1000,
            token_mint: Pubkey::default(),
            bump: 255,
        };

        // Before expiry
        assert!(!swap.is_expired(2000));
        // Exactly at expiry
        assert!(swap.is_expired(4600));
        // After expiry
        assert!(swap.is_expired(5000));
    }

    #[test]
    fn test_swap_state_transitions() {
        assert_ne!(SwapState::Open, SwapState::Claimed);
        assert_ne!(SwapState::Open, SwapState::Refunded);
        assert_ne!(SwapState::Claimed, SwapState::Refunded);
    }
}
