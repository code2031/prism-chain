import * as fs from 'fs';
import * as path from 'path';
import * as YAML from 'yaml';
import { SolCloneConfig, Commitment } from '../types';

const CONFIG_DIR = path.join(process.env.HOME || '~', '.solclone');
const CONFIG_FILE = path.join(CONFIG_DIR, 'config.yml');
const DEFAULT_KEYPAIR = path.join(CONFIG_DIR, 'id.json');

const DEFAULT_CONFIG: SolCloneConfig = {
  rpc_url: 'http://localhost:8899',
  keypair_path: DEFAULT_KEYPAIR,
  commitment: 'confirmed',
};

/**
 * Ensure the config directory exists.
 */
export function ensureConfigDir(): void {
  if (!fs.existsSync(CONFIG_DIR)) {
    fs.mkdirSync(CONFIG_DIR, { recursive: true, mode: 0o700 });
  }
}

/**
 * Load the configuration from disk, falling back to defaults.
 */
export function loadConfig(): SolCloneConfig {
  ensureConfigDir();

  if (!fs.existsSync(CONFIG_FILE)) {
    saveConfig(DEFAULT_CONFIG);
    return { ...DEFAULT_CONFIG };
  }

  try {
    const raw = fs.readFileSync(CONFIG_FILE, 'utf-8');
    const parsed = YAML.parse(raw) as Partial<SolCloneConfig>;
    return {
      rpc_url: parsed.rpc_url || DEFAULT_CONFIG.rpc_url,
      keypair_path: parsed.keypair_path || DEFAULT_CONFIG.keypair_path,
      commitment: parsed.commitment || DEFAULT_CONFIG.commitment,
    };
  } catch {
    return { ...DEFAULT_CONFIG };
  }
}

/**
 * Save the configuration to disk.
 */
export function saveConfig(config: SolCloneConfig): void {
  ensureConfigDir();
  const yamlStr = YAML.stringify(config);
  fs.writeFileSync(CONFIG_FILE, yamlStr, { mode: 0o600 });
}

/**
 * Update specific fields in the configuration.
 */
export function updateConfig(updates: Partial<SolCloneConfig>): SolCloneConfig {
  const config = loadConfig();

  if (updates.rpc_url !== undefined) {
    config.rpc_url = updates.rpc_url;
  }
  if (updates.keypair_path !== undefined) {
    config.keypair_path = resolveKeypairPath(updates.keypair_path);
  }
  if (updates.commitment !== undefined) {
    config.commitment = updates.commitment;
  }

  saveConfig(config);
  return config;
}

/**
 * Resolve a keypair path, expanding ~ to home directory.
 */
export function resolveKeypairPath(p: string): string {
  if (p.startsWith('~')) {
    return path.join(process.env.HOME || '~', p.slice(1));
  }
  return path.resolve(p);
}

/**
 * Get the config directory path.
 */
export function getConfigDir(): string {
  return CONFIG_DIR;
}

/**
 * Get the config file path.
 */
export function getConfigFile(): string {
  return CONFIG_FILE;
}

/**
 * Get the default keypair path.
 */
export function getDefaultKeypairPath(): string {
  return DEFAULT_KEYPAIR;
}

/**
 * Validate a commitment level string.
 */
export function isValidCommitment(value: string): value is Commitment {
  return ['processed', 'confirmed', 'finalized'].includes(value);
}
