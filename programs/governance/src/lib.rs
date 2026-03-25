use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
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

/// Minimum PRISM stake (in lamports) required to create a proposal.
/// 100_000 PRISM with 9 decimals.
pub const MIN_PROPOSAL_STAKE: u64 = 100_000_000_000_000;

/// Quorum: 10 % of staked supply must vote for a proposal to be valid.
pub const QUORUM_BPS: u16 = 1_000; // basis points (1000 = 10%)

/// Standard voting period: 7 days in seconds.
pub const VOTING_PERIOD_SECS: i64 = 7 * 24 * 60 * 60;

/// Emergency fast-track voting period: 24 hours.
pub const EMERGENCY_VOTING_PERIOD_SECS: i64 = 24 * 60 * 60;

/// Time-lock after a vote passes before it can be executed: 48 hours.
pub const EXECUTION_DELAY_SECS: i64 = 48 * 60 * 60;

/// Council veto window: 48 hours after voting ends.
pub const VETO_WINDOW_SECS: i64 = 48 * 60 * 60;

/// Emergency proposals require a 2/3 super-majority.
pub const EMERGENCY_SUPERMAJORITY_BPS: u16 = 6_667; // 66.67%

/// Maximum description length in bytes.
pub const MAX_DESCRIPTION_LEN: usize = 512;

/// Maximum number of council members.
pub const MAX_COUNCIL_MEMBERS: usize = 11;

/// Seed prefix for governance state PDA.
pub const GOVERNANCE_SEED: &[u8] = b"governance";

/// Seed prefix for proposal PDA.
pub const PROPOSAL_SEED: &[u8] = b"proposal";

/// Seed prefix for vote record PDA.
pub const VOTE_RECORD_SEED: &[u8] = b"vote_record";

/// Seed prefix for delegation PDA.
pub const DELEGATION_SEED: &[u8] = b"delegation";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Error, Debug, Clone)]
pub enum GovernanceError {
    #[error("Insufficient stake to create a proposal (requires 100K PRISM)")]
    InsufficientStake,
    #[error("Proposal is not in the Voting state")]
    ProposalNotVoting,
    #[error("Voting period has ended")]
    VotingPeriodEnded,
    #[error("Voting period has not ended yet")]
    VotingPeriodNotEnded,
    #[error("Quorum was not reached")]
    QuorumNotReached,
    #[error("Proposal did not pass")]
    ProposalNotPassed,
    #[error("Execution time-lock has not elapsed")]
    TimeLockNotElapsed,
    #[error("Proposal was vetoed by the council")]
    ProposalVetoed,
    #[error("Not a council member")]
    NotCouncilMember,
    #[error("Veto window has expired")]
    VetoWindowExpired,
    #[error("Emergency proposal requires 2/3 supermajority")]
    SupermajorityNotReached,
    #[error("Already voted on this proposal")]
    AlreadyVoted,
    #[error("Invalid proposal type")]
    InvalidProposalType,
    #[error("Description too long")]
    DescriptionTooLong,
    #[error("Cannot delegate to self")]
    SelfDelegation,
    #[error("Proposal already finalized")]
    ProposalAlreadyFinalized,
    #[error("Governance already initialized")]
    AlreadyInitialized,
    #[error("Invalid authority")]
    InvalidAuthority,
    #[error("Arithmetic overflow")]
    Overflow,
    #[error("Invalid account data")]
    InvalidAccountData,
    #[error("Account not owned by this program")]
    InvalidOwner,
}

