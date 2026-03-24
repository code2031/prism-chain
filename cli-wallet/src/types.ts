// ─── Configuration ──────────────────────────────────────────────────────────

export interface SolCloneConfig {
  rpc_url: string;
  keypair_path: string;
  commitment: Commitment;
}

export type Commitment = 'processed' | 'confirmed' | 'finalized';

// ─── JSON-RPC ───────────────────────────────────────────────────────────────

export interface JsonRpcRequest {
  jsonrpc: '2.0';
  id: number;
  method: string;
  params?: unknown[];
}

export interface JsonRpcResponse<T = unknown> {
  jsonrpc: '2.0';
  id: number;
  result?: T;
  error?: JsonRpcError;
}

export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

// ─── Account ────────────────────────────────────────────────────────────────

export interface AccountInfo {
  data: [string, string]; // [base64Data, encoding]
  executable: boolean;
  lamports: number;
  owner: string;
  rentEpoch: number;
  space: number;
}

export interface AccountInfoResponse {
  context: { slot: number };
  value: AccountInfo | null;
}

// ─── Balance ────────────────────────────────────────────────────────────────

export interface BalanceResponse {
  context: { slot: number };
  value: number;
}

// ─── Transaction ────────────────────────────────────────────────────────────

export interface TransactionInstruction {
  programIdIndex: number;
  accounts: number[];
  data: string; // base58 encoded
}

export interface TransactionMessage {
  accountKeys: string[];
  header: {
    numRequiredSignatures: number;
    numReadonlySignedAccounts: number;
    numReadonlyUnsignedAccounts: number;
  };
  instructions: TransactionInstruction[];
  recentBlockhash: string;
}

export interface SerializedTransaction {
  signatures: string[];
  message: TransactionMessage;
}

export interface SendTransactionResponse {
  // Transaction signature (base58)
  signature: string;
}

export interface BlockhashResponse {
  context: { slot: number };
  value: {
    blockhash: string;
    lastValidBlockHeight: number;
  };
}

export interface SignatureStatus {
  slot: number;
  confirmations: number | null;
  err: unknown | null;
  confirmationStatus: Commitment | null;
}

export interface SignatureStatusResponse {
  context: { slot: number };
  value: (SignatureStatus | null)[];
}

// ─── Token ──────────────────────────────────────────────────────────────────

export interface TokenAccountInfo {
  pubkey: string;
  account: {
    data: {
      parsed: {
        info: {
          isNative: boolean;
          mint: string;
          owner: string;
          state: string;
          tokenAmount: {
            amount: string;
            decimals: number;
            uiAmount: number;
            uiAmountString: string;
          };
        };
        type: string;
      };
      program: string;
      space: number;
    };
    executable: boolean;
    lamports: number;
    owner: string;
    rentEpoch: number;
  };
}

export interface TokenAccountsResponse {
  context: { slot: number };
  value: TokenAccountInfo[];
}

export interface TokenBalanceInfo {
  context: { slot: number };
  value: {
    amount: string;
    decimals: number;
    uiAmount: number;
    uiAmountString: string;
  };
}

// ─── Staking ────────────────────────────────────────────────────────────────

export interface StakeActivation {
  active: number;
  inactive: number;
  state: 'active' | 'inactive' | 'activating' | 'deactivating';
}

export interface VoteAccount {
  votePubkey: string;
  nodePubkey: string;
  activatedStake: number;
  epochVoteAccount: boolean;
  commission: number;
  lastVote: number;
}

export interface VoteAccountsResponse {
  current: VoteAccount[];
  delinquent: VoteAccount[];
}

// ─── Transaction History ────────────────────────────────────────────────────

export interface ConfirmedSignature {
  signature: string;
  slot: number;
  err: unknown | null;
  memo: string | null;
  blockTime: number | null;
  confirmationStatus: Commitment | null;
}

export interface TransactionDetail {
  slot: number;
  transaction: {
    message: TransactionMessage;
    signatures: string[];
  };
  meta: {
    err: unknown | null;
    fee: number;
    preBalances: number[];
    postBalances: number[];
    logMessages: string[];
  } | null;
  blockTime: number | null;
}

// ─── Cluster Info ───────────────────────────────────────────────────────────

export interface ClusterNode {
  pubkey: string;
  gossip: string | null;
  tpu: string | null;
  rpc: string | null;
  version: string | null;
  featureSet: number | null;
}

export interface EpochInfo {
  absoluteSlot: number;
  blockHeight: number;
  epoch: number;
  slotIndex: number;
  slotsInEpoch: number;
  transactionCount: number;
}

export interface VersionInfo {
  'solana-core': string;
  'feature-set': number;
}

// ─── Keypair ────────────────────────────────────────────────────────────────

export interface KeypairData {
  publicKey: Uint8Array;
  secretKey: Uint8Array;
}

// ─── Instruction Building ───────────────────────────────────────────────────

export interface CompiledInstruction {
  programIdIndex: number;
  accountIndexes: number[];
  data: Buffer;
}

export interface TransactionBuildParams {
  feePayer: Uint8Array; // 32-byte public key
  recentBlockhash: string;
  instructions: InstructionInput[];
}

export interface InstructionInput {
  programId: Uint8Array; // 32-byte public key
  keys: InstructionAccount[];
  data: Buffer;
}

export interface InstructionAccount {
  pubkey: Uint8Array; // 32-byte public key
  isSigner: boolean;
  isWritable: boolean;
}
