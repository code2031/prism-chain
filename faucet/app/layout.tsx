import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "SolClone Faucet — Get Free Devnet & Testnet Tokens",
  description:
    "Request free SCLONE tokens on devnet or testnet for development and testing on the SolClone blockchain.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${inter.variable} antialiased`}>
      <body className="min-h-screen bg-[#0F0B2E] text-gray-100">{children}</body>
    </html>
  );
}
