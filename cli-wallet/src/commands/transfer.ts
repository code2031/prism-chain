import { Command } from 'commander';
import ora from 'ora';
import bs58 from 'bs58';
import { createRpcClient } from '../lib/rpc-client';
import { loadDefaultKeypair, getPublicKeyBase58, base58ToPublicKey } from '../lib/keypair';
import { buildAndSignTransaction, createTransferInstruction } from '../lib/transaction';
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

export function registerTransferCommand(program: Command): void {
  program
    .command('transfer <to> <amount>')
    .description('Send SOL to another address')
    .option('--allow-unfunded', 'Allow transfers to unfunded accounts')
    .option('--no-confirm', 'Do not wait for confirmation')
    .action(async (to: string, amountStr: string, options) => {
      try {
        // Validate recipient address
        let toPublicKey: Uint8Array;
        try {
          toPublicKey = base58ToPublicKey(to);
          if (toPublicKey.length !== 32) {
            throw new Error('Invalid address length');
          }
        } catch {
          printError(`Invalid recipient address: ${to}`);
          process.exit(1);
          return;
        }

        // Parse amount
        const amount = parseFloat(amountStr);
        if (isNaN(amount) || amount <= 0) {
          printError('Amount must be a positive number');
          process.exit(1);
          return;
        }

        const lamports = BigInt(Math.round(amount * 1_000_000_000));

        // Load keypair
        const keypair = loadDefaultKeypair();
        const fromAddress = getPublicKeyBase58(keypair);

        printHeader('Transfer SOL');
        printKeyValue('From', colors.address(fromAddress));
        printKeyValue('To', colors.address(to));
        printKeyValue('Amount', formatSol(Number(lamports)));
        println();

        const rpc = createRpcClient();

        // Check sender balance
        const balanceSpinner = ora('Checking balance...').start();
        try {
          const balanceResult = await rpc.getBalance(fromAddress);
          const balance = balanceResult.value;
          balanceSpinner.stop();

          if (BigInt(balance) < lamports) {
            printError(
              `Insufficient balance. Have ${lamportsToSol(balance)} SOL, ` +
                `need ${lamportsToSol(Number(lamports))} SOL`
            );
            process.exit(1);
            return;
          }
        } catch (err: any) {
          balanceSpinner.fail('Failed to check balance');
          throw err;
        }

        // Get recent blockhash
        const bhSpinner = ora('Getting recent blockhash...').start();
        let recentBlockhash: string;
        try {
          const bhResult = await rpc.getLatestBlockhash();
          recentBlockhash = bhResult.value.blockhash;
          bhSpinner.stop();
        } catch (err: any) {
          bhSpinner.fail('Failed to get blockhash');
          throw err;
        }

        // Build and sign transaction
        const signSpinner = ora('Signing transaction...').start();
        let txBase64: string;
        try {
          const transferIx = createTransferInstruction(
            keypair.publicKey,
            toPublicKey,
            lamports
          );

          txBase64 = buildAndSignTransaction(
            {
              feePayer: keypair.publicKey,
              recentBlockhash,
              instructions: [transferIx],
            },
            [keypair.secretKey]
          );
          signSpinner.stop();
        } catch (err: any) {
          signSpinner.fail('Failed to sign transaction');
          throw err;
        }

        // Send transaction
        const sendSpinner = ora('Sending transaction...').start();
        let signature: string;
        try {
          signature = await rpc.sendTransaction(txBase64);
          sendSpinner.succeed('Transaction sent');
        } catch (err: any) {
          sendSpinner.fail('Failed to send transaction');
          throw err;
        }

        printKeyValue('Signature', colors.primary(signature));

        // Confirm transaction
        if (options.confirm !== false) {
          const confirmSpinner = ora('Confirming transaction...').start();
          try {
            await rpc.confirmTransaction(signature);
            confirmSpinner.succeed('Transaction confirmed');
          } catch (err: any) {
            confirmSpinner.fail('Confirmation failed');
            printWarning(err.message);
          }
        }

        println();
        printSuccess(`Sent ${lamportsToSol(Number(lamports))} SOL to ${to}`);
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });
}
