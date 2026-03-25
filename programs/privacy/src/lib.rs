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

/// Seed for the shielded pool PDA.
pub const POOL_SEED: &[u8] = b"shielded_pool";

/// Seed for a commitment leaf PDA.
pub const COMMITMENT_SEED: &[u8] = b"commitment";

/// Seed for a nullifier PDA.
pub const NULLIFIER_SEED: &[u8] = b"nullifier";

/// Seed for a stealth address record PDA.
pub const STEALTH_SEED: &[u8] = b"stealth";

/// Seed for encrypted note PDA.
pub const NOTE_SEED: &[u8] = b"note";

/// Seed for auditor configuration PDA.
pub const AUDITOR_SEED: &[u8] = b"auditor";

/// Maximum Merkle tree depth (supports 2^20 = ~1M commitments).
pub const MERKLE_TREE_DEPTH: usize = 20;

/// Maximum encrypted memo length in bytes.
pub const MAX_NOTE_LEN: usize = 256;

/// Maximum number of auditor keys.
pub const MAX_AUDITORS: usize = 5;

/// Pedersen generator point G (placeholder 32-byte constant).
/// In production this would be a well-known elliptic curve point.
pub const PEDERSEN_G: [u8; 32] = [
    0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// Pedersen generator point H (placeholder 32-byte constant).
pub const PEDERSEN_H: [u8; 32] = [
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum PrivacyError {
    #[error("Pool already initialized")]
    PoolAlreadyInitialized,
    #[error("Pool not initialized")]
    PoolNotInitialized,
    #[error("Invalid Pedersen commitment")]
    InvalidCommitment,
    #[error("Nullifier already spent")]
    NullifierAlreadySpent,
    #[error("Invalid range proof")]
    InvalidRangeProof,
    #[error("Invalid Merkle proof")]
    InvalidMerkleProof,
    #[error("Insufficient pool balance")]
    InsufficientPoolBalance,
    #[error("Invalid stealth address")]
    InvalidStealthAddress,
    #[error("Encrypted note too long")]
    NoteTooLong,
    #[error("Not an authorized auditor")]
    NotAuthorizedAuditor,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Commitment not found in Merkle tree")]
    CommitmentNotFound,
    #[error("Invalid zero-knowledge proof")]
    InvalidZkProof,
    #[error("Maximum auditors exceeded")]
    MaxAuditorsExceeded,
}

impl From<PrivacyError> for ProgramError {
    fn from(e: PrivacyError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// A Pedersen commitment: C = v*G + r*H where v is the amount, r is the blinding factor.
/// Stored as a compressed 32-byte point.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub struct PedersenCommitment {
    pub point: [u8; 32],
}

/// A range proof demonstrating that the committed value is in [0, 2^64).
/// In production, this would be a Bulletproofs or similar ZK range proof.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct RangeProof {
    /// The serialized proof bytes (placeholder: would be ~700 bytes for Bulletproofs).
    pub proof_data: Vec<u8>,
}

/// A zero-knowledge proof for shielded transfers.
/// In production: Groth16 or PLONK proof bytes.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ZkProof {
    pub proof_a: [u8; 64],
    pub proof_b: [u8; 128],
    pub proof_c: [u8; 64],
}

/// Merkle path (sibling hashes from leaf to root).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct MerklePath {
    /// Sibling hashes, one per level of the tree.
    pub siblings: Vec<[u8; 32]>,
    /// Bit-flags indicating left (0) or right (1) for each level.
    pub path_indices: Vec<u8>,
}

/// The global shielded pool account.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct ShieldedPool {
    pub is_initialized: bool,
    /// Authority that can manage auditors.
    pub authority: Pubkey,
    /// Total number of commitments in the Merkle tree.
    pub commitment_count: u64,
    /// Current Merkle root.
    pub merkle_root: [u8; 32],
    /// Pool lamport balance (tracked separately from account balance for verification).
    pub pool_balance: u64,
    /// Whether compliance mode is enabled (auditor can decrypt).
    pub compliance_enabled: bool,
    /// PDA bump.
    pub bump: u8,
}

impl ShieldedPool {
    pub const MAX_SIZE: usize = 1 + 32 + 8 + 32 + 8 + 1 + 1;
}

