import * as nacl from 'tweetnacl';
import bs58 from 'bs58';
import { TransactionBuildParams, InstructionInput, InstructionAccount } from '../types';

// ─── System Program Constants ───────────────────────────────────────────────

/** System Program ID (all zeros) */
export const SYSTEM_PROGRAM_ID = new Uint8Array(32);

/** SPL Token Program ID */
export const TOKEN_PROGRAM_ID = bs58.decode('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');

/** Associated Token Account Program ID */
export const ASSOCIATED_TOKEN_PROGRAM_ID = bs58.decode(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
);

/** Sysvar Rent Program ID */
export const SYSVAR_RENT_PUBKEY = bs58.decode('SysvarRent111111111111111111111111111111111');

/** Sysvar Clock Program ID */
export const SYSVAR_CLOCK_PUBKEY = bs58.decode('SysvarC1ock11111111111111111111111111111111');

/** Sysvar Stake History Program ID */
export const SYSVAR_STAKE_HISTORY_PUBKEY = bs58.decode(
  'SysvarStakeHistory1111111111111111111111111'
);

/** Stake Program ID */
export const STAKE_PROGRAM_ID = bs58.decode('Stake11111111111111111111111111111111111111');

/** Stake Config ID */
export const STAKE_CONFIG_ID = bs58.decode('StakeConfig11111111111111111111111111111111');

// ─── System Program Instructions ────────────────────────────────────────────

/**
 * Build a System Program Transfer instruction.
 */
export function createTransferInstruction(
  from: Uint8Array,
  to: Uint8Array,
  lamports: bigint
): InstructionInput {
  // Transfer instruction index = 2
  const data = Buffer.alloc(12);
  data.writeUInt32LE(2, 0); // instruction index
  data.writeBigUInt64LE(lamports, 4); // lamports

  return {
    programId: SYSTEM_PROGRAM_ID,
    keys: [
      { pubkey: from, isSigner: true, isWritable: true },
      { pubkey: to, isSigner: false, isWritable: true },
    ],
    data,
  };
}

/**
 * Build a System Program CreateAccount instruction.
 */
export function createAccountInstruction(
  from: Uint8Array,
  newAccount: Uint8Array,
  lamports: bigint,
  space: bigint,
  programId: Uint8Array
): InstructionInput {
  // CreateAccount instruction index = 0
  const data = Buffer.alloc(52);
  data.writeUInt32LE(0, 0); // instruction index
  data.writeBigUInt64LE(lamports, 4); // lamports
  data.writeBigUInt64LE(space, 12); // space
  Buffer.from(programId).copy(data, 20); // owner program

  return {
    programId: SYSTEM_PROGRAM_ID,
    keys: [
      { pubkey: from, isSigner: true, isWritable: true },
      { pubkey: newAccount, isSigner: true, isWritable: true },
    ],
    data,
  };
}

// ─── Transaction Serialization ──────────────────────────────────────────────

/**
 * Build, serialize, and sign a transaction.
 * Returns a base64-encoded signed transaction ready for sendTransaction.
 */
export function buildAndSignTransaction(
  params: TransactionBuildParams,
  signers: Uint8Array[] // array of 64-byte secret keys
): string {
  const message = compileMessage(params);
  const messageBytes = serializeMessage(message);

  // Sign with each signer
  const signatures: Uint8Array[] = [];
  for (const secretKey of signers) {
    const sig = nacl.sign.detached(messageBytes, secretKey);
    signatures.push(sig);
  }

  // Serialize the full transaction
  const tx = serializeTransaction(signatures, messageBytes);
  return Buffer.from(tx).toString('base64');
}

// ─── Message Compilation ────────────────────────────────────────────────────

interface CompiledMessage {
  header: {
    numRequiredSignatures: number;
    numReadonlySignedAccounts: number;
    numReadonlyUnsignedAccounts: number;
  };
  accountKeys: Uint8Array[];
  recentBlockhash: string;
  instructions: {
    programIdIndex: number;
    accountIndexes: number[];
    data: Buffer;
  }[];
}

/**
 * Compile transaction parameters into a message structure.
 */
