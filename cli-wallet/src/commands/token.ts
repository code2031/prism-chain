import { Command } from 'commander';
import ora from 'ora';
import bs58 from 'bs58';
import * as nacl from 'tweetnacl';
import { createRpcClient } from '../lib/rpc-client';
import {
  loadDefaultKeypair,
  getPublicKeyBase58,
  base58ToPublicKey,
  generateKeypair,
  publicKeyToBase58,
} from '../lib/keypair';
import {
  buildAndSignTransaction,
  createAccountInstruction,
  createInitializeMintInstruction,
  createMintToInstruction,
  createTokenTransferInstruction,
  createAssociatedTokenAccountInstruction,
  findAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from '../lib/transaction';
import {
  printError,
  printSuccess,
  printKeyValue,
  printHeader,
  printWarning,
  colors,
  println,
  printTable,
  truncate,
} from '../lib/display';

export function registerTokenCommand(program: Command): void {
  const token = program
    .command('token')
    .description('SPL token operations');

  // ─── Create Mint ──────────────────────────────────────────────────────

  token
    .command('create-mint')
    .description('Create a new SPL token mint')
    .option('-d, --decimals <number>', 'Number of decimals', '9')
    .action(async (options) => {
      try {
        const decimals = parseInt(options.decimals, 10);
        if (isNaN(decimals) || decimals < 0 || decimals > 18) {
          printError('Decimals must be between 0 and 18');
          process.exit(1);
          return;
        }

        const keypair = loadDefaultKeypair();
        const authority = getPublicKeyBase58(keypair);

        printHeader('Create Token Mint');
        printKeyValue('Authority', colors.address(authority));
        printKeyValue('Decimals', decimals.toString());
        println();

        const rpc = createRpcClient();

        // Generate a new keypair for the mint account
        const mintKeypair = generateKeypair();
        const mintAddress = getPublicKeyBase58(mintKeypair);

        // Get rent exemption
        const rentSpinner = ora('Calculating rent exemption...').start();
        let rentExemption: number;
        try {
          // SPL Mint account size is 82 bytes
          rentExemption = await rpc.getMinimumBalanceForRentExemption(82);
          rentSpinner.stop();
        } catch (err: any) {
          rentSpinner.fail('Failed to get rent exemption');
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

        // Build transaction with two instructions:
        // 1. CreateAccount for the mint
        // 2. InitializeMint
        const signSpinner = ora('Building transaction...').start();
        let txBase64: string;
        try {
          const createIx = createAccountInstruction(
            keypair.publicKey,
            mintKeypair.publicKey,
            BigInt(rentExemption),
            BigInt(82),
            TOKEN_PROGRAM_ID
          );

          const initMintIx = createInitializeMintInstruction(
            mintKeypair.publicKey,
            decimals,
            keypair.publicKey,
            keypair.publicKey // freeze authority = mint authority
          );

          txBase64 = buildAndSignTransaction(
            {
              feePayer: keypair.publicKey,
              recentBlockhash,
              instructions: [createIx, initMintIx],
            },
            [keypair.secretKey, mintKeypair.secretKey]
          );
          signSpinner.stop();
        } catch (err: any) {
          signSpinner.fail('Failed to build transaction');
          throw err;
        }

        // Send transaction
        const sendSpinner = ora('Creating token mint...').start();
        let signature: string;
        try {
          signature = await rpc.sendTransaction(txBase64);
          sendSpinner.succeed('Token mint created');
        } catch (err: any) {
          sendSpinner.fail('Failed to create token mint');
          throw err;
        }

        // Confirm
        const confirmSpinner = ora('Confirming...').start();
        try {
          await rpc.confirmTransaction(signature);
          confirmSpinner.succeed('Confirmed');
        } catch (err: any) {
          confirmSpinner.warn('Confirmation timed out');
        }

        println();
        printKeyValue('Mint Address', colors.address(mintAddress));
        printKeyValue('Signature', colors.primary(signature));
        printKeyValue('Decimals', decimals.toString());
        printKeyValue('Mint Authority', colors.address(authority));
        printKeyValue('Freeze Authority', colors.address(authority));
        println();
        printSuccess('Token mint created successfully');
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });

  // ─── Mint Tokens ──────────────────────────────────────────────────────

  token
    .command('mint <mint> <amount>')
    .description('Mint tokens to your token account')
    .option('--to <address>', 'Mint to a specific token account')
    .action(async (mintAddress: string, amountStr: string, options) => {
      try {
        const amount = parseFloat(amountStr);
        if (isNaN(amount) || amount <= 0) {
          printError('Amount must be a positive number');
          process.exit(1);
          return;
        }

        const keypair = loadDefaultKeypair();
        const ownerAddress = getPublicKeyBase58(keypair);
        const mintPubkey = base58ToPublicKey(mintAddress);

        printHeader('Mint Tokens');
        printKeyValue('Mint', colors.address(mintAddress));
        printKeyValue('Amount', colors.amount(amountStr));
        println();

        const rpc = createRpcClient();

        // Find or create associated token account
        const ataSpinner = ora('Finding token account...').start();
        const ata = findAssociatedTokenAddress(keypair.publicKey, mintPubkey);
        const ataAddress = publicKeyToBase58(ata);
        ataSpinner.stop();
        printKeyValue('Token Account', colors.address(ataAddress));

        // Get blockhash
        const bhResult = await rpc.getLatestBlockhash();
        const recentBlockhash = bhResult.value.blockhash;

        // Build mint instruction
        // Convert amount to raw (assuming we know decimals, default 9)
        const rawAmount = BigInt(Math.round(amount * 1_000_000_000));

        const mintIx = createMintToInstruction(
          mintPubkey,
          ata,
          keypair.publicKey,
          rawAmount
        );

        // Optionally create the ATA first
        const createAtaIx = createAssociatedTokenAccountInstruction(
          keypair.publicKey,
          ata,
          keypair.publicKey,
          mintPubkey
        );

        const txBase64 = buildAndSignTransaction(
          {
            feePayer: keypair.publicKey,
            recentBlockhash,
            instructions: [createAtaIx, mintIx],
          },
          [keypair.secretKey]
        );

        const sendSpinner = ora('Minting tokens...').start();
        try {
          const signature = await rpc.sendTransaction(txBase64);
          sendSpinner.succeed('Tokens minted');

          const confirmSpinner = ora('Confirming...').start();
          try {
            await rpc.confirmTransaction(signature);
            confirmSpinner.succeed('Confirmed');
          } catch {
            confirmSpinner.warn('Confirmation timed out');
          }

          printKeyValue('Signature', colors.primary(signature));
          println();
          printSuccess(`Minted ${amountStr} tokens`);
        } catch (err: any) {
          sendSpinner.fail('Failed to mint tokens');
          throw err;
        }
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });

  // ─── Transfer Tokens ──────────────────────────────────────────────────

  token
    .command('transfer <mint> <to> <amount>')
    .description('Transfer SPL tokens to another address')
    .action(async (mintAddress: string, to: string, amountStr: string) => {
      try {
        const amount = parseFloat(amountStr);
        if (isNaN(amount) || amount <= 0) {
          printError('Amount must be a positive number');
          process.exit(1);
          return;
        }

        const keypair = loadDefaultKeypair();
        const mintPubkey = base58ToPublicKey(mintAddress);
        const toPubkey = base58ToPublicKey(to);

        printHeader('Transfer Tokens');
        printKeyValue('Mint', colors.address(mintAddress));
        printKeyValue('To', colors.address(to));
        printKeyValue('Amount', colors.amount(amountStr));
        println();

        const rpc = createRpcClient();

        // Source ATA
        const sourceAta = findAssociatedTokenAddress(keypair.publicKey, mintPubkey);
        // Destination ATA
        const destAta = findAssociatedTokenAddress(toPubkey, mintPubkey);

        const bhResult = await rpc.getLatestBlockhash();
        const recentBlockhash = bhResult.value.blockhash;

        const rawAmount = BigInt(Math.round(amount * 1_000_000_000));

        // Create destination ATA if it doesn't exist, then transfer
        const createAtaIx = createAssociatedTokenAccountInstruction(
          keypair.publicKey,
          destAta,
          toPubkey,
          mintPubkey
        );

        const transferIx = createTokenTransferInstruction(
          sourceAta,
          destAta,
          keypair.publicKey,
          rawAmount
        );

        const txBase64 = buildAndSignTransaction(
          {
            feePayer: keypair.publicKey,
            recentBlockhash,
            instructions: [createAtaIx, transferIx],
          },
          [keypair.secretKey]
        );

        const sendSpinner = ora('Transferring tokens...').start();
        try {
          const signature = await rpc.sendTransaction(txBase64);
          sendSpinner.succeed('Tokens transferred');

          const confirmSpinner = ora('Confirming...').start();
          try {
            await rpc.confirmTransaction(signature);
            confirmSpinner.succeed('Confirmed');
          } catch {
            confirmSpinner.warn('Confirmation timed out');
          }

          printKeyValue('Signature', colors.primary(signature));
          println();
          printSuccess(`Transferred ${amountStr} tokens to ${to}`);
        } catch (err: any) {
          sendSpinner.fail('Failed to transfer tokens');
          throw err;
        }
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });

  // ─── Token Balance ────────────────────────────────────────────────────

  token
    .command('balance <mint> [owner]')
    .description('Check token balance for a specific mint')
    .action(async (mintAddress: string, owner: string | undefined) => {
      try {
        let ownerAddress = owner;
        if (!ownerAddress) {
          const keypair = loadDefaultKeypair();
          ownerAddress = getPublicKeyBase58(keypair);
        }

        const mintPubkey = base58ToPublicKey(mintAddress);
        const ownerPubkey = base58ToPublicKey(ownerAddress);

        printHeader('Token Balance');

        const spinner = ora('Fetching token balance...').start();
        const rpc = createRpcClient();

        try {
          const ata = findAssociatedTokenAddress(ownerPubkey, mintPubkey);
          const ataAddress = publicKeyToBase58(ata);

          const result = await rpc.getTokenAccountBalance(ataAddress);
          spinner.stop();

          printKeyValue('Mint', colors.address(mintAddress));
          printKeyValue('Owner', colors.address(ownerAddress));
          printKeyValue('Token Account', colors.address(ataAddress));
          printKeyValue(
            'Balance',
            `${colors.amount(result.value.uiAmountString)} tokens`
          );
          printKeyValue('Raw Amount', result.value.amount);
          printKeyValue('Decimals', result.value.decimals.toString());
          println();
        } catch (err: any) {
          spinner.fail('Failed to fetch token balance');
          if (err.message.includes('could not find')) {
            printWarning('No token account found for this mint/owner combination.');
            printKeyValue('Balance', colors.amount('0') + ' tokens');
          } else {
            throw err;
          }
        }
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });

  // ─── List Token Accounts ──────────────────────────────────────────────

  token
    .command('accounts [owner]')
    .description('List all token accounts for an owner')
    .action(async (owner: string | undefined) => {
      try {
        let ownerAddress = owner;
        if (!ownerAddress) {
          const keypair = loadDefaultKeypair();
          ownerAddress = getPublicKeyBase58(keypair);
        }

        printHeader('Token Accounts');
        printKeyValue('Owner', colors.address(ownerAddress));
        println();

        const spinner = ora('Fetching token accounts...').start();
        const rpc = createRpcClient();

        try {
          const result = await rpc.getTokenAccountsByOwner(ownerAddress);
          spinner.stop();

          if (result.value.length === 0) {
            printWarning('No token accounts found.');
            return;
          }

          const headers = ['Mint', 'Account', 'Balance', 'Decimals'];
          const rows = result.value.map((account) => {
            const info = account.account.data.parsed.info;
            return [
              truncate(info.mint, 16),
              truncate(account.pubkey, 16),
              info.tokenAmount.uiAmountString,
              info.tokenAmount.decimals.toString(),
            ];
          });

          printTable(headers, rows);
          println();
          printSuccess(`Found ${result.value.length} token account(s)`);
        } catch (err: any) {
          spinner.fail('Failed to fetch token accounts');
          throw err;
        }
      } catch (err: any) {
        printError(err.message);
        process.exit(1);
      }
    });
}