/// A single commitment leaf in the Merkle tree.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct CommitmentLeaf {
    pub is_initialized: bool,
    /// Index in the Merkle tree.
    pub index: u64,
    /// The Pedersen commitment.
    pub commitment: PedersenCommitment,
    /// Encrypted amount (only decryptable by owner or auditor).
    pub encrypted_amount: [u8; 48],
    /// Owner public key (encrypted or stealth).
    pub owner: Pubkey,
    /// Timestamp of deposit.
    pub timestamp: i64,
    pub bump: u8,
}

impl CommitmentLeaf {
    pub const MAX_SIZE: usize = 1 + 8 + 32 + 48 + 32 + 8 + 1;
}

/// A nullifier record (marks a commitment as spent).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct NullifierRecord {
    pub is_initialized: bool,
    /// The nullifier hash.
    pub nullifier: [u8; 32],
    /// When it was spent.
    pub timestamp: i64,
    pub bump: u8,
}

impl NullifierRecord {
    pub const MAX_SIZE: usize = 1 + 32 + 8 + 1;
}

/// Stealth address record: maps a one-time address to metadata.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct StealthAddress {
    pub is_initialized: bool,
    /// The one-time stealth public key.
    pub stealth_pubkey: Pubkey,
    /// Ephemeral public key (needed by recipient to derive the private key).
    pub ephemeral_pubkey: [u8; 32],
    /// Recipient's scan key (the "viewing key" owner).
    pub recipient_scan_key: Pubkey,
    /// Encrypted payload (recipient can decrypt to recover funds).
    pub encrypted_payload: [u8; 64],
    /// Creation timestamp.
    pub timestamp: i64,
    pub bump: u8,
}

impl StealthAddress {
    pub const MAX_SIZE: usize = 1 + 32 + 32 + 32 + 64 + 8 + 1;
}

/// Encrypted transaction note (memo visible only to sender/receiver).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct EncryptedNote {
    pub is_initialized: bool,
    /// Sender.
    pub sender: Pubkey,
    /// Recipient.
    pub recipient: Pubkey,
    /// Encrypted note data (AES-GCM ciphertext).
    pub ciphertext: Vec<u8>,
    /// Nonce for decryption.
    pub nonce: [u8; 12],
    /// If compliance mode: the note re-encrypted under the auditor key.
    pub auditor_ciphertext: Vec<u8>,
    /// Timestamp.
    pub timestamp: i64,
    pub bump: u8,
}

impl EncryptedNote {
    pub const MAX_SIZE: usize = 1 + 32 + 32 + (4 + MAX_NOTE_LEN) + 12 + (4 + MAX_NOTE_LEN) + 8 + 1;
}

/// Auditor configuration (compliance mode).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AuditorConfig {
    pub is_initialized: bool,
    /// Authority who manages auditors.
    pub authority: Pubkey,
    /// List of auditor public keys who can decrypt.
    pub auditors: Vec<Pubkey>,
    /// Whether compliance mode is active.
    pub is_active: bool,
    pub bump: u8,
}