impl From<GovernanceError> for ProgramError {
    fn from(e: GovernanceError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ProposalType {
    /// Change a protocol parameter (key, new value).
    ParameterChange { key: [u8; 32], value: [u8; 32] },
    /// Spend from the treasury.
    TreasurySpend { recipient: Pubkey, amount: u64 },
    /// Upgrade an on-chain program.
    ProgramUpgrade { program_id: Pubkey, buffer: Pubkey },
    /// Emergency action (fast-track, requires supermajority).
    Emergency { payload: [u8; 64] },
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum ProposalStatus {
    /// Proposal is open for voting.
    Voting,
    /// Voting concluded, proposal passed. Awaiting time-lock expiry.
    Passed,
    /// Voting concluded, proposal was defeated.
    Defeated,
    /// Council vetoed the proposal within the veto window.
    Vetoed,
    /// Proposal was executed on-chain.
    Executed,
    /// Proposal was cancelled by the proposer.
    Cancelled,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy, PartialEq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

/// Top-level governance configuration. One per realm (PDA).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct GovernanceState {
    /// Has this account been initialized?
    pub is_initialized: bool,
    /// The authority that can update governance parameters.
    pub authority: Pubkey,
    /// Running proposal counter (used to derive proposal PDAs).
    pub proposal_count: u64,
    /// Total staked supply snapshot used for quorum calculation.
    pub total_staked_supply: u64,
    /// Council members who can veto proposals.
    pub council: Vec<Pubkey>,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl GovernanceState {
    pub const MAX_SIZE: usize =
        1 + 32 + 8 + 8 + 4 + (32 * MAX_COUNCIL_MEMBERS) + 1;
}

/// A single governance proposal.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Proposal {
    /// Has this account been initialized?
    pub is_initialized: bool,
    /// Sequential proposal id.
    pub id: u64,
    /// Proposer's public key.
    pub proposer: Pubkey,
    /// Human-readable description (max 512 bytes).
    pub description: Vec<u8>,
    /// Type-specific payload.
    pub proposal_type: ProposalType,
    /// Current status.
    pub status: ProposalStatus,
    /// Unix timestamp when voting started.
    pub voting_start: i64,
    /// Unix timestamp when voting ends.
    pub voting_end: i64,
    /// Weighted votes *for*.
    pub votes_for: u64,
    /// Weighted votes *against*.
    pub votes_against: u64,
    /// Weighted votes *abstain*.
    pub votes_abstain: u64,
    /// Whether this is an emergency (fast-track) proposal.
    pub is_emergency: bool,
    /// Unix timestamp when execution becomes available (after time-lock).
    pub execution_time: i64,
    /// Bump seed for the PDA.
    pub bump: u8,
}

impl Proposal {
    pub const MAX_SIZE: usize =
        1 + 8 + 32 + (4 + MAX_DESCRIPTION_LEN) + 128 + 1 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 1;
}

/// Per-voter record for a specific proposal (prevents double-voting).
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VoteRecord {
    pub is_initialized: bool,
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub choice: VoteChoice,
    pub weight: u64,
    pub timestamp: i64,
    pub bump: u8,
}

impl VoteRecord {
    pub const MAX_SIZE: usize = 1 + 8 + 32 + 1 + 8 + 8 + 1;
}

/// Vote delegation: delegator → delegate.
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Delegation {
    pub is_initialized: bool,
    pub delegator: Pubkey,
    pub delegate: Pubkey,
    pub weight: u64,
    pub timestamp: i64,
    pub bump: u8,
}

impl Delegation {
    pub const MAX_SIZE: usize = 1 + 32 + 32 + 8 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum GovernanceInstruction {
    /// Initialize the governance realm.
    ///
    /// Accounts:
    ///   0. `[writable]`  Governance state PDA
    ///   1. `[signer]`    Authority
    ///   2. `[]`          System program
    ///   3. `[]`          Rent sysvar
    InitializeGovernance {
        council: Vec<Pubkey>,
        total_staked_supply: u64,
    },

    /// Create a new proposal.
    ///
    /// Accounts:
    ///   0. `[writable]`  Governance state PDA
    ///   1. `[writable]`  Proposal PDA (derived from proposal_count)
    ///   2. `[signer]`    Proposer
    ///   3. `[]`          Proposer's stake account (to verify min stake)
    ///   4. `[]`          Clock sysvar
    ///   5. `[]`          System program
    ///   6. `[]`          Rent sysvar
    CreateProposal {
        description: Vec<u8>,
        proposal_type: ProposalType,
        is_emergency: bool,
    },

    /// Cast a vote on an active proposal.
    ///
    /// Accounts:
    ///   0. `[writable]`  Proposal PDA
    ///   1. `[writable]`  Vote record PDA
    ///   2. `[signer]`    Voter
    ///   3. `[]`          Voter's stake account (for weight)
    ///   4. `[]`          Clock sysvar
    ///   5. `[]`          System program
    ///   6. `[]`          Rent sysvar
    CastVote {
        choice: VoteChoice,
        weight: u64,
    },

    /// Delegate voting power to another address.
    ///
    /// Accounts:
    ///   0. `[writable]`  Delegation PDA
    ///   1. `[signer]`    Delegator
    ///   2. `[]`          Delegate
    ///   3. `[]`          Delegator's stake account (for weight)
    ///   4. `[]`          Clock sysvar
    ///   5. `[]`          System program
    ///   6. `[]`          Rent sysvar
    DelegateVote {
        delegate: Pubkey,
        weight: u64,
    },

    /// Revoke a previously delegated vote.
    ///
    /// Accounts:
    ///   0. `[writable]`  Delegation PDA
    ///   1. `[signer]`    Delegator
    RevokeDelegation,

    /// Finalize voting on a proposal once the voting period has ended.
    ///
    /// Accounts:
    ///   0. `[writable]`  Proposal PDA
    ///   1. `[]`          Governance state PDA
    ///   2. `[]`          Clock sysvar
    FinalizeVoting,

    /// Execute a passed proposal after the time-lock has elapsed.
    ///
    /// Accounts:
    ///   0. `[writable]`  Proposal PDA
    ///   1. `[]`          Governance state PDA
    ///   2. `[signer]`    Executor (anyone can execute)
    ///   3. `[]`          Clock sysvar
    ///   (+ additional accounts depending on proposal type)
    ExecuteProposal,

    /// Council member vetoes a proposal within the veto window.
    ///
    /// Accounts:
    ///   0. `[writable]`  Proposal PDA
    ///   1. `[]`          Governance state PDA
    ///   2. `[signer]`    Council member
    ///   3. `[]`          Clock sysvar
    VetoProposal,

    /// Proposer cancels their own proposal (only while still in Voting state).
    ///
    /// Accounts:
    ///   0. `[writable]`  Proposal PDA
    ///   1. `[signer]`    Proposer
    CancelProposal,

    /// Authority updates the staked supply snapshot (called periodically).
    ///
    /// Accounts:
    ///   0. `[writable]`  Governance state PDA
    ///   1. `[signer]`    Authority
    UpdateStakedSupply {
        total_staked_supply: u64,
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
    let instruction = GovernanceInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    match instruction {
        GovernanceInstruction::InitializeGovernance {
            council,
            total_staked_supply,
        } => process_initialize_governance(program_id, accounts, council, total_staked_supply),

        GovernanceInstruction::CreateProposal {
            description,
            proposal_type,
            is_emergency,
        } => process_create_proposal(program_id, accounts, description, proposal_type, is_emergency),

        GovernanceInstruction::CastVote { choice, weight } => {
            process_cast_vote(program_id, accounts, choice, weight)
        }

        GovernanceInstruction::DelegateVote { delegate, weight } => {
            process_delegate_vote(program_id, accounts, delegate, weight)
        }

        GovernanceInstruction::RevokeDelegation => process_revoke_delegation(program_id, accounts),

        GovernanceInstruction::FinalizeVoting => process_finalize_voting(program_id, accounts),

        GovernanceInstruction::ExecuteProposal => process_execute_proposal(program_id, accounts),

        GovernanceInstruction::VetoProposal => process_veto_proposal(program_id, accounts),

        GovernanceInstruction::CancelProposal => process_cancel_proposal(program_id, accounts),

        GovernanceInstruction::UpdateStakedSupply {
            total_staked_supply,
        } => process_update_staked_supply(program_id, accounts, total_staked_supply),
    }
}

// ---------------------------------------------------------------------------
// Processors
// ---------------------------------------------------------------------------

fn process_initialize_governance(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    council: Vec<Pubkey>,
    total_staked_supply: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let rent_sysvar = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let rent = Rent::from_account_info(rent_sysvar)?;

    // Verify PDA
    let (expected_pda, bump) =
        Pubkey::find_program_address(&[GOVERNANCE_SEED], program_id);
    if governance_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    // Verify not already initialized
    if governance_info.data_len() > 0 {
        let existing = GovernanceState::try_from_slice(&governance_info.data.borrow());
        if let Ok(state) = existing {
            if state.is_initialized {
                return Err(GovernanceError::AlreadyInitialized.into());
            }
        }
    }

    if council.len() > MAX_COUNCIL_MEMBERS {
        return Err(ProgramError::InvalidArgument);
    }

    let state = GovernanceState {
        is_initialized: true,
        authority: *authority_info.key,
        proposal_count: 0,
        total_staked_supply,
        council,
        bump,
    };

    let data = state.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    if !rent.is_exempt(governance_info.lamports(), data.len()) {
        msg!("Governance account is not rent-exempt");
        return Err(ProgramError::AccountNotRentExempt);
    }

    governance_info
        .data
        .borrow_mut()
        .copy_from_slice(&data);

    msg!("Governance initialized with {} council members", state.council.len());
    Ok(())
}

fn process_create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    description: Vec<u8>,
    proposal_type: ProposalType,
    is_emergency: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;
    let proposer_info = next_account_info(account_iter)?;
    let stake_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !proposer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if description.len() > MAX_DESCRIPTION_LEN {
        return Err(GovernanceError::DescriptionTooLong.into());
    }

    // Verify proposer has sufficient stake.
    // The stake account balance is used as a proxy for staked amount.
    if stake_info.lamports() < MIN_PROPOSAL_STAKE {
        return Err(GovernanceError::InsufficientStake.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Read and update governance state (increment proposal counter).
    let mut gov_state = GovernanceState::try_from_slice(&governance_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;
    let proposal_id = gov_state.proposal_count;
    gov_state.proposal_count = gov_state
        .proposal_count
        .checked_add(1)
        .ok_or(GovernanceError::Overflow)?;

    // Determine voting period.
    let voting_period = if is_emergency {
        EMERGENCY_VOTING_PERIOD_SECS
    } else {
        VOTING_PERIOD_SECS
    };

    let voting_end = now
        .checked_add(voting_period)
        .ok_or(GovernanceError::Overflow)?;
    let execution_time = voting_end
        .checked_add(EXECUTION_DELAY_SECS)
        .ok_or(GovernanceError::Overflow)?;

    // Verify proposal PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[PROPOSAL_SEED, &proposal_id.to_le_bytes()],
        program_id,
    );
    if proposal_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let proposal = Proposal {
        is_initialized: true,
        id: proposal_id,
        proposer: *proposer_info.key,
        description,
        proposal_type,
        status: ProposalStatus::Voting,
        voting_start: now,
        voting_end,
        votes_for: 0,
        votes_against: 0,
        votes_abstain: 0,
        is_emergency,
        execution_time,
        bump,
    };

    // Serialize both accounts.
    let gov_data = gov_state.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    governance_info.data.borrow_mut()[..gov_data.len()].copy_from_slice(&gov_data);

    let prop_data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..prop_data.len()].copy_from_slice(&prop_data);

    msg!(
        "Proposal #{} created by {} (emergency={})",
        proposal_id,
        proposer_info.key,
        is_emergency
    );
    Ok(())
}

fn process_cast_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    choice: VoteChoice,
    weight: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_iter)?;
    let vote_record_info = next_account_info(account_iter)?;
    let voter_info = next_account_info(account_iter)?;
    let _stake_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !voter_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    // Load proposal.
    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if proposal.status != ProposalStatus::Voting {
        return Err(GovernanceError::ProposalNotVoting.into());
    }
    if now > proposal.voting_end {
        return Err(GovernanceError::VotingPeriodEnded.into());
    }

    // Verify vote record PDA and ensure no double vote.
    let (expected_vr, bump) = Pubkey::find_program_address(
        &[
            VOTE_RECORD_SEED,
            &proposal.id.to_le_bytes(),
            voter_info.key.as_ref(),
        ],
        program_id,
    );
    if vote_record_info.key != &expected_vr {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check if already voted (account has data and is initialized).
    if vote_record_info.data_len() > 0 {
        let existing = VoteRecord::try_from_slice(&vote_record_info.data.borrow());
        if let Ok(vr) = existing {
            if vr.is_initialized {
                return Err(GovernanceError::AlreadyVoted.into());
            }
        }
    }

    // Tally the vote.
    match choice {
        VoteChoice::For => {
            proposal.votes_for = proposal
                .votes_for
                .checked_add(weight)
                .ok_or(GovernanceError::Overflow)?;
        }
        VoteChoice::Against => {
            proposal.votes_against = proposal
                .votes_against
                .checked_add(weight)
                .ok_or(GovernanceError::Overflow)?;
        }
        VoteChoice::Abstain => {
            proposal.votes_abstain = proposal
                .votes_abstain
                .checked_add(weight)
                .ok_or(GovernanceError::Overflow)?;
        }
    }

    // Write vote record.
    let vote_record = VoteRecord {
        is_initialized: true,
        proposal_id: proposal.id,
        voter: *voter_info.key,
        choice,
        weight,
        timestamp: now,
        bump,
    };

    let vr_data = vote_record.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    vote_record_info.data.borrow_mut()[..vr_data.len()].copy_from_slice(&vr_data);

    // Write updated proposal.
    let prop_data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..prop_data.len()].copy_from_slice(&prop_data);

    msg!(
        "Vote cast on proposal #{}: {:?} with weight {}",
        proposal.id,
        choice,
        weight
    );
    Ok(())
}

fn process_delegate_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    delegate: Pubkey,
    weight: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let delegation_info = next_account_info(account_iter)?;
    let delegator_info = next_account_info(account_iter)?;
    let delegate_info = next_account_info(account_iter)?;
    let _stake_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;
    let _system_program = next_account_info(account_iter)?;
    let _rent_sysvar = next_account_info(account_iter)?;

    if !delegator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if delegator_info.key == delegate_info.key {
        return Err(GovernanceError::SelfDelegation.into());
    }

    let clock = Clock::from_account_info(clock_sysvar)?;

    // Verify delegation PDA.
    let (expected_pda, bump) = Pubkey::find_program_address(
        &[DELEGATION_SEED, delegator_info.key.as_ref()],
        program_id,
    );
    if delegation_info.key != &expected_pda {
        return Err(ProgramError::InvalidSeeds);
    }

    let delegation = Delegation {
        is_initialized: true,
        delegator: *delegator_info.key,
        delegate,
        weight,
        timestamp: clock.unix_timestamp,
        bump,
    };

    let data = delegation.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    delegation_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Delegation: {} -> {} (weight: {})",
        delegator_info.key,
        delegate,
        weight
    );
    Ok(())
}

fn process_revoke_delegation(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let delegation_info = next_account_info(account_iter)?;
    let delegator_info = next_account_info(account_iter)?;

    if !delegator_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut delegation = Delegation::try_from_slice(&delegation_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if delegation.delegator != *delegator_info.key {
        return Err(GovernanceError::InvalidAuthority.into());
    }

    delegation.is_initialized = false;
    delegation.weight = 0;

    let data = delegation.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    delegation_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Delegation revoked by {}", delegator_info.key);
    Ok(())
}

fn process_finalize_voting(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_iter)?;
    let governance_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if proposal.status != ProposalStatus::Voting {
        return Err(GovernanceError::ProposalNotVoting.into());
    }
    if now < proposal.voting_end {
        return Err(GovernanceError::VotingPeriodNotEnded.into());
    }

    let gov_state = GovernanceState::try_from_slice(&governance_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    // Calculate total votes cast (for + against + abstain).
    let total_votes = proposal
        .votes_for
        .checked_add(proposal.votes_against)
        .and_then(|v| v.checked_add(proposal.votes_abstain))
        .ok_or(GovernanceError::Overflow)?;

    // Quorum check: total votes must be >= QUORUM_BPS of staked supply.
    let quorum_threshold = gov_state
        .total_staked_supply
        .checked_mul(QUORUM_BPS as u64)
        .and_then(|v| v.checked_div(10_000))
        .ok_or(GovernanceError::Overflow)?;

    if total_votes < quorum_threshold {
        proposal.status = ProposalStatus::Defeated;
        msg!(
            "Proposal #{} defeated: quorum not reached ({} / {} required)",
            proposal.id,
            total_votes,
            quorum_threshold
        );
    } else if proposal.is_emergency {
        // Emergency proposals need 2/3 supermajority of (for + against).
        let decisive_votes = proposal
            .votes_for
            .checked_add(proposal.votes_against)
            .ok_or(GovernanceError::Overflow)?;
        let supermajority_threshold = decisive_votes
            .checked_mul(EMERGENCY_SUPERMAJORITY_BPS as u64)
            .and_then(|v| v.checked_div(10_000))
            .ok_or(GovernanceError::Overflow)?;

        if proposal.votes_for >= supermajority_threshold {
            proposal.status = ProposalStatus::Passed;
            msg!("Emergency proposal #{} passed with supermajority", proposal.id);
        } else {
            proposal.status = ProposalStatus::Defeated;
            msg!(
                "Emergency proposal #{} defeated: supermajority not reached",
                proposal.id
            );
        }
    } else {
        // Standard majority: for > against.
        if proposal.votes_for > proposal.votes_against {
            proposal.status = ProposalStatus::Passed;
            msg!("Proposal #{} passed", proposal.id);
        } else {
            proposal.status = ProposalStatus::Defeated;
            msg!("Proposal #{} defeated", proposal.id);
        }
    }

    let data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    Ok(())
}

fn process_execute_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_iter)?;
    let _governance_info = next_account_info(account_iter)?;
    let executor_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !executor_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if proposal.status != ProposalStatus::Passed {
        return Err(GovernanceError::ProposalNotPassed.into());
    }

