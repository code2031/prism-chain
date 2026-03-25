"use client";

interface ProposalCardProps {
  id: number;
  title: string;
  status: "Active" | "Passed" | "Failed" | "Executed" | "Cancelled" | "Pending";
  forVotes: number;
  againstVotes: number;
  endSlot: number;
  currentSlot: number;
  proposer: string;
}

const STATUS_STYLES: Record<string, string> = {
  Active: "bg-accent-purple/20 text-accent-purple border-accent-purple/40",
  Passed: "bg-accent-green/20 text-accent-green border-accent-green/40",
  Failed: "bg-accent-red/20 text-accent-red border-accent-red/40",
  Executed: "bg-accent-blue/20 text-accent-blue border-accent-blue/40",
  Cancelled: "bg-muted/20 text-muted border-muted/40",
  Pending: "bg-accent-amber/20 text-accent-amber border-accent-amber/40",
};

function formatAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

function formatVotes(votes: number): string {
  if (votes >= 1_000_000) return `${(votes / 1_000_000).toFixed(1)}M`;
  if (votes >= 1_000) return `${(votes / 1_000).toFixed(1)}K`;
  return votes.toLocaleString();
}

function slotsToTimeRemaining(slotsRemaining: number): string {
  if (slotsRemaining <= 0) return "Ended";
  const seconds = Math.floor(slotsRemaining * 0.4);
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h remaining`;
  if (hours > 0) return `${hours}h ${minutes}m remaining`;
  return `${minutes}m remaining`;
}

export default function ProposalCard({
  id,
  title,
  status,
  forVotes,
  againstVotes,
  endSlot,
  currentSlot,
  proposer,
}: ProposalCardProps) {
  const totalVotes = forVotes + againstVotes;
  const forPercent = totalVotes > 0 ? (forVotes / totalVotes) * 100 : 0;
  const againstPercent = totalVotes > 0 ? (againstVotes / totalVotes) * 100 : 0;
  const slotsRemaining = endSlot - currentSlot;
  const statusStyle = STATUS_STYLES[status] ?? STATUS_STYLES.Pending;

  return (
    <div className="group rounded-xl border border-card-border bg-card-bg p-5 transition-all hover:border-accent-purple/40 hover:shadow-lg hover:shadow-accent-purple/5">
      <div className="mb-3 flex items-start justify-between gap-3">
        <div className="flex-1">
          <p className="mb-1 text-xs font-medium text-muted">
            Proposal #{id}
          </p>
          <h3 className="text-base font-semibold leading-snug text-foreground group-hover:text-accent-purple transition-colors">
            {title}
          </h3>
        </div>
        <span
          className={`shrink-0 rounded-full border px-3 py-1 text-xs font-semibold ${statusStyle}`}
        >
          {status}
        </span>
      </div>

      {/* Vote bar */}
      <div className="mb-3">
        <div className="mb-1.5 flex items-center justify-between text-xs">
          <span className="text-accent-green">
            For: {formatVotes(forVotes)} ({forPercent.toFixed(1)}%)
          </span>
          <span className="text-accent-red">
            Against: {formatVotes(againstVotes)} ({againstPercent.toFixed(1)}%)
          </span>
        </div>
        <div className="flex h-2 w-full overflow-hidden rounded-full bg-card-border">
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
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between text-xs text-muted">
        <span>by {formatAddress(proposer)}</span>
        <span>
          {status === "Active"
            ? slotsToTimeRemaining(slotsRemaining)
            : status}
        </span>
      </div>
    </div>
  );
}