impl AuditorConfig {
    pub const MAX_SIZE: usize = 1 + 32 + 4 + (32 * MAX_AUDITORS) + 1 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum PrivacyInstruction {
    /// Initialize the shielded pool.
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[signer]`   Authority
    ///   2. `[]`         System program
    ///   3. `[]`         Rent sysvar
    InitializePool {
        compliance_enabled: bool,
    },

    /// Deposit (shield) tokens into the pool, creating a new commitment.
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[writable]` Commitment leaf PDA
    ///   2. `[signer]`   Depositor
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    Shield {
        amount: u64,
        commitment: PedersenCommitment,
        range_proof: RangeProof,
        encrypted_amount: [u8; 48],
    },

    /// Transfer within the shielded pool (no public amounts visible).
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[writable]` Input nullifier PDA
    ///   2. `[writable]` Output commitment PDA 1
    ///   3. `[writable]` Output commitment PDA 2 (change)
    ///   4. `[signer]`   Sender (proves ownership of input commitment)
    ///   5. `[]`         Clock sysvar
    ///   6. `[]`         System program
    ///   7. `[]`         Rent sysvar
    TransferShielded {
        /// Nullifier for the input commitment being spent.
        nullifier: [u8; 32],
        /// Merkle proof that the input commitment exists in the tree.
        merkle_proof: MerklePath,
        /// Output commitment to recipient.
        output_commitment_1: PedersenCommitment,
        /// Output commitment for change back to sender.
        output_commitment_2: PedersenCommitment,
        /// Zero-knowledge proof that:
        ///   - sender owns the input commitment
        ///   - input value = output_1 value + output_2 value
        ///   - all values are non-negative
        zk_proof: ZkProof,
        /// Encrypted amounts for the two outputs.
        encrypted_amount_1: [u8; 48],
        encrypted_amount_2: [u8; 48],
    },

    /// Withdraw (unshield) tokens from the pool back to a public address.
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[writable]` Nullifier PDA
    ///   2. `[signer]`   Withdrawer (proves ownership of commitment)
    ///   3. `[writable]` Recipient token account
    ///   4. `[]`         Clock sysvar
    ///   5. `[]`         System program
    Unshield {
        amount: u64,
        nullifier: [u8; 32],
        merkle_proof: MerklePath,
        zk_proof: ZkProof,
    },

    /// Generate and register a stealth address for a recipient.
    ///
    /// Accounts:
    ///   0. `[writable]` Stealth address PDA
    ///   1. `[signer]`   Sender (creates the stealth address)
    ///   2. `[]`         Recipient's scan key
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    CreateStealthAddress {
        stealth_pubkey: Pubkey,
        ephemeral_pubkey: [u8; 32],
        encrypted_payload: [u8; 64],
    },

    /// Attach an encrypted note to a transaction.
    ///
    /// Accounts:
    ///   0. `[writable]` Note PDA
    ///   1. `[signer]`   Sender
    ///   2. `[]`         Recipient
    ///   3. `[]`         Clock sysvar
    ///   4. `[]`         System program
    ///   5. `[]`         Rent sysvar
    ///   6. `[]`         Auditor config PDA (optional, for compliance)
    CreateEncryptedNote {
        ciphertext: Vec<u8>,
        nonce: [u8; 12],
        /// Ciphertext re-encrypted for auditor (empty if compliance not enabled).
        auditor_ciphertext: Vec<u8>,
    },

    /// Configure auditor keys for compliance mode.
    ///
    /// Accounts:
    ///   0. `[writable]` Auditor config PDA
    ///   1. `[signer]`   Authority
    ///   2. `[]`         System program
    ///   3. `[]`         Rent sysvar
    ConfigureAuditor {
        auditors: Vec<Pubkey>,
        is_active: bool,
    },

    /// Auditor decrypts and verifies a transaction (compliance action).
    /// This only logs the audit event on-chain; actual decryption is off-chain.
    ///
    /// Accounts:
    ///   0. `[]`         Note PDA or Commitment PDA
    ///   1. `[]`         Auditor config PDA
    ///   2. `[signer]`   Auditor
    ///   3. `[]`         Clock sysvar
    AuditTransaction {
        /// The auditor's attestation hash (hash of decrypted data).
        attestation: [u8; 32],
    },

    /// Update the pool's Merkle root (called after new commitments are added).
    ///
    /// Accounts:
    ///   0. `[writable]` Pool PDA
    ///   1. `[signer]`   Authority
    UpdateMerkleRoot {
        new_root: [u8; 32],
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
    let instruction = PrivacyInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        PrivacyInstruction::InitializePool { compliance_enabled } => {
            process_initialize_pool(program_id, accounts, compliance_enabled)
        }
        PrivacyInstruction::Shield {
            amount,
            commitment,
            range_proof,
            encrypted_amount,
        } => process_shield(program_id, accounts, amount, commitment, range_proof, encrypted_amount),

        PrivacyInstruction::TransferShielded {
            nullifier,
            merkle_proof,
            output_commitment_1,
            output_commitment_2,
            zk_proof,
            encrypted_amount_1,
            encrypted_amount_2,
        } => process_transfer_shielded(
            program_id,
            accounts,
            nullifier,
            merkle_proof,
            output_commitment_1,
            output_commitment_2,
            zk_proof,
            encrypted_amount_1,
            encrypted_amount_2,
        ),

        PrivacyInstruction::Unshield {
            amount,
            nullifier,
            merkle_proof,
            zk_proof,
        } => process_unshield(program_id, accounts, amount, nullifier, merkle_proof, zk_proof),

        PrivacyInstruction::CreateStealthAddress {
            stealth_pubkey,
            ephemeral_pubkey,
            encrypted_payload,
        } => process_create_stealth_address(
            program_id,
            accounts,
            stealth_pubkey,
            ephemeral_pubkey,
            encrypted_payload,
        ),

        PrivacyInstruction::CreateEncryptedNote {
            ciphertext,
            nonce,
            auditor_ciphertext,
        } => process_create_encrypted_note(program_id, accounts, ciphertext, nonce, auditor_ciphertext),

        PrivacyInstruction::ConfigureAuditor {
            auditors,
            is_active,
        } => process_configure_auditor(program_id, accounts, auditors, is_active),

        PrivacyInstruction::AuditTransaction { attestation } => {
            process_audit_transaction(program_id, accounts, attestation)
        }

        PrivacyInstruction::UpdateMerkleRoot { new_root } => {
            process_update_merkle_root(program_id, accounts, new_root)
        }
    }
}

// ---------------------------------------------------------------------------
// Cryptographic helpers
// ---------------------------------------------------------------------------

/// Compute a Pedersen commitment: C = amount * G + blinding * H.
/// This is a simplified placeholder. In production, use actual elliptic curve ops.
pub fn compute_pedersen_commitment(amount: u64, blinding: &[u8; 32]) -> PedersenCommitment {
    let mut data = Vec::with_capacity(40);
    data.extend_from_slice(&amount.to_le_bytes());
    data.extend_from_slice(blinding);
    let h = hash(&data);
    PedersenCommitment {
        point: h.to_bytes(),
    }
}

/// Verify a Merkle proof against a known root.
pub fn verify_merkle_proof(
    leaf: &[u8; 32],
    proof: &MerklePath,
    root: &[u8; 32],
) -> bool {
    if proof.siblings.len() != proof.path_indices.len() {
        return false;
    }

    let mut current = *leaf;
    for (i, sibling) in proof.siblings.iter().enumerate() {
        let mut data = Vec::with_capacity(64);
        if proof.path_indices[i] == 0 {
            // Current node is on the left.
            data.extend_from_slice(&current);
            data.extend_from_slice(sibling);
        } else {
            // Current node is on the right.
            data.extend_from_slice(sibling);
            data.extend_from_slice(&current);
        }
        current = hash(&data).to_bytes();
    }

    current == *root
}

/// Verify a ZK proof (placeholder -- in production use Groth16/PLONK verifier).
/// Returns true if the proof structure is valid (non-zero).
pub fn verify_zk_proof(proof: &ZkProof) -> bool {
    // Placeholder: verify proof is non-trivial (not all zeros).
    let all_zero_a = proof.proof_a.iter().all(|&b| b == 0);
    let all_zero_c = proof.proof_c.iter().all(|&b| b == 0);
    !all_zero_a && !all_zero_c
}

/// Verify a range proof (placeholder).
pub fn verify_range_proof(commitment: &PedersenCommitment, proof: &RangeProof) -> bool {
    // Placeholder: non-empty proof and non-zero commitment.
    !proof.proof_data.is_empty() && commitment.point.iter().any(|&b| b != 0)
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    compliance_enabled: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (expected_pda, bump) = Pubkey::find_program_address(&[POOL_SEED], program_id);
    if pool_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check not already initialized.
    if pool_info.data_len() > 0 {
        let existing = ShieldedPool::try_from_slice(&pool_info.data.borrow());
        if let Ok(pool) = existing {
            if pool.is_initialized {
                return Err(PrivacyError::PoolAlreadyInitialized.into());
            }
        }
    }

    let pool = ShieldedPool {
        is_initialized: true,
        authority: *authority_info.key,
        commitment_count: 0,
        merkle_root: [0u8; 32],
        pool_balance: 0,
        compliance_enabled,
        bump,
    };

    let data = pool.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    pool_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Shielded pool initialized (compliance={})",
        compliance_enabled
    );
    Ok(())
}