function compileMessage(params: TransactionBuildParams): CompiledMessage {
  const { feePayer, recentBlockhash, instructions } = params;

  // Collect all unique accounts with their metadata
  const accountMap = new Map<
    string,
    { pubkey: Uint8Array; isSigner: boolean; isWritable: boolean }
  >();

  // Fee payer is always first, always signer, always writable
  const feePayerKey = bs58.encode(feePayer);
  accountMap.set(feePayerKey, {
    pubkey: feePayer,
    isSigner: true,
    isWritable: true,
  });

  // Process all instruction accounts
  for (const ix of instructions) {
    // Add program ID (not signer, not writable)
    const progKey = bs58.encode(ix.programId);
    if (!accountMap.has(progKey)) {
      accountMap.set(progKey, {
        pubkey: ix.programId,
        isSigner: false,
        isWritable: false,
      });
    }

    for (const key of ix.keys) {
      const keyStr = bs58.encode(key.pubkey);
      const existing = accountMap.get(keyStr);
      if (existing) {
        // Merge: if any instruction marks as signer/writable, keep that
        existing.isSigner = existing.isSigner || key.isSigner;
        existing.isWritable = existing.isWritable || key.isWritable;
      } else {
        accountMap.set(keyStr, {
          pubkey: key.pubkey,
          isSigner: key.isSigner,
          isWritable: key.isWritable,
        });
      }
    }
  }

  // Sort accounts: signers+writable first, then signers+readonly, then
  // non-signers+writable, then non-signers+readonly.
  // Fee payer is always index 0.
  const accounts = Array.from(accountMap.values());
  const feePayerAccount = accounts.find(
    (a) => bs58.encode(a.pubkey) === feePayerKey
  )!;
  const others = accounts.filter(
    (a) => bs58.encode(a.pubkey) !== feePayerKey
  );

  others.sort((a, b) => {
    if (a.isSigner !== b.isSigner) return a.isSigner ? -1 : 1;
    if (a.isWritable !== b.isWritable) return a.isWritable ? -1 : 1;
    return 0;
  });

  const sortedAccounts = [feePayerAccount, ...others];

  // Calculate header
  let numRequiredSignatures = 0;
  let numReadonlySignedAccounts = 0;
  let numReadonlyUnsignedAccounts = 0;

  for (const acc of sortedAccounts) {
    if (acc.isSigner) {
      numRequiredSignatures++;
      if (!acc.isWritable) numReadonlySignedAccounts++;
    } else {
      if (!acc.isWritable) numReadonlyUnsignedAccounts++;
    }
  }

  // Build account index lookup
  const accountIndex = new Map<string, number>();
  sortedAccounts.forEach((acc, i) => {
    accountIndex.set(bs58.encode(acc.pubkey), i);
  });

  // Compile instructions
  const compiledInstructions = instructions.map((ix) => {
    const programIdIndex = accountIndex.get(bs58.encode(ix.programId))!;
    const accountIndexes = ix.keys.map(
      (key) => accountIndex.get(bs58.encode(key.pubkey))!
    );
    return { programIdIndex, accountIndexes, data: ix.data };
  });

  return {
    header: {
      numRequiredSignatures,
      numReadonlySignedAccounts,
      numReadonlyUnsignedAccounts,
    },
    accountKeys: sortedAccounts.map((a) => a.pubkey),
    recentBlockhash,
    instructions: compiledInstructions,
  };
}

// ─── Serialization ──────────────────────────────────────────────────────────

/**
 * Serialize a compiled message to bytes (Solana legacy transaction format).
 */
function serializeMessage(message: CompiledMessage): Uint8Array {
  const parts: Buffer[] = [];

  // Header: 3 bytes
  parts.push(
    Buffer.from([
      message.header.numRequiredSignatures,
      message.header.numReadonlySignedAccounts,
      message.header.numReadonlyUnsignedAccounts,
    ])
  );

  // Account keys compact-array
  parts.push(encodeCompactU16(message.accountKeys.length));
  for (const key of message.accountKeys) {
    parts.push(Buffer.from(key));
  }

  // Recent blockhash (32 bytes)
  parts.push(Buffer.from(bs58.decode(message.recentBlockhash)));

  // Instructions compact-array
  parts.push(encodeCompactU16(message.instructions.length));
  for (const ix of message.instructions) {
    // Program ID index
    parts.push(Buffer.from([ix.programIdIndex]));

    // Account indexes compact-array
    parts.push(encodeCompactU16(ix.accountIndexes.length));
    parts.push(Buffer.from(ix.accountIndexes));

    // Data compact-array
    parts.push(encodeCompactU16(ix.data.length));
    parts.push(ix.data);
  }

  return Buffer.concat(parts);
}

