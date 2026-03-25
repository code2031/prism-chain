import FaucetForm from "@/components/faucet-form";

export default function Home() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center px-4 py-16">
      {/* Logo Area */}
      <div className="mb-8 flex items-center gap-3">
        <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-[#9945FF]/20 border border-[#9945FF]/30">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="28"
            height="28"
            viewBox="0 0 24 24"
            fill="none"
            stroke="#9945FF"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M12 2L2 7l10 5 10-5-10-5z" />
            <path d="M2 17l10 5 10-5" />
            <path d="M2 12l10 5 10-5" />
          </svg>
        </div>
        <span className="text-xl font-bold text-white tracking-tight">
          Sol<span className="text-[#9945FF]">Clone</span>
        </span>
      </div>

      {/* Title & Description */}
      <div className="mb-10 text-center">
        <h1 className="mb-3 text-4xl font-bold text-white sm:text-5xl">
          SolClone <span className="text-[#9945FF]">Faucet</span>
        </h1>
        <p className="mx-auto max-w-md text-base text-gray-400 leading-relaxed">
          Get free SCLONE tokens on devnet or testnet for development and testing.
          Tokens have no real-world value.
        </p>
      </div>

      {/* Card */}
      <div className="w-full max-w-lg rounded-2xl border border-gray-800 bg-[#13103a] p-6 shadow-2xl shadow-[#9945FF]/5 sm:p-8">
        <FaucetForm />
      </div>

      {/* Info Pills */}
      <div className="mt-8 flex flex-wrap justify-center gap-3 text-xs text-gray-500">
        <span className="rounded-full border border-gray-800 bg-[#1A1640] px-3 py-1.5">
          Max 5 requests/hr per wallet
        </span>
        <span className="rounded-full border border-gray-800 bg-[#1A1640] px-3 py-1.5">
          Max 10 requests/hr per IP
        </span>
        <span className="rounded-full border border-gray-800 bg-[#1A1640] px-3 py-1.5">
          Devnet &amp; Testnet only
        </span>
      </div>

      {/* Footer */}
      <footer className="mt-16 text-center text-xs text-gray-600">
        <p>
          Built with SolClone &mdash; A Solana-compatible blockchain for learning and development.
        </p>
        <p className="mt-1 text-gray-700">
          Faucet tokens are for testing purposes only. They hold no monetary value.
        </p>
      </footer>
    </div>
  );
}
