import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import Link from "next/link";
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
  title: "SolMart — NFT Marketplace",
  description:
    "Discover, collect, and trade NFTs on the SolClone blockchain. Premium marketplace with auctions, collections, and royalty enforcement.",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-background text-foreground">
        {/* ── Navigation ── */}
        <header className="sticky top-0 z-50 glass-card border-b border-card-border/40">
          <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
            {/* Logo */}
            <Link href="/" className="flex items-center gap-2">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-gradient-to-br from-accent-purple to-accent-green">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="white"
                  strokeWidth="2.5"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />
                </svg>
              </div>
              <span className="text-xl font-bold gradient-text">SolMart</span>
            </Link>

            {/* Nav links */}
            <nav className="hidden items-center gap-8 md:flex">
              <Link
                href="/"
                className="text-sm font-medium text-muted transition-colors hover:text-foreground"
              >
                Explore
              </Link>
              <Link
                href="/collections"
                className="text-sm font-medium text-muted transition-colors hover:text-foreground"
              >
                Collections
              </Link>
              <Link
                href="/create"
                className="text-sm font-medium text-muted transition-colors hover:text-foreground"
              >
                Create
              </Link>
            </nav>

            {/* Wallet button */}
            <button className="flex items-center gap-2 rounded-xl bg-gradient-to-r from-accent-purple to-accent-green px-5 py-2.5 text-sm font-semibold text-white transition-opacity hover:opacity-90">
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <rect x="2" y="5" width="20" height="14" rx="2" />
                <line x1="2" y1="10" x2="22" y2="10" />
              </svg>
              Connect Wallet
            </button>
          </div>
        </header>

        {/* ── Main content ── */}
        <main className="flex-1">{children}</main>

        {/* ── Footer ── */}
        <footer className="border-t border-card-border/30 bg-surface/50">
          <div className="mx-auto flex max-w-7xl flex-col items-center justify-between gap-4 px-4 py-8 sm:flex-row sm:px-6 lg:px-8">
            <p className="text-sm text-muted">
              SolMart &mdash; Built on SolClone
            </p>
            <div className="flex items-center gap-6">
              <Link
                href="/"
                className="text-xs text-muted transition-colors hover:text-foreground"
              >
                Marketplace
              </Link>
              <Link
                href="/collections"
                className="text-xs text-muted transition-colors hover:text-foreground"
              >
                Collections
              </Link>
              <Link
                href="/create"
                className="text-xs text-muted transition-colors hover:text-foreground"
              >
                Create
              </Link>
            </div>
          </div>
        </footer>
      </body>
    </html>
  );
}