    if now < proposal.execution_time {
        return Err(GovernanceError::TimeLockNotElapsed.into());
    }

    // Execute based on proposal type.
    match &proposal.proposal_type {
        ProposalType::ParameterChange { key, value } => {
            msg!(
                "Executing parameter change: key={:?}, value={:?}",
                &key[..8],
                &value[..8]
            );
            // In production: invoke the target program to apply the parameter change.
        }
        ProposalType::TreasurySpend { recipient, amount } => {
            msg!(
                "Executing treasury spend: {} lamports to {}",
                amount,
                recipient
            );
            // In production: transfer from treasury PDA to recipient.
        }
        ProposalType::ProgramUpgrade { program_id, buffer } => {
            msg!(
                "Executing program upgrade: program={}, buffer={}",
                program_id,
                buffer
            );
            // In production: invoke BPF loader to upgrade the program.
        }
        ProposalType::Emergency { payload } => {
            msg!("Executing emergency action: payload={:?}", &payload[..8]);
        }
    }

    proposal.status = ProposalStatus::Executed;

    let data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Proposal #{} executed", proposal.id);
    Ok(())
}

fn process_veto_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_iter)?;
    let governance_info = next_account_info(account_iter)?;
    let council_member_info = next_account_info(account_iter)?;
    let clock_sysvar = next_account_info(account_iter)?;

    if !council_member_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let clock = Clock::from_account_info(clock_sysvar)?;
    let now = clock.unix_timestamp;

    let gov_state = GovernanceState::try_from_slice(&governance_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    // Verify the signer is a council member.
    if !gov_state.council.contains(council_member_info.key) {
        return Err(GovernanceError::NotCouncilMember.into());
    }

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    // Veto is only allowed on Passed proposals within the veto window.
    if proposal.status != ProposalStatus::Passed {
        return Err(GovernanceError::ProposalNotPassed.into());
    }

    let veto_deadline = proposal
        .voting_end
        .checked_add(VETO_WINDOW_SECS)
        .ok_or(GovernanceError::Overflow)?;

    if now > veto_deadline {
        return Err(GovernanceError::VetoWindowExpired.into());
    }

    proposal.status = ProposalStatus::Vetoed;

    let data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!(
        "Proposal #{} vetoed by council member {}",
        proposal.id,
        council_member_info.key
    );
    Ok(())
}