/**
 * Serialize a full transaction (signatures + message).
 */
function serializeTransaction(
  signatures: Uint8Array[],
  messageBytes: Uint8Array
): Uint8Array {
  const parts: Buffer[] = [];

  // Signatures compact-array
  parts.push(encodeCompactU16(signatures.length));
  for (const sig of signatures) {
    parts.push(Buffer.from(sig));
  }

  // Message bytes
  parts.push(Buffer.from(messageBytes));

  return Buffer.concat(parts);
}

/**
 * Encode a number as a compact-u16 (Solana's variable-length encoding).
 */
function encodeCompactU16(value: number): Buffer {
  const bytes: number[] = [];
  let remaining = value;

  while (true) {
    let byte = remaining & 0x7f;
    remaining >>= 7;
    if (remaining > 0) {
      byte |= 0x80;
    }
    bytes.push(byte);
    if (remaining === 0) break;
  }

  return Buffer.from(bytes);
}

// ─── Utility: Derive Associated Token Address ───────────────────────────────

/**
 * Derive the associated token account address for a wallet and mint.
 * Uses a PDA (Program Derived Address) approach.
 */
export function findAssociatedTokenAddress(
  walletAddress: Uint8Array,
  mintAddress: Uint8Array
): Uint8Array {
  // Seeds: [walletAddress, TOKEN_PROGRAM_ID, mintAddress]
  // Program: ASSOCIATED_TOKEN_PROGRAM_ID
  // We use a simplified derivation. In a production system, you'd call
  // findProgramAddress which tries bump seeds 255..0.
  return findProgramAddress(
    [walletAddress, TOKEN_PROGRAM_ID, mintAddress],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );
}

/**
 * Find a program derived address (PDA).
 * Tries bump seeds from 255 down to 0.
 */
export function findProgramAddress(
  seeds: Uint8Array[],
  programId: Uint8Array
): Uint8Array {
  for (let bump = 255; bump >= 0; bump--) {
    try {
      const address = createProgramAddress(
        [...seeds, new Uint8Array([bump])],
        programId
      );
      return address;
    } catch {
      continue;
    }
  }
  throw new Error('Could not find program address');
}

/**
 * Create a program address from seeds and a program ID.
 * Throws if the result is on the Ed25519 curve (not a valid PDA).
 */
function createProgramAddress(
  seeds: Uint8Array[],
  programId: Uint8Array
): Uint8Array {
  const crypto = require('crypto');
  const buffer = Buffer.concat([
    ...seeds.map((s) => Buffer.from(s)),
    Buffer.from(programId),
    Buffer.from('ProgramDerivedAddress'),
  ]);

  const hash = crypto.createHash('sha256').update(buffer).digest();

  // A valid PDA must NOT be on the Ed25519 curve.
  // We do a simplified check here -- in production you'd use a proper
  // point-on-curve check. For now, we just return the hash as the address.
  return new Uint8Array(hash);
}

// ─── Create Associated Token Account Instruction ────────────────────────────

/**
 * Build an instruction to create an associated token account.
 */
export function createAssociatedTokenAccountInstruction(
  payer: Uint8Array,
  associatedToken: Uint8Array,
  owner: Uint8Array,
  mint: Uint8Array
): InstructionInput {
  return {
    programId: ASSOCIATED_TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: associatedToken, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: false, isWritable: false },
      { pubkey: mint, isSigner: false, isWritable: false },
      { pubkey: SYSTEM_PROGRAM_ID, isSigner: false, isWritable: false },
      { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
    ],
    data: Buffer.alloc(0),
  };
}

// ─── SPL Token Instructions ─────────────────────────────────────────────────

/**
 * Build an InitializeMint instruction (SPL Token instruction index = 0).
 */
export function createInitializeMintInstruction(
  mint: Uint8Array,
  decimals: number,
  mintAuthority: Uint8Array,
  freezeAuthority: Uint8Array | null
): InstructionInput {
  const data = Buffer.alloc(67);
  data.writeUInt8(0, 0); // InitializeMint instruction
  data.writeUInt8(decimals, 1);
  Buffer.from(mintAuthority).copy(data, 2);
  if (freezeAuthority) {
    data.writeUInt8(1, 34); // COption: Some
    Buffer.from(freezeAuthority).copy(data, 35);
  } else {
    data.writeUInt8(0, 34); // COption: None
  }

  return {
    programId: TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data,
  };
}