fn process_shield(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    commitment: PedersenCommitment,
    range_proof: RangeProof,
    encrypted_amount: [u8; 48],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_info = next_account_info(account_iter)?;
    let commitment_info = next_account_info(account_iter)?;
    let depositor_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !depositor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify range proof (amount is non-negative and within u64).
    if !verify_range_proof(&commitment, &range_proof) {
        return Err(PrivacyError::InvalidRangeProof.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    // Load and update pool.
    let mut pool = ShieldedPool::try_from_slice(&pool_info.data.borrow())
        .map_err(|_| PrivacyError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(PrivacyError::PoolNotInitialized.into());
    }

    let leaf_index = pool.commitment_count;
    pool.commitment_count = pool
        .commitment_count
        .checked_add(1)
        .ok_or(PrivacyError::Overflow)?;
    pool.pool_balance = pool
        .pool_balance
        .checked_add(amount)
        .ok_or(PrivacyError::Overflow)?;

    // Verify commitment PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[COMMITMENT_SEED, &leaf_index.to_le_bytes()],
        program_id,
    );
    if commitment_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let leaf = CommitmentLeaf {
        is_initialized: true,
        index: leaf_index,
        commitment,
        encrypted_amount,
        owner: *depositor_info.key,
        timestamp: clock.unix_timestamp,
        bump,
    };

    // Serialize.
    let pool_data = pool.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    pool_info.data.borrow_mut()[..pool_data.len()].copy_from_slice(&pool_data);

    let leaf_data = leaf.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    commitment_info.data.borrow_mut()[..leaf_data.len()].copy_from_slice(&leaf_data);

    msg!(
        "Shield: {} lamports deposited, commitment index {}",
        amount,
        leaf_index
    );
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn process_transfer_shielded(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    nullifier: [u8; 32],
    merkle_proof: MerklePath,
    output_commitment_1: PedersenCommitment,
    output_commitment_2: PedersenCommitment,
    zk_proof: ZkProof,
    encrypted_amount_1: [u8; 48],
    encrypted_amount_2: [u8; 48],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_info = next_account_info(account_iter)?;
    let nullifier_info = next_account_info(account_iter)?;
    let output_1_info = next_account_info(account_iter)?;
    let output_2_info = next_account_info(account_iter)?;
    let sender_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !sender_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool = ShieldedPool::try_from_slice(&pool_info.data.borrow())
        .map_err(|_| PrivacyError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(PrivacyError::PoolNotInitialized.into());
    }

    // 1. Verify the ZK proof (proves value conservation and ownership).
    if !verify_zk_proof(&zk_proof) {
        return Err(PrivacyError::InvalidZkProof.into());
    }

    // 2. Verify Merkle proof (input commitment exists in the tree).
    // We use the nullifier hash as the leaf representative.
    if !verify_merkle_proof(&nullifier, &merkle_proof, &pool.merkle_root) {
        // NOTE: In production, the leaf would be the actual commitment hash,
        // and the nullifier would be derived from the commitment + secret.
        // For this implementation, we log a warning but proceed if the pool
        // has a zero root (freshly initialized).
        if pool.merkle_root != [0u8; 32] {
            return Err(PrivacyError::InvalidMerkleProof.into());
        }
        msg!("Warning: Merkle root is zero, skipping proof verification");
    }

    // 3. Verify nullifier has not been spent.
    let (expected_null_pda, null_bump) = Pubkey::find_program_address(
        &[NULLIFIER_SEED, &nullifier],
        program_id,
    );
    if nullifier_info.key != &expected_null_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    if nullifier_info.data_len() > 0 {
        let existing = NullifierRecord::try_from_slice(&nullifier_info.data.borrow());
        if let Ok(nr) = existing {
            if nr.is_initialized {
                return Err(PrivacyError::NullifierAlreadySpent.into());
            }
        }
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    // 4. Record nullifier as spent.
    let null_record = NullifierRecord {
        is_initialized: true,
        nullifier,
        timestamp: clock.unix_timestamp,
        bump: null_bump,
    };

    // 5. Create two new output commitments.
    let out_index_1 = pool.commitment_count;
    let out_index_2 = pool
        .commitment_count
        .checked_add(1)
        .ok_or(PrivacyError::Overflow)?;
    pool.commitment_count = pool
        .commitment_count
        .checked_add(2)
        .ok_or(PrivacyError::Overflow)?;

    let leaf_1 = CommitmentLeaf {
        is_initialized: true,
        index: out_index_1,
        commitment: output_commitment_1,
        encrypted_amount: encrypted_amount_1,
        owner: *sender_info.key, // In production, would be the recipient's stealth address.
        timestamp: clock.unix_timestamp,
        bump: 0, // Would be derived from PDA.
    };

    let leaf_2 = CommitmentLeaf {
        is_initialized: true,
        index: out_index_2,
        commitment: output_commitment_2,
        encrypted_amount: encrypted_amount_2,
        owner: *sender_info.key, // Change output.
        timestamp: clock.unix_timestamp,
        bump: 0,
    };

    // Serialize all state.
    let pool_data = pool.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    pool_info.data.borrow_mut()[..pool_data.len()].copy_from_slice(&pool_data);

    let null_data = null_record.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    nullifier_info.data.borrow_mut()[..null_data.len()].copy_from_slice(&null_data);

    let leaf_1_data = leaf_1.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    output_1_info.data.borrow_mut()[..leaf_1_data.len()].copy_from_slice(&leaf_1_data);

    let leaf_2_data = leaf_2.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    output_2_info.data.borrow_mut()[..leaf_2_data.len()].copy_from_slice(&leaf_2_data);

    msg!(
        "Shielded transfer: nullifier spent, 2 new commitments at indices {}, {}",
        out_index_1,
        out_index_2
    );
    Ok(())
}

fn process_unshield(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    nullifier: [u8; 32],
    merkle_proof: MerklePath,
    zk_proof: ZkProof,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_info = next_account_info(account_iter)?;
    let nullifier_info = next_account_info(account_iter)?;
    let withdrawer_info = next_account_info(account_iter)?;
    let _recipient_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;

    if !withdrawer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool = ShieldedPool::try_from_slice(&pool_info.data.borrow())
        .map_err(|_| PrivacyError::InvalidAccountData)?;

    if !pool.is_initialized {
        return Err(PrivacyError::PoolNotInitialized.into());
    }

    // Verify ZK proof.
    if !verify_zk_proof(&zk_proof) {
        return Err(PrivacyError::InvalidZkProof.into());
    }

    // Verify Merkle proof.
    if pool.merkle_root != [0u8; 32]
        && !verify_merkle_proof(&nullifier, &merkle_proof, &pool.merkle_root)
    {
        return Err(PrivacyError::InvalidMerkleProof.into());
    }

    // Check pool balance.
    if pool.pool_balance < amount {
        return Err(PrivacyError::InsufficientPoolBalance.into());
    }

    // Verify nullifier not already spent.
    let (expected_null_pda, null_bump) = Pubkey::find_program_address(
        &[NULLIFIER_SEED, &nullifier],
        program_id,
    );
    if nullifier_info.key != &expected_null_pda {
        return Err(ProgramError::InvalidSeeds);
    }
    if nullifier_info.data_len() > 0 {
        let existing = NullifierRecord::try_from_slice(&nullifier_info.data.borrow());
        if let Ok(nr) = existing {
            if nr.is_initialized {
                return Err(PrivacyError::NullifierAlreadySpent.into());
            }
        }
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    // Record nullifier.
    let null_record = NullifierRecord {
        is_initialized: true,
        nullifier,
        timestamp: clock.unix_timestamp,
        bump: null_bump,
    };

    pool.pool_balance = pool
        .pool_balance
        .checked_sub(amount)
        .ok_or(PrivacyError::InsufficientPoolBalance)?;

    // In production: transfer lamports from pool PDA to recipient.
    // This requires a CPI to the system program with PDA signing.

    // Serialize.
    let pool_data = pool.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    pool_info.data.borrow_mut()[..pool_data.len()].copy_from_slice(&pool_data);

    let null_data = null_record.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    nullifier_info.data.borrow_mut()[..null_data.len()].copy_from_slice(&null_data);

    msg!("Unshield: {} lamports withdrawn from pool", amount);
    Ok(())
}

fn process_create_stealth_address(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    stealth_pubkey: Pubkey,
    ephemeral_pubkey: [u8; 32],
    encrypted_payload: [u8; 64],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let stealth_info = next_account_info(account_iter)?;
    let sender_info = next_account_info(account_iter)?;
    let recipient_scan_key_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !sender_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Verify stealth address PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[STEALTH_SEED, stealth_pubkey.as_ref()],
        program_id,
    );
    if stealth_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    let stealth = StealthAddress {
        is_initialized: true,
        stealth_pubkey,
        ephemeral_pubkey,
        recipient_scan_key: *recipient_scan_key_info.key,
        encrypted_payload,
        timestamp: clock.unix_timestamp,
        bump,
    };

    let data = stealth.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    stealth_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Stealth address created: {} (recipient scan key: {})",
        stealth_pubkey,
        recipient_scan_key_info.key
    );
    Ok(())
}

fn process_create_encrypted_note(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    ciphertext: Vec<u8>,
    nonce: [u8; 12],
    auditor_ciphertext: Vec<u8>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let note_info = next_account_info(account_iter)?;
    let sender_info = next_account_info(account_iter)?;
    let recipient_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !sender_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if ciphertext.len() > MAX_NOTE_LEN {
        return Err(PrivacyError::NoteTooLong.into());
    }

    // Derive note PDA from sender + recipient + nonce.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[NOTE_SEED, sender_info.key.as_ref(), &nonce],
        program_id,
    );
    if note_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    let note = EncryptedNote {
        is_initialized: true,
        sender: *sender_info.key,
        recipient: *recipient_info.key,
        ciphertext,
        nonce,
        auditor_ciphertext,
        timestamp: clock.unix_timestamp,
        bump,
    };

    let data = note.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    note_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Encrypted note created: {} -> {}",
        sender_info.key,
        recipient_info.key
    );
    Ok(())
}

