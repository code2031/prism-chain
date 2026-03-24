import { Command } from 'commander';
import ora from 'ora';
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

export function registerAirdropCommand(program: Command): void {
  program
    .command('airdrop <amount> [address]')
    .description('Request an airdrop of SOL (testnet/devnet only)')
    .action(async (amountStr: string, address: string | undefined) => {
      try {
        // Parse amount
        const amount = parseFloat(amountStr);
        if (isNaN(amount) || amount <= 0) {
          printError('Amount must be a positive number (in SOL)');
          process.exit(1);
          return;
        }

        const lamports = Math.round(amount * 1_000_000_000);

        // Determine target address
        let targetAddress = address;
        if (!targetAddress) {
          const keypair = loadDefaultKeypair();
          targetAddress = getPublicKeyBase58(keypair);
        }

        printHeader('Airdrop Request');
        printKeyValue('Address', colors.address(targetAddress));
        printKeyValue('Amount', formatSol(lamports));
        println();

        const rpc = createRpcClient();

        // Request airdrop
        const airdropSpinner = ora('Requesting airdrop...').start();
        let signature: string;
        try {
          signature = await rpc.requestAirdrop(targetAddress, lamports);
          airdropSpinner.succeed('Airdrop requested');
        } catch (err: any) {
          airdropSpinner.fail('Airdrop request failed');

          if (err.message.includes('429') || err.message.includes('rate')) {
            printWarning('Rate limit reached. Please try again in a few seconds.');
          } else if (err.message.includes('airdrop')) {
            printWarning(
              'Airdrop may not be available on this network. ' +
                'Try using devnet or testnet.'
            );
          }
          throw err;
        }

        printKeyValue('Signature', colors.primary(signature));

        // Wait for confirmation
        const confirmSpinner = ora('Confirming transaction...').start();
        try {
          await rpc.confirmTransaction(signature);
          confirmSpinner.succeed('Transaction confirmed');
        } catch (err: any) {
          confirmSpinner.fail('Confirmation timed out');
          printWarning(
            'The airdrop may still succeed. Check your balance with: solclone balance'
          );
        }

        // Show new balance
        const balanceSpinner = ora('Fetching updated balance...').start();
        try {
          const balResult = await rpc.getBalance(targetAddress);
          balanceSpinner.stop();
          printKeyValue('New Balance', formatSol(balResult.value));
        } catch {
          balanceSpinner.stop();
        }

        println();
        printSuccess(`Airdropped ${lamportsToSol(lamports)} SOL to ${targetAddress}`);
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });
}
