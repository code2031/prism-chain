#!/usr/bin/env node

import { Command } from 'commander';
import chalk from 'chalk';
import { registerKeygenCommand } from './commands/keygen';
import { registerBalanceCommand } from './commands/balance';
import { registerTransferCommand } from './commands/transfer';
import { registerAirdropCommand } from './commands/airdrop';
import { registerTokenCommand } from './commands/token';
import { registerStakeCommand } from './commands/stake';
import { registerConfigCommand } from './commands/config';
import { registerHistoryCommand } from './commands/history';
import { registerInfoCommand } from './commands/info';
import { registerAddressCommand } from './commands/address';
import { registerInitCommand } from './commands/init';
import { registerProgramCommand } from './commands/program';

const VERSION = '1.0.0';

const BANNER = `
  ${chalk.cyan.bold('Prism CLI Wallet')} ${chalk.gray(`v${VERSION}`)}
  ${chalk.gray('A command-line wallet for the Prism blockchain')}
`;

const program = new Command();

program
  .name('prism')
  .version(VERSION)
  .description('Prism CLI Wallet - manage keys, tokens, and transactions')
  .addHelpText('before', BANNER);

// ─── Register All Commands ──────────────────────────────────────────────────

registerKeygenCommand(program);
registerBalanceCommand(program);
registerTransferCommand(program);
registerAirdropCommand(program);
registerTokenCommand(program);
registerStakeCommand(program);
registerConfigCommand(program);
registerHistoryCommand(program);
registerInfoCommand(program);
registerAddressCommand(program);
registerInitCommand(program);
registerProgramCommand(program);

// ─── Global Error Handling ──────────────────────────────────────────────────

program.showHelpAfterError('(add --help for additional information)');

process.on('uncaughtException', (err) => {
  console.error(chalk.red(`\n  Fatal error: ${err.message}`));
  process.exit(1);
});

process.on('unhandledRejection', (reason: any) => {
  console.error(chalk.red(`\n  Unhandled error: ${reason?.message || reason}`));
  process.exit(1);
});

// ─── Parse & Execute ────────────────────────────────────────────────────────

program.parseAsync(process.argv).catch((err) => {
  console.error(chalk.red(`\n  Error: ${err.message}`));
  process.exit(1);
});