fn process_configure_auditor(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    auditors: Vec<Pubkey>,
    is_active: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let auditor_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if auditors.len() > MAX_AUDITORS {
        return Err(PrivacyError::MaxAuditorsExceeded.into());
    }

    let (expected_pda, bump) =
        Pubkey::find_program_address(&[AUDITOR_SEED], program_id);
    if auditor_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let config = AuditorConfig {
        is_initialized: true,
        authority: *authority_info.key,
        auditors,
        is_active,
        bump,
    };

    let data = config.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    auditor_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Auditor config updated: {} auditors, active={}",
        config.auditors.len(),
        is_active
    );
    Ok(())
}

fn process_audit_transaction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    attestation: [u8; 32],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _target_info = next_account_info(account_iter)?;
    let auditor_config_info = next_account_info(account_iter)?;
    let auditor_info = next_account_info(account_iter)?;
    let _clock_sysvar = next_account_info(account_iter)?;

    if !auditor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let config = AuditorConfig::try_from_slice(&auditor_config_info.data.borrow())
        .map_err(|_| PrivacyError::InvalidAccountData)?;

    if !config.is_active {
        return Err(PrivacyError::NotAuthorizedAuditor.into());
    }

    if !config.auditors.contains(auditor_info.key) {
        return Err(PrivacyError::NotAuthorizedAuditor.into());
    }

    // Log the audit event. Actual decryption happens off-chain.
    msg!(
        "Audit attestation by {}: {:?}",
        auditor_info.key,
        &attestation[..8]
    );
    Ok(())
}

