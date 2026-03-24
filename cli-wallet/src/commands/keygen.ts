import { Command } from 'commander';
import * as qrcode from 'qrcode-terminal';
import {
  generateKeypair,
  getPublicKeyBase58,
  saveKeypairToFile,
} from '../lib/keypair';
import { getDefaultKeypairPath } from '../lib/config';
import {
  printSuccess,
  printError,
  printKeyValue,
  printHeader,
  printWarning,
  println,
  colors,
  printBox,
} from '../lib/display';
import * as fs from 'fs';

export function registerKeygenCommand(program: Command): void {
  program
    .command('keygen')
    .description('Generate a new Ed25519 keypair')
    .option('-o, --outfile <path>', 'Save keypair to a specific file')
    .option('-f, --force', 'Overwrite existing keypair file', false)
    .option('--no-save', 'Display keypair without saving to disk')
    .option('--no-qr', 'Skip QR code display')
    .action(async (options) => {
      try {
        const keypair = generateKeypair();
        const address = getPublicKeyBase58(keypair);

        printHeader('New Keypair Generated');

        if (options.save === false) {
          // Just display, don't save
          printKeyValue('Public Key', colors.address(address));
          printKeyValue('Secret Key', colors.warning('[hidden - use --outfile to save]'));
          println();

          if (options.qr !== false) {
            console.log(colors.muted('  QR Code (public key):'));
            qrcode.generate(address, { small: true }, (code: string) => {
              const indented = code.split('\n').map((line: string) => '    ' + line).join('\n');
              console.log(indented);
            });
            println();
          }

          printWarning('This keypair was NOT saved. Use --outfile to persist it.');
          return;
        }

        const outfile = options.outfile || getDefaultKeypairPath();

        if (fs.existsSync(outfile) && !options.force) {
          printError(`Keypair file already exists: ${outfile}`);
          console.log(
            `  Use ${colors.primary('--force')} to overwrite, or ${colors.primary(
              '--outfile'
            )} to save elsewhere.`
          );
          process.exit(1);
        }

        saveKeypairToFile(keypair, outfile);

        printKeyValue('Public Key', colors.address(address));
        printKeyValue('Keypair saved to', outfile);
        println();

        if (options.qr !== false) {
          console.log(colors.muted('  QR Code (public key):'));
          qrcode.generate(address, { small: true }, (code: string) => {
            const indented = code.split('\n').map((line: string) => '    ' + line).join('\n');
            console.log(indented);
          });
          println();
        }

        printBox([
          colors.warning('IMPORTANT: Back up your keypair file!'),
          `File: ${outfile}`,
          'Anyone with this file can access your funds.',
          'Store it in a secure location.',
        ]);

        println();
        printSuccess('Keypair generated successfully');
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });
}
