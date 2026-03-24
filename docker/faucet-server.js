/**
 * SolClone Faucet Server
 * Provides SOL airdrops for devnet and testnet
 */

const express = require("express");
const http = require("http");

const app = express();
app.use(express.json());

const CLUSTER = process.env.CLUSTER || "devnet";
const RPC_URL = process.env.RPC_URL || "http://validator:8899";
const PORT = process.env.PORT || 9900;

const LIMITS = {
  devnet: { maxAmount: 5, ratePerHour: 10 },
  testnet: { maxAmount: 1, ratePerHour: 3 },
  mainnet: { maxAmount: 0, ratePerHour: 0 },
};

const rateTracker = new Map();

function rpcCall(method, params) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method,
      params,
    });

    const url = new URL(RPC_URL);
    const req = http.request(
      {
        hostname: url.hostname,
        port: url.port,
        path: url.pathname,
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(data),
        },
      },
      (res) => {
        let body = "";
        res.on("data", (chunk) => (body += chunk));
        res.on("end", () => {
          try {
            resolve(JSON.parse(body));
          } catch (e) {
            reject(new Error("Invalid JSON from RPC"));
          }
        });
      }
    );
    req.on("error", reject);
    req.write(data);
    req.end();
  });
}

function checkRateLimit(ip) {
  const limit = LIMITS[CLUSTER] || LIMITS.devnet;
  const now = Date.now();
  const hourAgo = now - 3600000;

  if (!rateTracker.has(ip)) {
    rateTracker.set(ip, []);
  }

  const requests = rateTracker.get(ip).filter((t) => t > hourAgo);
  rateTracker.set(ip, requests);

  if (requests.length >= limit.ratePerHour) {
    return false;
  }

  requests.push(now);
  return true;
}

// Health check
app.get("/health", (req, res) => {
  res.json({ status: "ok", cluster: CLUSTER });
});

// Airdrop endpoint
app.post("/airdrop", async (req, res) => {
  const limit = LIMITS[CLUSTER] || LIMITS.devnet;

  if (limit.maxAmount === 0) {
    return res.status(403).json({
      error: "Airdrops are not available on mainnet",
    });
  }

  const { address, amount } = req.body;

  if (!address) {
    return res.status(400).json({ error: "Missing 'address' field" });
  }

  const solAmount = parseFloat(amount) || 1;

  if (solAmount > limit.maxAmount) {
    return res.status(400).json({
      error: `Max airdrop is ${limit.maxAmount} SOL on ${CLUSTER}`,
    });
  }

  const ip = req.ip || req.connection.remoteAddress;
  if (!checkRateLimit(ip)) {
    return res.status(429).json({
      error: `Rate limited. Max ${limit.ratePerHour} requests per hour on ${CLUSTER}`,
    });
  }

  try {
    const lamports = Math.floor(solAmount * 1e9);
    const result = await rpcCall("requestAirdrop", [address, lamports]);

    if (result.error) {
      return res.status(500).json({ error: result.error.message });
    }

    res.json({
      success: true,
      signature: result.result,
      amount: solAmount,
      cluster: CLUSTER,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

// Get balance
app.get("/balance/:address", async (req, res) => {
  try {
    const result = await rpcCall("getBalance", [req.params.address]);
    if (result.error) {
      return res.status(500).json({ error: result.error.message });
    }
    res.json({
      address: req.params.address,
      lamports: result.result.value,
      sol: result.result.value / 1e9,
    });
  } catch (err) {
    res.status(500).json({ error: err.message });
  }
});

app.listen(PORT, "0.0.0.0", () => {
  console.log(`SolClone Faucet (${CLUSTER}) running on port ${PORT}`);
  console.log(`RPC: ${RPC_URL}`);
  console.log(`Max airdrop: ${LIMITS[CLUSTER]?.maxAmount || 0} SOL`);
});