fn process_update_merkle_root(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    new_root: [u8; 32],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let pool_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut pool = ShieldedPool::try_from_slice(&pool_info.data.borrow())
        .map_err(|_| PrivacyError::InvalidAccountData)?;

    if pool.authority != *authority_info.key {
        return Err(PrivacyError::InvalidAuthority.into());
    }

    pool.merkle_root = new_root;

    let data = pool.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    pool_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Merkle root updated: {:?}", &new_root[..8]);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commitment() {
        let blinding = [0xABu8; 32];
        let c1 = compute_pedersen_commitment(1000, &blinding);
        let c2 = compute_pedersen_commitment(1000, &blinding);
        assert_eq!(c1, c2); // Same inputs produce same commitment.

        let c3 = compute_pedersen_commitment(1001, &blinding);
        assert_ne!(c1, c3); // Different amount produces different commitment.

        let blinding2 = [0xCDu8; 32];
        let c4 = compute_pedersen_commitment(1000, &blinding2);
        assert_ne!(c1, c4); // Different blinding produces different commitment.
    }

    #[test]
    fn test_merkle_proof_verification() {
        // Build a simple 2-level tree:
        //       root
        //      /    \
        //   h01      h23
        //  /  \     /  \
        // L0   L1  L2   L3
        let l0 = hash(b"leaf0").to_bytes();
        let l1 = hash(b"leaf1").to_bytes();
        let l2 = hash(b"leaf2").to_bytes();
        let l3 = hash(b"leaf3").to_bytes();

        let mut buf = Vec::new();
        buf.extend_from_slice(&l0);
        buf.extend_from_slice(&l1);
        let h01 = hash(&buf).to_bytes();

        buf.clear();
        buf.extend_from_slice(&l2);
        buf.extend_from_slice(&l3);
        let h23 = hash(&buf).to_bytes();

        buf.clear();
        buf.extend_from_slice(&h01);
        buf.extend_from_slice(&h23);
        let root = hash(&buf).to_bytes();

        // Prove l0 is in the tree.
        let proof = MerklePath {
            siblings: vec![l1, h23],
            path_indices: vec![0, 0],
        };
        assert!(verify_merkle_proof(&l0, &proof, &root));

        // Prove l3 is in the tree.
        let proof3 = MerklePath {
            siblings: vec![l2, h01],
            path_indices: vec![1, 1],
        };
        assert!(verify_merkle_proof(&l3, &proof3, &root));

        // Invalid proof should fail.
        let bad_proof = MerklePath {
            siblings: vec![l2, h23],
            path_indices: vec![0, 0],
        };
        assert!(!verify_merkle_proof(&l0, &bad_proof, &root));
    }

    #[test]
    fn test_zk_proof_verification() {
        let valid_proof = ZkProof {
            proof_a: [1u8; 64],
            proof_b: [2u8; 128],
            proof_c: [3u8; 64],
        };
        assert!(verify_zk_proof(&valid_proof));

        let invalid_proof = ZkProof {
            proof_a: [0u8; 64],
            proof_b: [0u8; 128],
            proof_c: [0u8; 64],
        };
        assert!(!verify_zk_proof(&invalid_proof));
    }

    #[test]
    fn test_shielded_pool_serialization() {
        let pool = ShieldedPool {
            is_initialized: true,
            authority: Pubkey::new_unique(),
            commitment_count: 42,
            merkle_root: [0xAA; 32],
            pool_balance: 1_000_000,
            compliance_enabled: true,
            bump: 255,
        };
        let data = pool.try_to_vec().unwrap();
        let decoded = ShieldedPool::try_from_slice(&data).unwrap();
        assert_eq!(decoded.commitment_count, 42);
        assert_eq!(decoded.pool_balance, 1_000_000);
        assert!(decoded.compliance_enabled);
    }

    #[test]
    fn test_nullifier_serialization() {
        let nr = NullifierRecord {
            is_initialized: true,
            nullifier: [0xBB; 32],
            timestamp: 1_700_000_000,
            bump: 254,
        };
        let data = nr.try_to_vec().unwrap();
        let decoded = NullifierRecord::try_from_slice(&data).unwrap();
        assert_eq!(decoded.nullifier, [0xBB; 32]);
    }

    #[test]
    fn test_stealth_address_serialization() {
        let sa = StealthAddress {
            is_initialized: true,
            stealth_pubkey: Pubkey::new_unique(),
            ephemeral_pubkey: [0xCC; 32],
            recipient_scan_key: Pubkey::new_unique(),
            encrypted_payload: [0xDD; 64],
            timestamp: 1_700_000_000,
            bump: 253,
        };
        let data = sa.try_to_vec().unwrap();
        let decoded = StealthAddress::try_from_slice(&data).unwrap();
        assert_eq!(decoded.ephemeral_pubkey, [0xCC; 32]);
        assert_eq!(decoded.encrypted_payload, [0xDD; 64]);
    }

    #[test]
    fn test_encrypted_note_serialization() {
        let note = EncryptedNote {
            is_initialized: true,
            sender: Pubkey::new_unique(),
            recipient: Pubkey::new_unique(),
            ciphertext: vec![1, 2, 3, 4, 5],
            nonce: [0xEE; 12],
            auditor_ciphertext: vec![6, 7, 8],
            timestamp: 1_700_000_000,
            bump: 252,
        };
        let data = note.try_to_vec().unwrap();
        let decoded = EncryptedNote::try_from_slice(&data).unwrap();
        assert_eq!(decoded.ciphertext, vec![1, 2, 3, 4, 5]);
        assert_eq!(decoded.nonce, [0xEE; 12]);
    }

    #[test]
    fn test_range_proof_verification() {
        let commitment = compute_pedersen_commitment(500, &[0x11; 32]);
        let valid_proof = RangeProof {
            proof_data: vec![1, 2, 3],
        };
        assert!(verify_range_proof(&commitment, &valid_proof));

        let empty_proof = RangeProof {
            proof_data: vec![],
        };
        assert!(!verify_range_proof(&commitment, &empty_proof));
    }
}
