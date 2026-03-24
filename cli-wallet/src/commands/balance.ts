import { Command } from 'commander';
import ora from 'ora';
import { createRpcClient } from '../lib/rpc-client';
import { loadDefaultKeypair, getPublicKeyBase58 } from '../lib/keypair';
import {
  printError,
  printKeyValue,
  printHeader,
  formatSol,
  colors,
  println,
} from '../lib/display';

export function registerBalanceCommand(program: Command): void {
  program
    .command('balance [address]')
    .description('Check SOL balance of an account')
    .option('--lamports', 'Display balance in lamports instead of SOL')
    .action(async (address: string | undefined, options) => {
      try {
        let targetAddress = address;

        if (!targetAddress) {
          const keypair = loadDefaultKeypair();
          targetAddress = getPublicKeyBase58(keypair);
        }

        const spinner = ora({
          text: 'Fetching balance...',
          color: 'cyan',
        }).start();

        const rpc = createRpcClient();

        try {
          const result = await rpc.getBalance(targetAddress);
          spinner.stop();

          printHeader('Account Balance');
          printKeyValue('Address', colors.address(targetAddress));

          if (options.lamports) {
            printKeyValue('Balance', `${colors.amount(result.value.toLocaleString())} lamports`);
          } else {
            printKeyValue('Balance', formatSol(result.value));
          }

          printKeyValue('Slot', colors.muted(result.context.slot.toString()));
          println();
        } catch (err: any) {
          spinner.fail('Failed to fetch balance');
          throw err;
        }
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });
}
