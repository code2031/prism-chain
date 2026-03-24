import chalk from 'chalk';

// ─── Color Helpers ──────────────────────────────────────────────────────────

export const colors = {
  primary: chalk.cyan,
  success: chalk.green,
  warning: chalk.yellow,
  error: chalk.red,
  info: chalk.blue,
  muted: chalk.gray,
  bold: chalk.bold,
  address: chalk.cyan.bold,
  amount: chalk.green.bold,
  label: chalk.white.bold,
  value: chalk.white,
};

// ─── Formatting Helpers ─────────────────────────────────────────────────────

/**
 * Format lamports to SOL with proper decimal places.
 */
export function lamportsToSol(lamports: number): string {
  return (lamports / 1_000_000_000).toFixed(9);
}

/**
 * Format SOL amount for display.
 */
export function formatSol(lamports: number): string {
  const sol = lamports / 1_000_000_000;
  return `${colors.amount(sol.toFixed(9))} SOL`;
}

/**
 * Truncate a long string (like an address or signature) for display.
 */
export function truncate(str: string, maxLen: number = 20): string {
  if (str.length <= maxLen) return str;
  const half = Math.floor((maxLen - 3) / 2);
  return `${str.slice(0, half)}...${str.slice(-half)}`;
}

/**
 * Format a Unix timestamp to a human-readable date.
 */
export function formatTimestamp(timestamp: number | null): string {
  if (timestamp === null) return colors.muted('N/A');
  return new Date(timestamp * 1000).toLocaleString();
}

// ─── Output Helpers ─────────────────────────────────────────────────────────

/**
 * Print a success message.
 */
export function printSuccess(message: string): void {
  console.log(colors.success('  \u2714 ') + message);
}

/**
 * Print an error message.
 */
export function printError(message: string): void {
  console.error(colors.error('  \u2718 Error: ') + message);
}

/**
 * Print a warning message.
 */
export function printWarning(message: string): void {
  console.log(colors.warning('  \u26A0 Warning: ') + message);
}

/**
 * Print an info message.
 */
export function printInfo(message: string): void {
  console.log(colors.info('  \u2139 ') + message);
}

/**
 * Print a labeled key-value pair.
 */
export function printKeyValue(label: string, value: string, indent: number = 2): void {
  const pad = ' '.repeat(indent);
  console.log(`${pad}${colors.label(label + ':')} ${colors.value(value)}`);
}

/**
 * Print a divider line.
 */
export function printDivider(char: string = '\u2500', length: number = 60): void {
  console.log(colors.muted(char.repeat(length)));
}

/**
 * Print a header / title.
 */
export function printHeader(title: string): void {
  console.log();
  printDivider();
  console.log(colors.bold(`  ${title}`));
  printDivider();
}

/**
 * Print a table of data.
 */
export function printTable(
  headers: string[],
  rows: string[][],
  columnWidths?: number[]
): void {
  const widths = columnWidths || headers.map((h, i) => {
    const maxRow = rows.reduce((max, row) => Math.max(max, (row[i] || '').length), 0);
    return Math.max(h.length, maxRow) + 2;
  });

  // Header row
  const headerLine = headers
    .map((h, i) => colors.label(h.padEnd(widths[i])))
    .join('  ');
  console.log(`  ${headerLine}`);
  console.log(`  ${widths.map(w => '\u2500'.repeat(w)).join('  ')}`);

  // Data rows
  for (const row of rows) {
    const line = row
      .map((cell, i) => cell.padEnd(widths[i]))
      .join('  ');
    console.log(`  ${line}`);
  }
}

/**
 * Print a blank line.
 */
export function println(): void {
  console.log();
}

/**
 * Format a large number with commas.
 */
export function formatNumber(num: number): string {
  return num.toLocaleString();
}

/**
 * Create a boxed message.
 */
export function printBox(lines: string[]): void {
  const maxLen = lines.reduce((max, l) => Math.max(max, stripAnsi(l).length), 0);
  const border = '\u250C' + '\u2500'.repeat(maxLen + 2) + '\u2510';
  const bottom = '\u2514' + '\u2500'.repeat(maxLen + 2) + '\u2518';

  console.log(colors.muted(border));
  for (const line of lines) {
    const padding = ' '.repeat(maxLen - stripAnsi(line).length);
    console.log(colors.muted('\u2502') + ` ${line}${padding} ` + colors.muted('\u2502'));
  }
  console.log(colors.muted(bottom));
}

/**
 * Strip ANSI escape codes from a string for length calculation.
 */
function stripAnsi(str: string): string {
  // eslint-disable-next-line no-control-regex
  return str.replace(/\u001b\[[0-9;]*m/g, '');
}
