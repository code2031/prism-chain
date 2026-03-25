import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "SolSwap | SolClone DEX",
  description:
    "Trade tokens instantly on SolClone with near-zero fees. The fastest AMM DEX on the SolClone blockchain.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased dark`}
    >
      <body className="flex min-h-full flex-col bg-[#0b0b14] text-gray-100">
        {/* Header */}
        <header className="sticky top-0 z-40 border-b border-white/[0.04] bg-[#0b0b14]/80 backdrop-blur-xl">
          <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-4 sm:px-6">
            {/* Logo */}
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-gradient-to-br from-purple-600 to-green-500 shadow-lg shadow-purple-600/20">
                <svg className="h-5 w-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4" />
                </svg>
              </div>
              <span className="text-xl font-bold">
                <span className="gradient-text">Sol</span>
                <span className="text-white">Swap</span>
              </span>
            </div>

            {/* Nav links */}
            <nav className="hidden items-center gap-1 sm:flex">
              <a
                href="#"
                className="rounded-lg px-3 py-2 text-sm font-medium text-gray-400 transition hover:bg-white/5 hover:text-white"
              >
                Trade
              </a>
              <a
                href="#"
                className="rounded-lg px-3 py-2 text-sm font-medium text-gray-400 transition hover:bg-white/5 hover:text-white"
              >
                Pools
              </a>
              <a
                href="#"
                className="rounded-lg px-3 py-2 text-sm font-medium text-gray-400 transition hover:bg-white/5 hover:text-white"
              >
                Charts
              </a>
            </nav>

            {/* Connect wallet button */}
            <button
              type="button"
              className="rounded-xl bg-gradient-to-r from-purple-600 to-green-500 px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-purple-600/20 transition hover:shadow-purple-600/40 hover:brightness-110"
            >
              Connect Wallet
            </button>
          </div>
        </header>

        {/* Main content */}
        <main className="flex flex-1 flex-col">{children}</main>

        {/* Footer */}
        <footer className="border-t border-white/[0.04] py-6">
          <div className="mx-auto flex max-w-6xl flex-col items-center justify-between gap-4 px-4 sm:flex-row sm:px-6">
            <div className="flex items-center gap-2 text-sm text-gray-500">
              <span className="gradient-text font-semibold">SolSwap</span>
              <span>by SolClone</span>
            </div>
            <div className="flex items-center gap-4 text-xs text-gray-600">
              <a href="#" className="transition hover:text-gray-400">Docs</a>
              <a href="#" className="transition hover:text-gray-400">GitHub</a>
              <a href="#" className="transition hover:text-gray-400">Discord</a>
              <a href="#" className="transition hover:text-gray-400">Terms</a>
            </div>
          </div>
        </footer>
      </body>
    </html>
  );
}
