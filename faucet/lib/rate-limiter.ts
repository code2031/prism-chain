interface RateLimitEntry {
  timestamps: number[];
}

const ipLimits = new Map<string, RateLimitEntry>();
const addressLimits = new Map<string, RateLimitEntry>();

const IP_MAX_REQUESTS = 10;
const ADDRESS_MAX_REQUESTS = 5;
const WINDOW_MS = 60 * 60 * 1000; // 1 hour

function pruneOldEntries(entry: RateLimitEntry): void {
  const cutoff = Date.now() - WINDOW_MS;
  entry.timestamps = entry.timestamps.filter((t) => t > cutoff);
}

function isWithinLimit(
  map: Map<string, RateLimitEntry>,
  key: string,
  maxRequests: number
): boolean {
  let entry = map.get(key);
  if (!entry) {
    entry = { timestamps: [] };
    map.set(key, entry);
  }
  pruneOldEntries(entry);
  if (entry.timestamps.length >= maxRequests) {
    return false;
  }
  entry.timestamps.push(Date.now());
  return true;
}

export function checkLimit(ip: string, address: string): boolean {
  const ipOk = isWithinLimit(ipLimits, ip, IP_MAX_REQUESTS);
  if (!ipOk) return false;

  const addrOk = isWithinLimit(addressLimits, address, ADDRESS_MAX_REQUESTS);
  if (!addrOk) return false;

  return true;
}
