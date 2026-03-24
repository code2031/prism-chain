import * as nacl from 'tweetnacl';
import bs58 from 'bs58';
import * as fs from 'fs';
import * as path from 'path';
import { loadConfig, ensureConfigDir, getDefaultKeypairPath } from './config';
import { KeypairData } from '../types';

/**
 * Generate a new Ed25519 keypair.
 */
export function generateKeypair(): KeypairData {
  const kp = nacl.sign.keyPair();
  return {
    publicKey: kp.publicKey,
    secretKey: kp.secretKey,
  };
}

/**
 * Restore a keypair from a 64-byte secret key.
 */
export function keypairFromSecretKey(secretKey: Uint8Array): KeypairData {
  const kp = nacl.sign.keyPair.fromSecretKey(secretKey);
  return {
    publicKey: kp.publicKey,
    secretKey: kp.secretKey,
  };
}

/**
 * Get the public key (base58 address) from a keypair.
 */
export function getPublicKeyBase58(keypair: KeypairData): string {
  return bs58.encode(keypair.publicKey);
}

/**
 * Encode a public key (Uint8Array) to base58.
 */
export function publicKeyToBase58(pubkey: Uint8Array): string {
  return bs58.encode(pubkey);
}

/**
 * Decode a base58 address to Uint8Array.
 */
export function base58ToPublicKey(address: string): Uint8Array {
  return bs58.decode(address);
}

/**
 * Save a keypair to a JSON file (Solana-compatible format: array of 64 bytes).
 */
export function saveKeypairToFile(keypair: KeypairData, filePath: string): void {
  const dir = path.dirname(filePath);
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true, mode: 0o700 });
  }
  const arr = Array.from(keypair.secretKey);
  fs.writeFileSync(filePath, JSON.stringify(arr), { mode: 0o600 });
}

/**
 * Load a keypair from a JSON file (Solana-compatible format).
 */
export function loadKeypairFromFile(filePath: string): KeypairData {
  const resolved = resolvePath(filePath);

  if (!fs.existsSync(resolved)) {
    throw new Error(`Keypair file not found: ${resolved}`);
  }

  const raw = fs.readFileSync(resolved, 'utf-8');
  let parsed: number[];

  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new Error(`Invalid keypair file format: ${resolved}`);
  }

  if (!Array.isArray(parsed) || parsed.length !== 64) {
    throw new Error(
      `Invalid keypair file: expected an array of 64 bytes, got ${
        Array.isArray(parsed) ? parsed.length : typeof parsed
      }`
    );
  }

  const secretKey = new Uint8Array(parsed);
  return keypairFromSecretKey(secretKey);
}

/**
 * Load the default keypair from config.
 */
export function loadDefaultKeypair(): KeypairData {
  const config = loadConfig();
  return loadKeypairFromFile(config.keypair_path);
}

/**
 * Check if the default keypair exists.
 */
export function defaultKeypairExists(): boolean {
  const config = loadConfig();
  const resolved = resolvePath(config.keypair_path);
  return fs.existsSync(resolved);
}

/**
 * Sign a message with a keypair.
 */
export function signMessage(message: Uint8Array, secretKey: Uint8Array): Uint8Array {
  return nacl.sign.detached(message, secretKey);
}

/**
 * Verify a signature.
 */
export function verifySignature(
  message: Uint8Array,
  signature: Uint8Array,
  publicKey: Uint8Array
): boolean {
  return nacl.sign.detached.verify(message, signature, publicKey);
}

/**
 * Resolve a path, expanding ~ to home directory.
 */
function resolvePath(p: string): string {
  if (p.startsWith('~')) {
    return path.join(process.env.HOME || '~', p.slice(1));
  }
  return path.resolve(p);
}

/**
 * Generate the default keypair if it doesn't exist. Returns the keypair.
 */
export function ensureDefaultKeypair(): KeypairData {
  ensureConfigDir();
  const defaultPath = getDefaultKeypairPath();

  if (fs.existsSync(defaultPath)) {
    return loadKeypairFromFile(defaultPath);
  }

  const keypair = generateKeypair();
  saveKeypairToFile(keypair, defaultPath);
  return keypair;
}
