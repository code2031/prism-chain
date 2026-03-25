import { NextRequest, NextResponse } from "next/server";
import { checkLimit } from "@/lib/rate-limiter";

const BASE58_REGEX = /^[1-9A-HJ-NP-Za-km-z]{32,44}$/;

const NETWORK_URLS: Record<string, string> = {
  devnet: "http://localhost:8899",
  testnet: "http://localhost:8799",
};

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { address, amount, network } = body;

    // Validate required fields
    if (!address || !amount || !network) {
      return NextResponse.json(
        { error: "Missing required fields: address, amount, network" },
        { status: 400 }
      );
    }

    // Validate network
    if (!NETWORK_URLS[network]) {
      return NextResponse.json(
        { error: "Invalid network. Must be 'devnet' or 'testnet'" },
        { status: 400 }
      );
    }

    // Validate Base58 address
    if (!BASE58_REGEX.test(address)) {
      return NextResponse.json(
        { error: "Invalid wallet address. Must be a valid Base58 address." },
        { status: 400 }
      );
    }

    // Validate amount
    const parsedAmount = Number(amount);
    if (![1, 2, 5].includes(parsedAmount)) {
      return NextResponse.json(
        { error: "Invalid amount. Must be 1, 2, or 5 SCLONE." },
        { status: 400 }
      );
    }

    // Rate limiting
    const ip =
      request.headers.get("x-forwarded-for")?.split(",")[0]?.trim() ||
      request.headers.get("x-real-ip") ||
      "unknown";

    if (!checkLimit(ip, address)) {
      return NextResponse.json(
        {
          error:
            "Rate limit exceeded. Max 10 requests/hour per IP and 5 requests/hour per address.",
        },
        { status: 429 }
      );
    }

    // Convert SCLONE to lamports (1 SCLONE = 1_000_000_000 lamports)
    const lamports = parsedAmount * 1_000_000_000;

    // Call JSON-RPC requestAirdrop
    const rpcUrl = NETWORK_URLS[network];
    const rpcResponse = await fetch(rpcUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: 1,
        method: "requestAirdrop",
        params: [address, lamports],
      }),
    });

    const rpcResult = await rpcResponse.json();

    if (rpcResult.error) {
      return NextResponse.json(
        {
          error:
            rpcResult.error.message || "Airdrop request failed on the network.",
        },
        { status: 502 }
      );
    }

    return NextResponse.json({
      success: true,
      signature: rpcResult.result,
      amount: parsedAmount,
      network,
    });
  } catch (error: unknown) {
    const message =
      error instanceof Error ? error.message : "Internal server error";
    return NextResponse.json({ error: message }, { status: 500 });
  }
}
