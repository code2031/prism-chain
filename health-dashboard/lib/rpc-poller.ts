"use client";

import useSWR from "swr";

// ---------- Types ----------

export interface ValidatorInfo {
  identity: string;
  stake: number; // lamports
  commission: number;
  lastVote: number;
  delinquent: boolean;
}

export interface ClusterStats {
  slot: number;
  epoch: number;
  epochProgress: number; // 0-100
  slotsInEpoch: number;
  slotIndex: number;
  tps: number;
  validatorCount: number;
  validators: ValidatorInfo[];
  tpsSamples: number[];
}

// ---------- RPC helpers ----------

const RPC_URL = "https://api.devnet.solana.com";

async function rpcCall<T>(method: string, params: unknown[] = []): Promise<T> {
  const res = await fetch(RPC_URL, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: 1, method, params }),
  });
  const json = await res.json();
  if (json.error) throw new Error(json.error.message);
  return json.result as T;
}

interface EpochInfo {
  epoch: number;
  slotIndex: number;
  slotsInEpoch: number;
  absoluteSlot: number;
  blockHeight: number;
  transactionCount: number;
}

interface PerfSample {
  numTransactions: number;
  numSlots: number;
  samplePeriodSecs: number;
  slot: number;
}

interface VoteAccountsResult {
  current: { nodePubkey: string; activatedStake: number; commission: number; lastVote: number }[];
  delinquent: { nodePubkey: string; activatedStake: number; commission: number; lastVote: number }[];
}

// ---------- Fetcher ----------

async function fetchClusterStats(): Promise<ClusterStats> {
  const [epochInfo, perfSamples, voteAccounts] = await Promise.all([
    rpcCall<EpochInfo>("getEpochInfo"),
    rpcCall<PerfSample[]>("getRecentPerformanceSamples", [60]),
    rpcCall<VoteAccountsResult>("getVoteAccounts"),
  ]);

  const epochProgress =
    epochInfo.slotsInEpoch > 0
      ? (epochInfo.slotIndex / epochInfo.slotsInEpoch) * 100
      : 0;

  // Compute TPS from the most recent sample
  const latestSample = perfSamples[0];
  const tps =
    latestSample && latestSample.samplePeriodSecs > 0
      ? Math.round(latestSample.numTransactions / latestSample.samplePeriodSecs)
      : 0;

  // TPS history array (oldest -> newest)
  const tpsSamples = perfSamples
    .slice()
    .reverse()
    .map((s) =>
      s.samplePeriodSecs > 0 ? Math.round(s.numTransactions / s.samplePeriodSecs) : 0
    );

  // Combine validators
  const mapValidator = (v: VoteAccountsResult["current"][number], delinquent: boolean): ValidatorInfo => ({
    identity: v.nodePubkey,
    stake: v.activatedStake,
    commission: v.commission,
    lastVote: v.lastVote,
    delinquent,
  });

  const validators: ValidatorInfo[] = [
    ...voteAccounts.current.map((v) => mapValidator(v, false)),
    ...voteAccounts.delinquent.map((v) => mapValidator(v, true)),
  ];

  // Sort by stake descending
  validators.sort((a, b) => b.stake - a.stake);

  return {
    slot: epochInfo.absoluteSlot,
    epoch: epochInfo.epoch,
    epochProgress,
    slotsInEpoch: epochInfo.slotsInEpoch,
    slotIndex: epochInfo.slotIndex,
    tps,
    validatorCount: validators.length,
    validators,
    tpsSamples,
  };
}

// ---------- SWR hook ----------

export function useClusterStats() {
  const { data, error, isLoading, isValidating } = useSWR<ClusterStats>(
    "cluster-stats",
    fetchClusterStats,
    {
      refreshInterval: 2000,
      revalidateOnFocus: false,
      dedupingInterval: 1000,
    }
  );

  return { data, error, isLoading, isValidating };
}
