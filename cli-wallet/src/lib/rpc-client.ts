import * as http from 'http';
import * as https from 'https';
import { URL } from 'url';
import {
  JsonRpcRequest,
  JsonRpcResponse,
  AccountInfoResponse,
  BalanceResponse,
  BlockhashResponse,
  SignatureStatusResponse,
  ConfirmedSignature,
  TransactionDetail,
  ClusterNode,
  EpochInfo,
  VersionInfo,
  VoteAccountsResponse,
  TokenAccountsResponse,
  TokenBalanceInfo,
  Commitment,
} from '../types';
import { loadConfig } from './config';

let requestId = 0;

/**
 * Raw JSON-RPC client for Solana-compatible APIs.
 * No dependency on @solana/web3.js.
 */
export class RpcClient {
  private url: string;
  private commitment: Commitment;

  constructor(url?: string, commitment?: Commitment) {
    const config = loadConfig();
    this.url = url || config.rpc_url;
    this.commitment = commitment || config.commitment;
  }

  // ─── Core RPC Call ──────────────────────────────────────────────────────

  /**
   * Send a raw JSON-RPC request and return the parsed response.
   */
  async call<T>(method: string, params?: unknown[]): Promise<T> {
    const payload: JsonRpcRequest = {
      jsonrpc: '2.0',
      id: ++requestId,
      method,
      params: params || [],
    };

    const body = JSON.stringify(payload);
    const response = await this.httpPost(body);
    const parsed = JSON.parse(response) as JsonRpcResponse<T>;

    if (parsed.error) {
      const errMsg = parsed.error.message || 'Unknown RPC error';
      const errCode = parsed.error.code || 0;
      throw new RpcError(errMsg, errCode, parsed.error.data);
    }

    return parsed.result as T;
  }

  /**
   * Low-level HTTP POST to the RPC endpoint.
   */
  private httpPost(body: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const parsedUrl = new URL(this.url);
      const transport = parsedUrl.protocol === 'https:' ? https : http;

      const options = {
        hostname: parsedUrl.hostname,
        port: parsedUrl.port || (parsedUrl.protocol === 'https:' ? 443 : 80),
        path: parsedUrl.pathname + parsedUrl.search,
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(body),
        },
        timeout: 30000,
      };

      const req = transport.request(options, (res) => {
        let data = '';
        res.on('data', (chunk: Buffer) => {
          data += chunk.toString();
        });
        res.on('end', () => {
          if (res.statusCode && res.statusCode >= 400) {
            reject(new Error(`HTTP ${res.statusCode}: ${data}`));
          } else {
            resolve(data);
          }
        });
      });

      req.on('error', (err) => {
        reject(new Error(`Connection failed: ${err.message}. Is the RPC server running at ${this.url}?`));
      });

      req.on('timeout', () => {
        req.destroy();
        reject(new Error(`Request timed out after 30s to ${this.url}`));
      });