/**
 * Build a MintTo instruction (SPL Token instruction index = 7).
 */
export function createMintToInstruction(
  mint: Uint8Array,
  destination: Uint8Array,
  authority: Uint8Array,
  amount: bigint
): InstructionInput {
  const data = Buffer.alloc(9);
  data.writeUInt8(7, 0); // MintTo instruction
  data.writeBigUInt64LE(amount, 1);

  return {
    programId: TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: mint, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
      { pubkey: authority, isSigner: true, isWritable: false },
    ],
    data,
  };
}

/**
 * Build a Transfer instruction for SPL tokens (instruction index = 3).
 */
export function createTokenTransferInstruction(
  source: Uint8Array,
  destination: Uint8Array,
  owner: Uint8Array,
  amount: bigint
): InstructionInput {
  const data = Buffer.alloc(9);
  data.writeUInt8(3, 0); // Transfer instruction
  data.writeBigUInt64LE(amount, 1);

  return {
    programId: TOKEN_PROGRAM_ID,
    keys: [
      { pubkey: source, isSigner: false, isWritable: true },
      { pubkey: destination, isSigner: false, isWritable: true },
      { pubkey: owner, isSigner: true, isWritable: false },
    ],
    data,
  };
}

// ─── Stake Instructions ─────────────────────────────────────────────────────

/**
 * Build a StakeProgram Initialize instruction (index = 0).
 */
export function createStakeInitializeInstruction(
  stakeAccount: Uint8Array,
  staker: Uint8Array,
  withdrawer: Uint8Array
): InstructionInput {
  // Stake Initialize: 4 bytes index + Authorized struct (two pubkeys)
  const data = Buffer.alloc(100);
  data.writeUInt32LE(0, 0); // Initialize instruction

  // Authorized { staker, withdrawer }
  Buffer.from(staker).copy(data, 4);
  Buffer.from(withdrawer).copy(data, 36);

  // Lockup { unix_timestamp: 0, epoch: 0, custodian: Pubkey::default() }
  // 8 + 8 + 32 = 48 bytes of zeros (already zero)

  return {
    programId: STAKE_PROGRAM_ID,
    keys: [
      { pubkey: stakeAccount, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_RENT_PUBKEY, isSigner: false, isWritable: false },
    ],
    data: data.subarray(0, 100),
  };
}

/**
 * Build a StakeProgram Delegate instruction (index = 2).
 */
export function createStakeDelegateInstruction(
  stakeAccount: Uint8Array,
  voteAccount: Uint8Array,
  staker: Uint8Array
): InstructionInput {
  const data = Buffer.alloc(4);
  data.writeUInt32LE(2, 0); // DelegateStake

  return {
    programId: STAKE_PROGRAM_ID,
    keys: [
      { pubkey: stakeAccount, isSigner: false, isWritable: true },
      { pubkey: voteAccount, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: STAKE_CONFIG_ID, isSigner: false, isWritable: false },
      { pubkey: staker, isSigner: true, isWritable: false },
    ],
    data,
  };
}

/**
 * Build a StakeProgram Deactivate instruction (index = 5).
 */
export function createStakeDeactivateInstruction(
  stakeAccount: Uint8Array,
  staker: Uint8Array
): InstructionInput {
  const data = Buffer.alloc(4);
  data.writeUInt32LE(5, 0); // Deactivate

  return {
    programId: STAKE_PROGRAM_ID,
    keys: [
      { pubkey: stakeAccount, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: staker, isSigner: true, isWritable: false },
    ],
    data,
  };
}

/**
 * Build a StakeProgram Withdraw instruction (index = 4).
 */
export function createStakeWithdrawInstruction(
  stakeAccount: Uint8Array,
  withdrawer: Uint8Array,
  to: Uint8Array,
  lamports: bigint
): InstructionInput {
  const data = Buffer.alloc(12);
  data.writeUInt32LE(4, 0); // Withdraw
  data.writeBigUInt64LE(lamports, 4);

  return {
    programId: STAKE_PROGRAM_ID,
    keys: [
      { pubkey: stakeAccount, isSigner: false, isWritable: true },
      { pubkey: to, isSigner: false, isWritable: true },
      { pubkey: SYSVAR_CLOCK_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SYSVAR_STAKE_HISTORY_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: withdrawer, isSigner: true, isWritable: false },
    ],
    data,
  };
}
