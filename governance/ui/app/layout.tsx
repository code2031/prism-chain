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
  title: "SolClone DAO",
  description: "Decentralized governance for the SolClone network",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col bg-background text-foreground">
        <header className="sticky top-0 z-50 border-b border-card-border bg-background/80 backdrop-blur-md">
          <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-4 sm:px-6 lg:px-8">
            <div className="flex items-center gap-3">
              <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-accent-purple">
                <svg
                  width="20"
                  height="20"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  className="text-white"
                >
                  <path d="M12 2L2 7l10 5 10-5-10-5z" />
                  <path d="M2 17l10 5 10-5" />
                  <path d="M2 12l10 5 10-5" />
                </svg>
              </div>
              <span className="text-xl font-bold tracking-tight">
                SolClone DAO
              </span>
            </div>
            <nav className="hidden items-center gap-6 text-sm font-medium sm:flex">
              <a
                href="/"
                className="text-foreground transition-colors hover:text-accent-purple"
              >
                Dashboard
              </a>
              <a
                href="#proposals"
                className="text-muted transition-colors hover:text-accent-purple"
              >
                Proposals
              </a>
              <a
                href="#treasury"
                className="text-muted transition-colors hover:text-accent-purple"
              >
                Treasury
              </a>
            </nav>
            <button className="rounded-lg border border-card-border bg-card-bg px-4 py-2 text-sm font-medium transition-colors hover:border-accent-purple hover:text-accent-purple">
              Connect Wallet
            </button>
          </div>
        </header>
        <main className="flex-1">{children}</main>
        <footer className="border-t border-card-border py-6 text-center text-sm text-muted">
          SolClone DAO Governance v0.1.0
        </footer>
      </body>
    </html>
  );
}