fn process_cancel_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposal_info = next_account_info(account_iter)?;
    let proposer_info = next_account_info(account_iter)?;

    if !proposer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if proposal.proposer != *proposer_info.key {
        return Err(GovernanceError::InvalidAuthority.into());
    }

    if proposal.status != ProposalStatus::Voting {
        return Err(GovernanceError::ProposalAlreadyFinalized.into());
    }

    proposal.status = ProposalStatus::Cancelled;

    let data = proposal.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    proposal_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Proposal #{} cancelled by proposer", proposal.id);
    Ok(())
}

fn process_update_staked_supply(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    total_staked_supply: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let governance_info = next_account_info(account_iter)?;
    let authority_info = next_account_info(account_iter)?;

    if !authority_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut gov_state = GovernanceState::try_from_slice(&governance_info.data.borrow())
        .map_err(|_| GovernanceError::InvalidAccountData)?;

    if gov_state.authority != *authority_info.key {
        return Err(GovernanceError::InvalidAuthority.into());
    }

    gov_state.total_staked_supply = total_staked_supply;

    let data = gov_state.try_to_vec().map_err(|_| ProgramError::InvalidAccountData)?;
    governance_info.data.borrow_mut()[..data.len()].copy_from_slice(&data);

    msg!("Staked supply updated to {}", total_staked_supply);
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_state_serialization() {
        let state = GovernanceState {
            is_initialized: true,
            authority: Pubkey::new_unique(),
            proposal_count: 42,
            total_staked_supply: 1_000_000_000_000,
            council: vec![Pubkey::new_unique(), Pubkey::new_unique()],
            bump: 255,
        };
        let data = state.try_to_vec().unwrap();
        let decoded = GovernanceState::try_from_slice(&data).unwrap();
        assert_eq!(decoded.is_initialized, true);
        assert_eq!(decoded.proposal_count, 42);
        assert_eq!(decoded.total_staked_supply, 1_000_000_000_000);
        assert_eq!(decoded.council.len(), 2);
    }

    #[test]
    fn test_proposal_serialization() {
        let proposal = Proposal {
            is_initialized: true,
            id: 7,
            proposer: Pubkey::new_unique(),
            description: b"Test proposal".to_vec(),
            proposal_type: ProposalType::TreasurySpend {
                recipient: Pubkey::new_unique(),
                amount: 500_000,
            },
            status: ProposalStatus::Voting,
            voting_start: 1_700_000_000,
            voting_end: 1_700_604_800,
            votes_for: 1_000,
            votes_against: 200,
            votes_abstain: 50,
            is_emergency: false,
            execution_time: 1_700_777_600,
            bump: 254,
        };
        let data = proposal.try_to_vec().unwrap();
        let decoded = Proposal::try_from_slice(&data).unwrap();
        assert_eq!(decoded.id, 7);
        assert_eq!(decoded.status, ProposalStatus::Voting);
        assert_eq!(decoded.votes_for, 1_000);
    }

    #[test]
    fn test_vote_record_serialization() {
        let vr = VoteRecord {
            is_initialized: true,
            proposal_id: 3,
            voter: Pubkey::new_unique(),
            choice: VoteChoice::For,
            weight: 500,
            timestamp: 1_700_000_000,
            bump: 253,
        };
        let data = vr.try_to_vec().unwrap();
        let decoded = VoteRecord::try_from_slice(&data).unwrap();
        assert_eq!(decoded.choice, VoteChoice::For);
        assert_eq!(decoded.weight, 500);
    }

    #[test]
    fn test_delegation_serialization() {
        let del = Delegation {
            is_initialized: true,
            delegator: Pubkey::new_unique(),
            delegate: Pubkey::new_unique(),
            weight: 10_000,
            timestamp: 1_700_000_000,
            bump: 252,
        };
        let data = del.try_to_vec().unwrap();
        let decoded = Delegation::try_from_slice(&data).unwrap();
        assert_eq!(decoded.weight, 10_000);
        assert!(decoded.is_initialized);
    }

    #[test]
    fn test_quorum_calculation() {
        let total_staked: u64 = 1_000_000_000;
        let quorum = total_staked
            .checked_mul(QUORUM_BPS as u64)
            .unwrap()
            .checked_div(10_000)
            .unwrap();
        // 10% of 1 billion = 100 million
        assert_eq!(quorum, 100_000_000);
    }

    #[test]
    fn test_emergency_supermajority() {
        let votes_for: u64 = 700;
        let votes_against: u64 = 300;
        let decisive = votes_for + votes_against;
        let threshold = decisive * EMERGENCY_SUPERMAJORITY_BPS as u64 / 10_000;
        // 66.67% of 1000 = 666
        assert_eq!(threshold, 666);
        assert!(votes_for >= threshold); // 700 >= 666
    }

    #[test]
    fn test_voting_period_constants() {
        assert_eq!(VOTING_PERIOD_SECS, 604_800); // 7 days
        assert_eq!(EMERGENCY_VOTING_PERIOD_SECS, 86_400); // 24 hours
        assert_eq!(EXECUTION_DELAY_SECS, 172_800); // 48 hours
        assert_eq!(VETO_WINDOW_SECS, 172_800); // 48 hours
    }
}
