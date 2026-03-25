"use client";

import { useState } from "react";

type Network = "devnet" | "testnet";

interface AirdropResult {
  success: boolean;
  signature?: string;
  amount?: number;
  network?: string;
  error?: string;
}

export default function FaucetForm() {
  const [address, setAddress] = useState("");
  const [amount, setAmount] = useState<number>(1);
  const [network, setNetwork] = useState<Network>("devnet");
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<AirdropResult | null>(null);

  async function handlePaste() {
    try {
      const text = await navigator.clipboard.readText();
      setAddress(text.trim());
    } catch {
      // Clipboard access denied - user can paste manually
    }
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setLoading(true);
    setResult(null);

    try {
      const res = await fetch("/api/airdrop", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ address, amount, network }),
      });

      const data: AirdropResult = await res.json();

      if (!res.ok) {
        setResult({ success: false, error: data.error || "Request failed" });
      } else {
        setResult(data);
      }
    } catch {
      setResult({ success: false, error: "Network error. Is the validator running?" });
    } finally {
      setLoading(false);
    }
  }

  return (
    <form onSubmit={handleSubmit} className="w-full max-w-lg mx-auto space-y-6">
      {/* Wallet Address */}
      <div className="space-y-2">
        <label htmlFor="address" className="block text-sm font-medium text-gray-300">
          Wallet Address
        </label>
        <div className="flex gap-2">
          <input
            id="address"
            type="text"
            value={address}
            onChange={(e) => setAddress(e.target.value)}
            placeholder="Enter your SolClone wallet address"
            required
            className="flex-1 rounded-lg border border-gray-700 bg-[#1A1640] px-4 py-3 text-sm text-white placeholder-gray-500 outline-none transition-colors focus:border-[#9945FF] focus:ring-1 focus:ring-[#9945FF]"
          />
          <button
            type="button"
            onClick={handlePaste}
            className="rounded-lg border border-gray-700 bg-[#1A1640] px-4 py-3 text-sm text-gray-300 transition-colors hover:border-[#9945FF] hover:text-white"
            title="Paste from clipboard"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
          </button>
        </div>
      </div>

      {/* Amount Selection */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-300">Amount (SCLONE)</label>
        <div className="flex gap-3">
          {[1, 2, 5].map((val) => (
            <label
              key={val}
              className={`flex-1 cursor-pointer rounded-lg border px-4 py-3 text-center text-sm font-semibold transition-all ${
                amount === val
                  ? "border-[#9945FF] bg-[#9945FF]/20 text-[#9945FF]"
                  : "border-gray-700 bg-[#1A1640] text-gray-400 hover:border-gray-500"
              }`}
            >
              <input
                type="radio"
                name="amount"
                value={val}
                checked={amount === val}
                onChange={() => setAmount(val)}
                className="sr-only"
              />
              {val} SCLONE
            </label>
          ))}
        </div>
      </div>

      {/* Network Selection */}
      <div className="space-y-2">
        <label className="block text-sm font-medium text-gray-300">Network</label>
        <div className="flex gap-3">
          {(["devnet", "testnet"] as const).map((net) => (
            <label
              key={net}
              className={`flex-1 cursor-pointer rounded-lg border px-4 py-3 text-center text-sm font-semibold capitalize transition-all ${
                network === net
                  ? "border-[#9945FF] bg-[#9945FF]/20 text-[#9945FF]"
                  : "border-gray-700 bg-[#1A1640] text-gray-400 hover:border-gray-500"
              }`}
            >
              <input
                type="radio"
                name="network"
                value={net}
                checked={network === net}
                onChange={() => setNetwork(net)}
                className="sr-only"
              />
              {net}
            </label>
          ))}
        </div>
      </div>

      {/* Submit Button */}
      <button
        type="submit"
        disabled={loading || !address}
        className="w-full rounded-lg bg-[#9945FF] px-6 py-3.5 text-sm font-semibold text-white transition-all hover:bg-[#8A3AEE] disabled:cursor-not-allowed disabled:opacity-50"
      >
        {loading ? (
          <span className="inline-flex items-center gap-2">
            <svg
              className="h-4 w-4 animate-spin"
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
            Requesting Airdrop...
          </span>
        ) : (
          "Request Airdrop"
        )}
      </button>

      {/* Result Display */}
      {result && (
        <div
          className={`rounded-lg border p-4 text-sm ${
            result.success
              ? "border-[#14F195]/30 bg-[#14F195]/10 text-[#14F195]"
              : "border-red-500/30 bg-red-500/10 text-red-400"
          }`}
        >
          {result.success ? (
            <div className="space-y-2">
              <div className="flex items-center gap-2 font-semibold">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="18"
                  height="18"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                  <polyline points="22 4 12 14.01 9 11.01" />
                </svg>
                Airdrop Successful!
              </div>
              <p className="text-gray-300">
                {result.amount} SCLONE sent on {result.network}
              </p>
              {result.signature && (
                <div className="mt-2">
                  <p className="text-xs text-gray-400">Transaction Signature:</p>
                  <a
                    href={`#sig=${result.signature}`}
                    className="mt-1 block break-all rounded bg-[#0F0B2E] px-3 py-2 font-mono text-xs text-[#14F195] hover:underline"
                  >
                    {result.signature}
                  </a>
                </div>
              )}
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <circle cx="12" cy="12" r="10" />
                <line x1="15" y1="9" x2="9" y2="15" />
                <line x1="9" y1="9" x2="15" y2="15" />
              </svg>
              {result.error}
            </div>
          )}
        </div>
      )}
    </form>
  );
}