      req.write(body);
      req.end();
    });
  }

  // ─── Account Methods ────────────────────────────────────────────────────

  /**
   * Get the balance of an account in lamports.
   */
  async getBalance(address: string): Promise<BalanceResponse> {
    return this.call<BalanceResponse>('getBalance', [
      address,
      { commitment: this.commitment },
    ]);
  }

  /**
   * Get account info.
   */
  async getAccountInfo(address: string, encoding: string = 'base64'): Promise<AccountInfoResponse> {
    return this.call<AccountInfoResponse>('getAccountInfo', [
      address,
      { encoding, commitment: this.commitment },
    ]);
  }

  // ─── Transaction Methods ────────────────────────────────────────────────

  /**
   * Get a recent blockhash.
   */
  async getLatestBlockhash(): Promise<BlockhashResponse> {
    return this.call<BlockhashResponse>('getLatestBlockhash', [
      { commitment: this.commitment },
    ]);
  }

  /**
   * Send a signed, serialized transaction.
   * @param txBase64 - The base64-encoded signed transaction.
   */
  async sendTransaction(txBase64: string): Promise<string> {
    return this.call<string>('sendTransaction', [
      txBase64,
      {
        encoding: 'base64',
        skipPreflight: false,
        preflightCommitment: this.commitment,
      },
    ]);
  }

  /**
   * Confirm a transaction signature.
   */
  async confirmTransaction(signature: string, timeout: number = 30000): Promise<SignatureStatusResponse> {
    const start = Date.now();

    while (Date.now() - start < timeout) {
      const status = await this.call<SignatureStatusResponse>('getSignatureStatuses', [
        [signature],
        { searchTransactionHistory: true },
      ]);

      if (status.value[0] !== null) {
        const s = status.value[0];
        if (s.err) {
          throw new Error(`Transaction failed: ${JSON.stringify(s.err)}`);
        }
        if (
          s.confirmationStatus === 'confirmed' ||
          s.confirmationStatus === 'finalized'
        ) {
          return status;
        }
      }

      await new Promise((resolve) => setTimeout(resolve, 1000));
    }

    throw new Error(`Transaction confirmation timed out after ${timeout / 1000}s`);
  }

  /**
   * Get a transaction by signature.
   */
  async getTransaction(signature: string): Promise<TransactionDetail | null> {
    return this.call<TransactionDetail | null>('getTransaction', [
      signature,
      { encoding: 'json', commitment: this.commitment, maxSupportedTransactionVersion: 0 },
    ]);
  }

  /**
   * Request an airdrop.
   */
  async requestAirdrop(address: string, lamports: number): Promise<string> {
    return this.call<string>('requestAirdrop', [address, lamports]);
  }

  // ─── History Methods ────────────────────────────────────────────────────

  /**
   * Get confirmed signatures for an address.
   */
  async getSignaturesForAddress(
    address: string,
    limit: number = 10
  ): Promise<ConfirmedSignature[]> {
    return this.call<ConfirmedSignature[]>('getSignaturesForAddress', [
      address,
      { limit, commitment: this.commitment },
    ]);
  }

  // ─── Token Methods ─────────────────────────────────────────────────────

  /**
   * Get all token accounts owned by an address.
   */
  async getTokenAccountsByOwner(
    owner: string,
    programId: string = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
  ): Promise<TokenAccountsResponse> {
    return this.call<TokenAccountsResponse>('getTokenAccountsByOwner', [
      owner,
      { programId },
      { encoding: 'jsonParsed', commitment: this.commitment },
    ]);
  }

  /**
   * Get the token balance of a specific token account.
   */
  async getTokenAccountBalance(tokenAccount: string): Promise<TokenBalanceInfo> {
    return this.call<TokenBalanceInfo>('getTokenAccountBalance', [
      tokenAccount,
      { commitment: this.commitment },
    ]);
  }

  // ─── Cluster Methods ───────────────────────────────────────────────────

  /**
   * Get cluster nodes.
   */
  async getClusterNodes(): Promise<ClusterNode[]> {
    return this.call<ClusterNode[]>('getClusterNodes');
  }

  /**
   * Get epoch info.
   */
  async getEpochInfo(): Promise<EpochInfo> {
    return this.call<EpochInfo>('getEpochInfo', [{ commitment: this.commitment }]);
  }

  /**
   * Get the version of the node.
   */
  async getVersion(): Promise<VersionInfo> {
    return this.call<VersionInfo>('getVersion');
  }

  /**
   * Get the current slot.
   */
  async getSlot(): Promise<number> {
    return this.call<number>('getSlot', [{ commitment: this.commitment }]);
  }

  /**
   * Get the current block height.
   */
  async getBlockHeight(): Promise<number> {
    return this.call<number>('getBlockHeight', [{ commitment: this.commitment }]);
  }

  /**
   * Get the leader schedule.
   */
  async getSlotLeader(): Promise<string> {
    return this.call<string>('getSlotLeader', [{ commitment: this.commitment }]);
  }

  /**
   * Get vote accounts.
   */
  async getVoteAccounts(): Promise<VoteAccountsResponse> {
    return this.call<VoteAccountsResponse>('getVoteAccounts', [
      { commitment: this.commitment },
    ]);
  }

  /**
   * Get the minimum balance needed for rent exemption.
   */
  async getMinimumBalanceForRentExemption(dataSize: number): Promise<number> {
    return this.call<number>('getMinimumBalanceForRentExemption', [
      dataSize,
      { commitment: this.commitment },
    ]);
  }

  /**
   * Get the supply of SOL.
   */
  async getSupply(): Promise<{
    context: { slot: number };
    value: {
      total: number;
      circulating: number;
      nonCirculating: number;
      nonCirculatingAccounts: string[];
    };
  }> {
    return this.call('getSupply', [{ commitment: this.commitment }]);
  }

  /**
   * Get the transaction count.
   */
  async getTransactionCount(): Promise<number> {
    return this.call<number>('getTransactionCount', [{ commitment: this.commitment }]);
  }

  /**
   * Get the RPC URL in use.
   */
  getUrl(): string {
    return this.url;
  }
}

/**
 * Custom error class for RPC errors.
 */
export class RpcError extends Error {
  code: number;
  data: unknown;

  constructor(message: string, code: number, data?: unknown) {
    super(message);
    this.name = 'RpcError';
    this.code = code;
    this.data = data;
  }
}

/**
 * Create a new RPC client with the current config.
 */
export function createRpcClient(): RpcClient {
  return new RpcClient();
}
