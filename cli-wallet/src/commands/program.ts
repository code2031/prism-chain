import { Command } from 'commander';
import ora from 'ora';
import * as fs from 'fs';
import * as path from 'path';
import { execFileSync } from 'child_process';
import { createRpcClient } from '../lib/rpc-client';
import { loadDefaultKeypair, getPublicKeyBase58 } from '../lib/keypair';
import {
  printError,
  printSuccess,
  printKeyValue,
  printHeader,
  printWarning,
  formatSol,
  colors,
  println,
  lamportsToSol,
} from '../lib/display';

export function registerProgramCommand(program: Command): void {
  const cmd = program
    .command('program')
    .description('Deploy and manage on-chain programs (smart contracts)');

  // ─── Deploy ────────────────────────────────────────────────────────────

  cmd
    .command('deploy <program-path>')
    .description('Deploy a compiled program (.so) to the network')
    .option('--keypair <path>', 'Program keypair for deterministic address')
    .option('--upgrade-authority <address>', 'Set upgrade authority (default: deployer)')
    .action(async (programPath: string, options) => {
      try {
        printHeader('Deploy Program');

        // Validate file exists
        const resolvedPath = path.resolve(programPath);
        if (!fs.existsSync(resolvedPath)) {
          printError(`File not found: ${resolvedPath}`);
          process.exit(1);
        }

        if (!resolvedPath.endsWith('.so')) {
          printWarning('File does not have .so extension — are you sure this is a compiled Solana program?');
        }

        // Read program binary
        const programData = fs.readFileSync(resolvedPath);
        const programSize = programData.length;

        println(`  Program binary: ${resolvedPath}`);
        println(`  Size: ${(programSize / 1024).toFixed(1)} KB`);
        println('');

        // Estimate deployment cost
        const estimatedRentLamports = Math.ceil(programSize * 6.96) + 1_000_000;
        const estimatedCost = lamportsToSol(estimatedRentLamports + 5_000_000);
        printKeyValue('Estimated cost', `~${formatSol(estimatedCost)}`);

        // Load deployer keypair
        const keypair = loadDefaultKeypair();
        const deployerAddress = getPublicKeyBase58(keypair);
        printKeyValue('Deployer', deployerAddress);

        // Check balance
        const rpc = createRpcClient();
        const balance = await rpc.getBalance(deployerAddress);
        const balanceSol = lamportsToSol(balance);
        printKeyValue('Balance', formatSol(balanceSol));

        if (balance < estimatedRentLamports + 5_000_000) {
          printError(
            `Insufficient balance. Need ~${formatSol(estimatedCost)} SOL but only have ${formatSol(balanceSol)} SOL.\n` +
            `  Request an airdrop: npx prism airdrop 100`
          );
          process.exit(1);
        }

        println('');
        const spinner = ora('Deploying program to the network...').start();

        try {
          // Find the Solana/Prism CLI binary
          const validatorCli = path.resolve(__dirname, '../../../validator/target/release/solana');
          const altCli = path.resolve(__dirname, '../../../validator/target/release/prism');

          let cli: string;
          if (fs.existsSync(validatorCli)) {
            cli = validatorCli;
          } else if (fs.existsSync(altCli)) {
            cli = altCli;
          } else {
            cli = 'solana';
          }

          const rpcUrl = rpc.getUrl();

          // Use execFileSync to avoid shell injection
          const result = execFileSync(cli, [
            'program', 'deploy', resolvedPath,
            '--url', rpcUrl,
          ], { encoding: 'utf-8', timeout: 120_000 });

          spinner.succeed('Program deployed successfully!');
          println('');

          // Parse program ID from output
          const programIdMatch = result.match(/Program Id: ([1-9A-HJ-NP-Za-km-z]{32,44})/);
          if (programIdMatch) {
            printKeyValue('Program ID', programIdMatch[1]);
          }

          println('');
          println(result.trim());
        } catch (cliError: any) {
          spinner.fail('Deployment failed');
          printError(cliError.message || 'Failed to deploy program');
          println('');
          println('Make sure:');
          println('  1. The validator is running (make testnet-bg)');
          println('  2. The program binary is valid (cargo build-sbf)');
          println('  3. You have enough SOL for deployment');
          process.exit(1);
        }

        println('');
        printSuccess('Your smart contract is now live on Prism.');
        println('  Interact with it using the web3.js SDK or any Solana-compatible client.');
      } catch (error: any) {
        printError(error.message || 'Deployment failed');
        process.exit(1);
      }
    });

  // ─── Show ──────────────────────────────────────────────────────────────

  cmd
    .command('show <program-id>')
    .description('Show details of a deployed program')
    .action(async (programId: string) => {
      try {
        printHeader('Program Info');

        const rpc = createRpcClient();
        const spinner = ora('Fetching program info...').start();

        const accountInfo = await rpc.getAccountInfo(programId);

        if (!accountInfo) {
          spinner.fail('Program not found');
          printError(`No account found at address: ${programId}`);
          process.exit(1);
        }

        spinner.succeed('Program found');
        println('');
        printKeyValue('Address', programId);
        printKeyValue('Executable', accountInfo.executable ? 'Yes' : 'No');
        printKeyValue('Owner', accountInfo.owner);
        printKeyValue('Lamports', accountInfo.lamports.toLocaleString());
        printKeyValue('Balance', formatSol(lamportsToSol(accountInfo.lamports)));
        printKeyValue('Data size', `${(accountInfo.data?.length || 0)} bytes`);

        if (!accountInfo.executable) {
          println('');
          printWarning('This account is not an executable program.');
        }
      } catch (error: any) {
        printError(error.message || 'Failed to fetch program info');
        process.exit(1);
      }
    });

  // ─── List Built-in Programs ────────────────────────────────────────────

  cmd
    .command('list')
    .description('List Prism built-in programs')
    .action(async () => {
      printHeader('Built-in Prism Programs');
      println('');

      const programs = [
        { name: 'Staking',           desc: 'Locked tiers (1x-5x), liquid staking (stPRISM), auto-compound' },
        { name: 'Fee Burn',          desc: '50% fee burn, 30% validator, 10% treasury, 10% stakers' },
        { name: 'Vesting',           desc: 'Cliff periods, linear/quarterly schedules, revocable' },
        { name: 'Governance',        desc: 'On-chain voting, delegation, emergency fast-track, veto' },
        { name: 'Privacy',           desc: 'Confidential transfers, shielded pool, stealth addresses' },
        { name: 'Multisig',          desc: 'M-of-N multi-signature accounts (2-of-2 to 11-of-11)' },
        { name: 'Name Service',      desc: '.prism domain names, subdomains, reverse lookup' },
        { name: 'Social Recovery',   desc: 'Guardian-based account recovery (3-5 guardians)' },
        { name: 'Atomic Swap',       desc: 'Hash time-locked contracts for cross-chain swaps' },
        { name: 'Flash Loan',        desc: 'Borrow/repay in one transaction (0.09% fee)' },
        { name: 'Oracle',            desc: 'Multi-source price feeds with median aggregation' },
        { name: 'Batch Tx',          desc: 'Batch, scheduled, conditional, recurring transfers' },
        { name: 'PUSD Stablecoin',   desc: 'Mint PUSD with 150% PRISM collateral, 120% liquidation' },
      ];

      const maxName = Math.max(...programs.map(p => p.name.length));
      for (const p of programs) {
        println(`  ${colors.cyan(p.name.padEnd(maxName + 2))} ${p.desc}`);
      }

      println('');
      println(`  Plus the full SPL program library: Token, Token-2022, Governance, Stake Pool, Memo`);
      println('');
      println('  Build your own: npx prism init my-project --template <token|nft-collection|escrow|voting|staking-pool>');
    });
}
