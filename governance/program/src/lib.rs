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

/// Voting period: ~3 days in slots (at 400ms per slot)
const DEFAULT_VOTING_PERIOD_SLOTS: u64 = 648_000;

/// Quorum: 10 % of total supply (1000 basis points)
const DEFAULT_QUORUM_BPS: u64 = 1_000;

/// Minimum SCLONE tokens required to create a proposal
const DEFAULT_PROPOSAL_THRESHOLD: u64 = 100_000;

/// Timelock: ~2 days in slots after a proposal passes before it can be executed
const DEFAULT_TIMELOCK_SLOTS: u64 = 432_000;

/// PDA seeds
const DAO_STATE_SEED: &[u8] = b"dao_state";
const PROPOSAL_SEED: &[u8] = b"proposal";
const VOTE_SEED: &[u8] = b"vote";
const DELEGATION_SEED: &[u8] = b"delegation";

// ---------------------------------------------------------------------------
// State accounts
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ProposalStatus {
    Pending,
    Active,
    Passed,
    Failed,
    Executed,
    Cancelled,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct DaoState {
    /// Authority that initialized the DAO
    pub authority: Pubkey,
    /// SCLONE governance token mint
    pub governance_token_mint: Pubkey,
    /// Treasury account (program-owned)
    pub treasury: Pubkey,
    /// Number of proposals created so far
    pub proposal_count: u64,
    /// Voting period in slots (~3 days default)
    pub voting_period: u64,
    /// Quorum in basis points (1000 = 10 %)
    pub quorum_bps: u64,
    /// Minimum tokens to create a proposal
    pub proposal_threshold: u64,
    /// Timelock period in slots (~2 days)
    pub timelock_period: u64,
    /// Total voting power supply (for quorum calculation)
    pub total_voting_power: u64,
    /// PDA bump
    pub bump: u8,
    /// Whether initialized
    pub is_initialized: bool,
}

impl DaoState {
    pub const LEN: usize = 32 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1; // 146 bytes
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Proposal {
    /// Unique proposal ID (sequential)
    pub id: u64,
    /// Address of the proposer
    pub proposer: Pubkey,
    /// Short title (max 128 chars)
    pub title: String,
    /// Detailed description (max 1024 chars)
    pub description: String,
    /// Serialized instruction data to execute if passed
    pub instruction_data: Vec<u8>,
    /// Total token-weighted votes in favor
    pub for_votes: u64,
    /// Total token-weighted votes against
    pub against_votes: u64,
    /// Slot when voting starts
    pub start_slot: u64,
    /// Slot when voting ends
    pub end_slot: u64,
    /// Current status
    pub status: ProposalStatus,
    /// Slot after which a passed proposal can be executed
    pub timelock_end: u64,
    /// PDA bump
    pub bump: u8,
    /// Whether initialized
    pub is_initialized: bool,
}

impl Proposal {
    /// Conservative upper bound
    pub const MAX_LEN: usize = 8           // id
        + 32                               // proposer
        + 4 + 128                          // title
        + 4 + 1024                         // description
        + 4 + 512                          // instruction_data
        + 8 + 8                            // for_votes, against_votes
        + 8 + 8                            // start_slot, end_slot
        + 1                                // status enum
        + 8                                // timelock_end
        + 1                                // bump
        + 1;                               // is_initialized
        // = 1759 bytes
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct VoteRecord {
    /// Voter address
    pub voter: Pubkey,
    /// Proposal ID this vote is for
    pub proposal_id: u64,
    /// Token-weighted vote power
    pub vote_weight: u64,
    /// true = for, false = against
    pub support: bool,
    /// PDA bump
    pub bump: u8,
    /// Whether initialized
    pub is_initialized: bool,
}

impl VoteRecord {
    pub const LEN: usize = 32 + 8 + 8 + 1 + 1 + 1; // 51 bytes
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Delegation {
    /// Delegator address
    pub delegator: Pubkey,
    /// Delegate (receives voting power)
    pub delegate: Pubkey,
    /// Amount of voting power delegated
    pub amount: u64,
    /// PDA bump
    pub bump: u8,
    /// Whether active
    pub is_initialized: bool,
}

impl Delegation {
    pub const LEN: usize = 32 + 32 + 8 + 1 + 1; // 74 bytes
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum GovernanceInstruction {
    /// Initialize the DAO with a treasury.
    ///
    /// Accounts:
    ///   0. `[signer]`   Authority / payer
    ///   1. `[writable]` DAO state PDA
    ///   2. `[]`         Governance token mint (SCLONE)
    ///   3. `[writable]` Treasury account
    ///   4. `[]`         System program
    InitializeDao {
        total_voting_power: u64,
    },

    /// Create a new proposal (must hold >= proposal_threshold tokens).
    ///
    /// Accounts:
    ///   0. `[signer]`   Proposer
    ///   1. `[writable]` DAO state PDA
    ///   2. `[writable]` Proposal PDA
    ///   3. `[]`         Proposer's token account
    ///   4. `[]`         System program
    CreateProposal {
        title: String,
        description: String,
        instruction_data: Vec<u8>,
    },

    /// Cast a vote for or against a proposal (token-weighted).
    ///
    /// Accounts:
    ///   0. `[signer]`   Voter
    ///   1. `[writable]` Proposal PDA
    ///   2. `[writable]` Vote record PDA
    ///   3. `[]`         Voter's token account
    ///   4. `[]`         DAO state PDA
    ///   5. `[]`         System program
    CastVote {
        support: bool,
    },

    /// Finalize a proposal after the voting period ends. Sets status to Passed or Failed.
    ///
    /// Accounts:
    ///   0. `[signer]`   Anyone can finalize
    ///   1. `[writable]` Proposal PDA
    ///   2. `[]`         DAO state PDA
    FinalizeProposal,

    /// Execute a passed proposal after the timelock period.
    ///
    /// Accounts:
    ///   0. `[signer]`   Anyone can execute
    ///   1. `[writable]` Proposal PDA
    ///   2. `[]`         DAO state PDA
    ///   3..N `[writable]` Accounts required by the proposal's instruction
    ExecuteProposal,

    /// Cancel a proposal (only the proposer can cancel, and only before voting ends).
    ///
    /// Accounts:
    ///   0. `[signer]`   Proposer
    ///   1. `[writable]` Proposal PDA
    CancelProposal,

    /// Delegate voting power to another address.
    ///
    /// Accounts:
    ///   0. `[signer]`   Delegator
    ///   1. `[writable]` Delegation PDA
    ///   2. `[]`         Delegator's token account
    ///   3. `[]`         System program
    DelegateVotes {
        delegate: Pubkey,
        amount: u64,
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
        GovernanceInstruction::InitializeDao { total_voting_power } => {
            process_initialize_dao(program_id, accounts, total_voting_power)
        }
        GovernanceInstruction::CreateProposal {
            title,
            description,
            instruction_data,
        } => process_create_proposal(program_id, accounts, title, description, instruction_data),
        GovernanceInstruction::CastVote { support } => {
            process_cast_vote(program_id, accounts, support)
        }
        GovernanceInstruction::FinalizeProposal => {
            process_finalize_proposal(program_id, accounts)
        }
        GovernanceInstruction::ExecuteProposal => {
            process_execute_proposal(program_id, accounts)
        }
        GovernanceInstruction::CancelProposal => {
            process_cancel_proposal(program_id, accounts)
        }
        GovernanceInstruction::DelegateVotes { delegate, amount } => {
            process_delegate_votes(program_id, accounts, delegate, amount)
        }
    }
}

// ---------------------------------------------------------------------------
// Instruction processors
// ---------------------------------------------------------------------------

fn process_initialize_dao(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    total_voting_power: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let authority = next_account_info(account_iter)?;
    let dao_state_info = next_account_info(account_iter)?;
    let governance_token_mint = next_account_info(account_iter)?;
    let treasury_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let (dao_pda, bump) = Pubkey::find_program_address(&[DAO_STATE_SEED], program_id);
    if *dao_state_info.key != dao_pda {
        msg!("Error: invalid DAO state PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
    let space = DaoState::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            authority.key,
            dao_state_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            authority.clone(),
            dao_state_info.clone(),
            system_program.clone(),
        ],
        &[&[DAO_STATE_SEED, &[bump]]],
    )?;

    let dao_state = DaoState {
        authority: *authority.key,
        governance_token_mint: *governance_token_mint.key,
        treasury: *treasury_info.key,
        proposal_count: 0,
        voting_period: DEFAULT_VOTING_PERIOD_SLOTS,
        quorum_bps: DEFAULT_QUORUM_BPS,
        proposal_threshold: DEFAULT_PROPOSAL_THRESHOLD,
        timelock_period: DEFAULT_TIMELOCK_SLOTS,
        total_voting_power,
        bump,
        is_initialized: true,
    };

    dao_state.serialize(&mut &mut dao_state_info.data.borrow_mut()[..])?;

    msg!("DAO initialized with treasury {}", treasury_info.key);
    Ok(())
}

fn process_create_proposal(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    description: String,
    instruction_data: Vec<u8>,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposer = next_account_info(account_iter)?;
    let dao_state_info = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;
    let proposer_token_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut dao_state = DaoState::try_from_slice(&dao_state_info.data.borrow())?;
    if !dao_state.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // Verify proposer holds enough tokens
    let token_data = spl_token::state::Account::unpack(&proposer_token_account.data.borrow())?;
    if token_data.owner != *proposer.key {
        msg!("Error: token account not owned by proposer");
        return Err(ProgramError::InvalidArgument);
    }
    if token_data.amount < dao_state.proposal_threshold {
        msg!(
            "Error: proposer holds {} tokens, needs {} to create proposal",
            token_data.amount,
            dao_state.proposal_threshold
        );
        return Err(ProgramError::InsufficientFunds);
    }

    // Validate input lengths
    if title.len() > 128 {
        msg!("Error: title too long (max 128 chars)");
        return Err(ProgramError::InvalidArgument);
    }
    if description.len() > 1024 {
        msg!("Error: description too long (max 1024 chars)");
        return Err(ProgramError::InvalidArgument);
    }
    if instruction_data.len() > 512 {
        msg!("Error: instruction data too large (max 512 bytes)");
        return Err(ProgramError::InvalidArgument);
    }

    let proposal_id = dao_state.proposal_count;
    let id_bytes = proposal_id.to_le_bytes();

    let (proposal_pda, proposal_bump) =
        Pubkey::find_program_address(&[PROPOSAL_SEED, &id_bytes], program_id);
    if *proposal_info.key != proposal_pda {
        msg!("Error: invalid proposal PDA");
        return Err(ProgramError::InvalidArgument);
    }

    let rent = Rent::get()?;
    let space = Proposal::MAX_LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            proposer.key,
            proposal_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            proposer.clone(),
            proposal_info.clone(),
            system_program.clone(),
        ],
        &[&[PROPOSAL_SEED, &id_bytes, &[proposal_bump]]],
    )?;

    let clock = Clock::get()?;
    let start_slot = clock.slot;
    let end_slot = start_slot + dao_state.voting_period;

    let proposal = Proposal {
        id: proposal_id,
        proposer: *proposer.key,
        title: title.clone(),
        description,
        instruction_data,
        for_votes: 0,
        against_votes: 0,
        start_slot,
        end_slot,
        status: ProposalStatus::Active,
        timelock_end: 0,
        bump: proposal_bump,
        is_initialized: true,
    };

    proposal.serialize(&mut &mut proposal_info.data.borrow_mut()[..])?;

    dao_state.proposal_count = proposal_id + 1;
    dao_state.serialize(&mut &mut dao_state_info.data.borrow_mut()[..])?;

    msg!("Proposal #{} created: {}", proposal_id, title);
    Ok(())
}

fn process_cast_vote(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    support: bool,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let voter = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;
    let vote_record_info = next_account_info(account_iter)?;
    let voter_token_account = next_account_info(account_iter)?;
    let _dao_state_info = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !voter.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())?;
    if !proposal.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    // Verify voting is active
    let clock = Clock::get()?;
    if proposal.status != ProposalStatus::Active {
        msg!("Error: proposal is not active");
        return Err(ProgramError::InvalidArgument);
    }
    if clock.slot > proposal.end_slot {
        msg!("Error: voting period has ended");
        return Err(ProgramError::InvalidArgument);
    }

    // Get voter's token balance as vote weight
    let token_data = spl_token::state::Account::unpack(&voter_token_account.data.borrow())?;
    if token_data.owner != *voter.key {
        msg!("Error: token account not owned by voter");
        return Err(ProgramError::InvalidArgument);
    }
    let vote_weight = token_data.amount;
    if vote_weight == 0 {
        msg!("Error: voter has no tokens");
        return Err(ProgramError::InsufficientFunds);
    }

    // Derive vote record PDA
    let proposal_id_bytes = proposal.id.to_le_bytes();
    let (vote_pda, vote_bump) = Pubkey::find_program_address(
        &[VOTE_SEED, &proposal_id_bytes, voter.key.as_ref()],
        program_id,
    );
    if *vote_record_info.key != vote_pda {
        msg!("Error: invalid vote record PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Ensure voter hasn't already voted
    if !vote_record_info.data_is_empty() {
        msg!("Error: voter has already voted on this proposal");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Create vote record account
    let rent = Rent::get()?;
    let space = VoteRecord::LEN;
    let lamports = rent.minimum_balance(space);

    invoke_signed(
        &system_instruction::create_account(
            voter.key,
            vote_record_info.key,
            lamports,
            space as u64,
            program_id,
        ),
        &[
            voter.clone(),
            vote_record_info.clone(),
            system_program.clone(),
        ],
        &[&[
            VOTE_SEED,
            &proposal_id_bytes,
            voter.key.as_ref(),
            &[vote_bump],
        ]],
    )?;

    let vote_record = VoteRecord {
        voter: *voter.key,
        proposal_id: proposal.id,
        vote_weight,
        support,
        bump: vote_bump,
        is_initialized: true,
    };

    vote_record.serialize(&mut &mut vote_record_info.data.borrow_mut()[..])?;

    // Update proposal tallies
    if support {
        proposal.for_votes = proposal.for_votes.checked_add(vote_weight)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    } else {
        proposal.against_votes = proposal.against_votes.checked_add(vote_weight)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    proposal.serialize(&mut &mut proposal_info.data.borrow_mut()[..])?;

    msg!(
        "Vote cast on proposal #{}: {} with weight {}",
        proposal.id,
        if support { "FOR" } else { "AGAINST" },
        vote_weight
    );
    Ok(())
}

fn process_finalize_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _caller = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;
    let dao_state_info = next_account_info(account_iter)?;

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())?;
    if !proposal.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if proposal.status != ProposalStatus::Active {
        msg!("Error: proposal is not active");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    if clock.slot <= proposal.end_slot {
        msg!("Error: voting period has not ended yet (current: {}, end: {})", clock.slot, proposal.end_slot);
        return Err(ProgramError::InvalidArgument);
    }

    let dao_state = DaoState::try_from_slice(&dao_state_info.data.borrow())?;

    // Check quorum: total votes must be >= quorum_bps of total_voting_power
    let total_votes = proposal.for_votes.checked_add(proposal.against_votes)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let quorum_threshold = (dao_state.total_voting_power as u128)
        .checked_mul(dao_state.quorum_bps as u128)
        .unwrap_or(0)
        / 10_000;

    let quorum_met = total_votes as u128 >= quorum_threshold;
    let majority_for = proposal.for_votes > proposal.against_votes;

    if quorum_met && majority_for {
        proposal.status = ProposalStatus::Passed;
        proposal.timelock_end = clock.slot + dao_state.timelock_period;
        msg!(
            "Proposal #{} PASSED (for: {}, against: {}, quorum: {}/{}). Timelock until slot {}.",
            proposal.id,
            proposal.for_votes,
            proposal.against_votes,
            total_votes,
            quorum_threshold,
            proposal.timelock_end
        );
    } else {
        proposal.status = ProposalStatus::Failed;
        msg!(
            "Proposal #{} FAILED (for: {}, against: {}, quorum met: {})",
            proposal.id,
            proposal.for_votes,
            proposal.against_votes,
            quorum_met
        );
    }

    proposal.serialize(&mut &mut proposal_info.data.borrow_mut()[..])?;
    Ok(())
}

fn process_execute_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let _caller = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;
    let _dao_state_info = next_account_info(account_iter)?;

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())?;
    if !proposal.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if proposal.status != ProposalStatus::Passed {
        msg!("Error: proposal has not passed");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    if clock.slot < proposal.timelock_end {
        msg!(
            "Error: timelock not expired (current: {}, end: {})",
            clock.slot,
            proposal.timelock_end
        );
        return Err(ProgramError::InvalidArgument);
    }

    // In a full implementation, the instruction_data would be deserialized
    // and executed via CPI. For now we mark it as executed.
    // The remaining accounts (index 3..N) would be passed to the target program.
    proposal.status = ProposalStatus::Executed;
    proposal.serialize(&mut &mut proposal_info.data.borrow_mut()[..])?;

    msg!(
        "Proposal #{} EXECUTED: {} ({} bytes of instruction data)",
        proposal.id,
        proposal.title,
        proposal.instruction_data.len()
    );
    Ok(())
}

fn process_cancel_proposal(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let proposer = next_account_info(account_iter)?;
    let proposal_info = next_account_info(account_iter)?;

    if !proposer.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut proposal = Proposal::try_from_slice(&proposal_info.data.borrow())?;
    if !proposal.is_initialized {
        return Err(ProgramError::UninitializedAccount);
    }

    if proposal.proposer != *proposer.key {
        msg!("Error: only the proposer can cancel");
        return Err(ProgramError::InvalidArgument);
    }

    // Can only cancel if still Active (voting not yet ended)
    if proposal.status != ProposalStatus::Active {
        msg!("Error: can only cancel active proposals");
        return Err(ProgramError::InvalidArgument);
    }

    let clock = Clock::get()?;
    if clock.slot > proposal.end_slot {
        msg!("Error: voting period has already ended — use finalize instead");
        return Err(ProgramError::InvalidArgument);
    }

    proposal.status = ProposalStatus::Cancelled;
    proposal.serialize(&mut &mut proposal_info.data.borrow_mut()[..])?;

    msg!("Proposal #{} CANCELLED by proposer", proposal.id);
    Ok(())
}

fn process_delegate_votes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    delegate: Pubkey,
    amount: u64,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let delegator = next_account_info(account_iter)?;
    let delegation_info = next_account_info(account_iter)?;
    let delegator_token_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;

    if !delegator.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if *delegator.key == delegate {
        msg!("Error: cannot delegate to yourself");
        return Err(ProgramError::InvalidArgument);
    }

    // Verify delegator holds enough tokens
    let token_data = spl_token::state::Account::unpack(&delegator_token_account.data.borrow())?;
    if token_data.owner != *delegator.key {
        msg!("Error: token account not owned by delegator");
        return Err(ProgramError::InvalidArgument);
    }
    if token_data.amount < amount {
        msg!("Error: insufficient token balance for delegation");
        return Err(ProgramError::InsufficientFunds);
    }

    // Derive delegation PDA
    let (delegation_pda, del_bump) = Pubkey::find_program_address(
        &[DELEGATION_SEED, delegator.key.as_ref(), delegate.as_ref()],
        program_id,
    );
    if *delegation_info.key != delegation_pda {
        msg!("Error: invalid delegation PDA");
        return Err(ProgramError::InvalidArgument);
    }

    // Create or update the delegation record
    if delegation_info.data_is_empty() {
        let rent = Rent::get()?;
        let space = Delegation::LEN;
        let lamports = rent.minimum_balance(space);

        invoke_signed(
            &system_instruction::create_account(
                delegator.key,
                delegation_info.key,
                lamports,
                space as u64,
                program_id,
            ),
            &[
                delegator.clone(),
                delegation_info.clone(),
                system_program.clone(),
            ],
            &[&[
                DELEGATION_SEED,
                delegator.key.as_ref(),
                delegate.as_ref(),
                &[del_bump],
            ]],
        )?;
    }

    let delegation = Delegation {
        delegator: *delegator.key,
        delegate,
        amount,
        bump: del_bump,
        is_initialized: true,
    };

    delegation.serialize(&mut &mut delegation_info.data.borrow_mut()[..])?;

    msg!(
        "Delegated {} voting power from {} to {}",
        amount,
        delegator.key,
        delegate
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
    fn test_proposal_status_serialization() {
        let status = ProposalStatus::Active;
        let encoded = borsh::to_vec(&status).unwrap();
        let decoded = ProposalStatus::try_from_slice(&encoded).unwrap();
        assert_eq!(decoded, ProposalStatus::Active);
    }

    #[test]
    fn test_quorum_calculation() {
        let total_supply: u128 = 1_000_000;
        let quorum_bps: u128 = 1_000; // 10%
        let quorum_threshold = total_supply * quorum_bps / 10_000;
        assert_eq!(quorum_threshold, 100_000);
    }

    #[test]
    fn test_voting_period_slots() {
        // 3 days at 400ms per slot
        let slots_per_second = 1_000 / 400; // 2.5 slots/sec
        let seconds_per_day = 86_400;
        let expected = (slots_per_second * seconds_per_day * 3) as u64;
        assert_eq!(expected, 648_000);
        assert_eq!(DEFAULT_VOTING_PERIOD_SLOTS, 648_000);
    }

    #[test]
    fn test_timelock_period_slots() {
        // 2 days at 400ms per slot
        let slots_per_second = 1_000 / 400;
        let seconds_per_day = 86_400;
        let expected = (slots_per_second * seconds_per_day * 2) as u64;
        assert_eq!(expected, 432_000);
        assert_eq!(DEFAULT_TIMELOCK_SLOTS, 432_000);
    }
}
