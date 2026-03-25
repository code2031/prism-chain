"use client";

import { useState } from "react";

interface Voter {
  address: string;
  weight: number;
  support: boolean;
}

interface ProposalDetailProps {
  id: number;
  title: string;
  description: string;
  status: "Active" | "Passed" | "Failed" | "Executed" | "Cancelled" | "Pending";
  forVotes: number;
  againstVotes: number;
  startSlot: number;
  endSlot: number;
  currentSlot: number;
  timelockEnd: number;
  proposer: string;
  voters: Voter[];
  onVoteFor?: () => void;
  onVoteAgainst?: () => void;
}

function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

function formatVotes(votes: number): string {
  if (votes >= 1_000_000) return `${(votes / 1_000_000).toFixed(2)}M`;
  if (votes >= 1_000) return `${(votes / 1_000).toFixed(2)}K`;
  return votes.toLocaleString();
}

function slotToApproxDate(slot: number, currentSlot: number): string {
  const slotDiff = slot - currentSlot;
  const secondsDiff = Math.floor(slotDiff * 0.4);
  const date = new Date(Date.now() + secondsDiff * 1000);
  return date.toLocaleDateString("en-US", {
    month: "short",
    day: "numeric",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const STATUS_STYLES: Record<string, string> = {
  Active: "bg-accent-purple/20 text-accent-purple border-accent-purple/40",
  Passed: "bg-accent-green/20 text-accent-green border-accent-green/40",
  Failed: "bg-accent-red/20 text-accent-red border-accent-red/40",
  Executed: "bg-accent-blue/20 text-accent-blue border-accent-blue/40",
  Cancelled: "bg-muted/20 text-muted border-muted/40",
  Pending: "bg-accent-amber/20 text-accent-amber border-accent-amber/40",
};

export default function ProposalDetail({
  id,
  title,
  description,
  status,
  forVotes,
  againstVotes,
  startSlot,
  endSlot,
  currentSlot,
  timelockEnd,
  proposer,
  voters,
  onVoteFor,
  onVoteAgainst,
}: ProposalDetailProps) {
  const [activeTab, setActiveTab] = useState<"description" | "voters">(
    "description"
  );

  const totalVotes = forVotes + againstVotes;
  const forPercent = totalVotes > 0 ? (forVotes / totalVotes) * 100 : 0;
  const againstPercent = totalVotes > 0 ? (againstVotes / totalVotes) * 100 : 0;
  const isVotingActive = status === "Active" && currentSlot <= endSlot;
  const statusStyle = STATUS_STYLES[status] ?? STATUS_STYLES.Pending;

  return (
    <div className="mx-auto max-w-3xl">
      {/* Header */}
      <div className="mb-6">
        <div className="mb-2 flex items-center gap-3">
          <span className="text-sm font-medium text-muted">
            Proposal #{id}
          </span>
          <span
            className={`rounded-full border px-3 py-1 text-xs font-semibold ${statusStyle}`}
          >
            {status}
          </span>
        </div>
        <h1 className="mb-2 text-2xl font-bold text-foreground">{title}</h1>
        <p className="text-sm text-muted">
          Proposed by{" "}
          <span className="font-mono text-foreground">
            {formatAddress(proposer)}
          </span>
        </p>
      </div>

      {/* Vote breakdown */}
      <div className="mb-6 rounded-xl border border-card-border bg-card-bg p-5">
        <h2 className="mb-4 text-sm font-semibold uppercase tracking-wider text-muted">
          Vote Breakdown
        </h2>

        <div className="mb-4 grid grid-cols-2 gap-4">
          <div className="rounded-lg border border-accent-green/20 bg-accent-green/5 p-4">
            <p className="mb-1 text-sm text-accent-green">For</p>
            <p className="text-2xl font-bold text-accent-green">
              {formatVotes(forVotes)}
            </p>
            <p className="text-sm text-muted">{forPercent.toFixed(1)}%</p>
          </div>
          <div className="rounded-lg border border-accent-red/20 bg-accent-red/5 p-4">
            <p className="mb-1 text-sm text-accent-red">Against</p>
            <p className="text-2xl font-bold text-accent-red">
              {formatVotes(againstVotes)}
            </p>
            <p className="text-sm text-muted">{againstPercent.toFixed(1)}%</p>
          </div>
        </div>

        {/* Vote bar */}
        <div className="mb-4">
          <div className="flex h-3 w-full overflow-hidden rounded-full bg-card-border">
            {totalVotes > 0 && (
              <>
                <div
                  className="h-full bg-accent-green transition-all"
                  style={{ width: `${forPercent}%` }}
                />
                <div
                  className="h-full bg-accent-red transition-all"
                  style={{ width: `${againstPercent}%` }}
                />
              </>
            )}
          </div>
          <p className="mt-1 text-center text-xs text-muted">
            {formatVotes(totalVotes)} total votes
          </p>
        </div>

        {/* Vote buttons */}
        {isVotingActive && (
          <div className="flex gap-3">
            <button
              onClick={onVoteFor}
              className="flex-1 rounded-lg bg-accent-green py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90"
            >
              Vote For
            </button>
            <button
              onClick={onVoteAgainst}
              className="flex-1 rounded-lg bg-accent-red py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90"
            >
              Vote Against
            </button>
          </div>
        )}
      </div>

      {/* Timeline */}
      <div className="mb-6 rounded-xl border border-card-border bg-card-bg p-5">
        <h2 className="mb-4 text-sm font-semibold uppercase tracking-wider text-muted">
          Timeline
        </h2>
        <div className="space-y-3">
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted">Voting Started</span>
            <span className="font-mono text-foreground">
              Slot {startSlot.toLocaleString()} ({slotToApproxDate(startSlot, currentSlot)})
            </span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-muted">Voting Ends</span>
            <span className="font-mono text-foreground">
              Slot {endSlot.toLocaleString()} ({slotToApproxDate(endSlot, currentSlot)})
            </span>
          </div>
          {timelockEnd > 0 && (
            <div className="flex items-center justify-between text-sm">
              <span className="text-muted">Executable After</span>
              <span className="font-mono text-foreground">
                Slot {timelockEnd.toLocaleString()} ({slotToApproxDate(timelockEnd, currentSlot)})
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Tabs: Description / Voters */}
      <div className="rounded-xl border border-card-border bg-card-bg">
        <div className="flex border-b border-card-border">
          <button
            onClick={() => setActiveTab("description")}
            className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === "description"
                ? "border-b-2 border-accent-purple text-accent-purple"
                : "text-muted hover:text-foreground"
            }`}
          >
            Description
          </button>
          <button
            onClick={() => setActiveTab("voters")}
            className={`flex-1 px-4 py-3 text-sm font-medium transition-colors ${
              activeTab === "voters"
                ? "border-b-2 border-accent-purple text-accent-purple"
                : "text-muted hover:text-foreground"
            }`}
          >
            Voters ({voters.length})
          </button>
        </div>

        <div className="p-5">
          {activeTab === "description" && (
            <div className="prose prose-invert max-w-none text-sm leading-relaxed text-foreground/80">
              {description.split("\n").map((paragraph, i) => (
                <p key={i} className="mb-3 last:mb-0">
                  {paragraph}
                </p>
              ))}
            </div>
          )}

          {activeTab === "voters" && (
            <div className="space-y-2">
              {voters.length === 0 && (
                <p className="text-center text-sm text-muted py-6">
                  No votes yet.
                </p>
              )}
              {voters.map((voter, i) => (
                <div
                  key={i}
                  className="flex items-center justify-between rounded-lg border border-card-border px-4 py-3"
                >
                  <span className="font-mono text-sm text-foreground">
                    {formatAddress(voter.address)}
                  </span>
                  <div className="flex items-center gap-3">
                    <span className="text-sm text-muted">
                      {formatVotes(voter.weight)} SCLONE
                    </span>
                    <span
                      className={`rounded-full px-2 py-0.5 text-xs font-semibold ${
                        voter.support
                          ? "bg-accent-green/20 text-accent-green"
                          : "bg-accent-red/20 text-accent-red"
                      }`}
                    >
                      {voter.support ? "For" : "Against"}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
