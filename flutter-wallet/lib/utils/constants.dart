class AppConstants {
  static const String appName = 'Prism Wallet';
  static const String appVersion = '1.0.0';

  // ── Prism / Solana RPC URLs ──
  static const String mainnetRpcUrl = 'https://api.mainnet-beta.solana.com';
  static const String testnetRpcUrl = 'https://api.testnet.solana.com';
  static const String devnetRpcUrl = 'https://api.devnet.solana.com';
  static const String localRpcUrl = 'http://localhost:8899';

  // Default network
  static const String defaultRpcUrl = localRpcUrl;
  static const String defaultNetwork = 'localnet';

  // ── Multi-chain RPC / API URLs ──

  // Bitcoin - Blockstream.info REST API
  static const String bitcoinApiUrl = 'https://blockstream.info';
  static const String bitcoinTestnetApiUrl = 'https://blockstream.info/testnet';

  // Ethereum - public JSON-RPC endpoints
  static const String ethereumRpcUrl = 'https://eth.llamarpc.com';
  static const String ethereumGoerliRpcUrl = 'https://rpc.ankr.com/eth_goerli';

  // Solana (real mainnet / devnet, distinct from Prism local)
  static const String solanaMainnetRpcUrl = 'https://api.mainnet-beta.solana.com';
  static const String solanaDevnetRpcUrl = 'https://api.devnet.solana.com';

  // Polygon (PoS) - public JSON-RPC
  static const String polygonRpcUrl = 'https://polygon-rpc.com';
  static const String polygonMumbaiRpcUrl = 'https://rpc.ankr.com/polygon_mumbai';

  // BNB Smart Chain - public JSON-RPC
  static const String bnbRpcUrl = 'https://bsc-dataseed.binance.org';
  static const String bnbTestnetRpcUrl = 'https://data-seed-prebsc-1-s1.binance.org:8545';

  // Avalanche C-Chain - public JSON-RPC
  static const String avalancheRpcUrl = 'https://api.avax.network/ext/bc/C/rpc';
  static const String avalancheTestnetRpcUrl = 'https://api.avax-test.network/ext/bc/C/rpc';

  // Arbitrum One - public JSON-RPC
  static const String arbitrumRpcUrl = 'https://arb1.arbitrum.io/rpc';
  static const String arbitrumTestnetRpcUrl = 'https://goerli-rollup.arbitrum.io/rpc';

  // Optimism - public JSON-RPC
  static const String optimismRpcUrl = 'https://mainnet.optimism.io';
  static const String optimismTestnetRpcUrl = 'https://goerli.optimism.io';

  // Base (Coinbase L2) - public JSON-RPC
  static const String baseRpcUrl = 'https://mainnet.base.org';
  static const String baseTestnetRpcUrl = 'https://goerli.base.org';

  // Fantom Opera - public JSON-RPC
  static const String fantomRpcUrl = 'https://rpc.ftm.tools';
  static const String fantomTestnetRpcUrl = 'https://rpc.testnet.fantom.network';

  // Cronos (Crypto.com) - public JSON-RPC
  static const String cronosRpcUrl = 'https://evm.cronos.org';
  static const String cronosTestnetRpcUrl = 'https://evm-t3.cronos.org';

  // TRON - TronGrid REST API
  static const String tronApiUrl = 'https://api.trongrid.io';
  static const String tronTestnetApiUrl = 'https://api.shasta.trongrid.io';

  // Dogecoin - DogeChain REST API
  static const String dogecoinApiUrl = 'https://dogechain.info/api/v1';

  // Litecoin - litecoinspace.org REST API
  static const String litecoinApiUrl = 'https://litecoinspace.org';

  // Cardano - Blockfrost REST API
  static const String cardanoApiUrl = 'https://cardano-mainnet.blockfrost.io/api/v0';
  static const String cardanoTestnetApiUrl = 'https://cardano-preprod.blockfrost.io/api/v0';
  static const String cardanoApiKey = 'YOUR_BLOCKFROST_API_KEY'; // Replace with actual key

  // XRP Ledger - Ripple JSON-RPC
  static const String xrpRpcUrl = 'https://s1.ripple.com:51234';
  static const String xrpTestnetRpcUrl = 'https://s.altnet.rippletest.net:51234';

  // Cosmos Hub - Cosmos REST / Tendermint RPC
  static const String cosmosRestUrl = 'https://rest.cosmos.directory/cosmoshub';
  static const String cosmosRpcUrl = 'https://rpc.cosmos.directory/cosmoshub';

  // Polkadot - Subscan REST API
  static const String polkadotApiUrl = 'https://polkadot.api.subscan.io';
  static const String polkadotRpcUrl = 'wss://rpc.polkadot.io';

  // NEAR Protocol - NEAR JSON-RPC
  static const String nearRpcUrl = 'https://rpc.mainnet.near.org';
  static const String nearTestnetRpcUrl = 'https://rpc.testnet.near.org';

  // Sui - Sui JSON-RPC
  static const String suiRpcUrl = 'https://fullnode.mainnet.sui.io';
  static const String suiTestnetRpcUrl = 'https://fullnode.testnet.sui.io';

  // Aptos - Aptos REST API
  static const String aptosApiUrl = 'https://fullnode.mainnet.aptoslabs.com/v1';
  static const String aptosTestnetApiUrl = 'https://fullnode.testnet.aptoslabs.com/v1';

  // Stellar - Horizon REST API
  static const String stellarApiUrl = 'https://horizon.stellar.org';
  static const String stellarTestnetApiUrl = 'https://horizon-testnet.stellar.org';

  // Algorand - Algod REST API
  static const String algorandApiUrl = 'https://mainnet-api.algonode.cloud';
  static const String algorandTestnetApiUrl = 'https://testnet-api.algonode.cloud';

  // Hedera - Mirror Node REST API
  static const String hederaApiUrl = 'https://mainnet-public.mirrornode.hedera.com';
  static const String hederaTestnetApiUrl = 'https://testnet.mirrornode.hedera.com';

  // TON - toncenter REST API
  static const String tonApiUrl = 'https://toncenter.com/api/v2';
  static const String tonTestnetApiUrl = 'https://testnet.toncenter.com/api/v2';

  // Kaspa - Kaspa REST API
  static const String kaspaApiUrl = 'https://api.kaspa.org';

  // Filecoin - Filecoin JSON-RPC (Glif)
  static const String filecoinRpcUrl = 'https://api.node.glif.io/rpc/v1';
  static const String filecoinTestnetRpcUrl = 'https://api.calibration.node.glif.io/rpc/v1';

  // Celestia - Celestia REST API (Cosmos SDK)
  static const String celestiaRestUrl = 'https://rest.cosmos.directory/celestia';
  static const String celestiaRpcUrl = 'https://rpc.cosmos.directory/celestia';

  // Sei - Sei REST API (Cosmos SDK)
  static const String seiRestUrl = 'https://rest.sei-apis.com';
  static const String seiRpcUrl = 'https://rpc.sei-apis.com';

  // ── Block Explorers ──
  static const String bitcoinExplorer = 'https://blockstream.info';
  static const String ethereumExplorer = 'https://etherscan.io';
  static const String solanaExplorer = 'https://explorer.solana.com';
  static const String polygonExplorer = 'https://polygonscan.com';
  static const String bnbExplorer = 'https://bscscan.com';
  static const String avalancheExplorer = 'https://snowtrace.io';
  static const String arbitrumExplorer = 'https://arbiscan.io';
  static const String optimismExplorer = 'https://optimistic.etherscan.io';
  static const String baseExplorer = 'https://basescan.org';
  static const String fantomExplorer = 'https://ftmscan.com';
  static const String cronosExplorer = 'https://cronoscan.com';
  static const String tronExplorer = 'https://tronscan.org';
  static const String dogecoinExplorer = 'https://dogechain.info';
  static const String litecoinExplorer = 'https://litecoinspace.org';
  static const String cardanoExplorer = 'https://cardanoscan.io';
  static const String xrpExplorer = 'https://xrpscan.com';
  static const String cosmosExplorer = 'https://www.mintscan.io/cosmos';
  static const String polkadotExplorer = 'https://polkadot.subscan.io';
  static const String nearExplorer = 'https://nearblocks.io';
  static const String suiExplorer = 'https://suiscan.xyz';
  static const String aptosExplorer = 'https://explorer.aptoslabs.com';
  static const String stellarExplorer = 'https://stellarchain.io';
  static const String algorandExplorer = 'https://allo.info';
  static const String hederaExplorer = 'https://hashscan.io/mainnet';
  static const String tonExplorer = 'https://tonscan.org';
  static const String kaspaExplorer = 'https://explorer.kaspa.org';
  static const String filecoinExplorer = 'https://filfox.info/en';
  static const String celestiaExplorer = 'https://www.mintscan.io/celestia';
  static const String seiExplorer = 'https://www.seiscan.app';

  // ── Program IDs (Prism / Solana) ──
  static const String systemProgramId = '11111111111111111111111111111111';
  static const String tokenProgramId = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
  static const String associatedTokenProgramId = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
  static const String stakeProgramId = 'Stake11111111111111111111111111111111111111';

  // ── Storage Keys ──
  static const String walletMnemonicKey = 'wallet_mnemonic';
  static const String walletPrivateKeyKey = 'wallet_private_key';
  static const String walletPublicKeyKey = 'wallet_public_key';
  static const String selectedNetworkKey = 'selected_network';
  static const String customRpcUrlKey = 'custom_rpc_url';
  static const String biometricEnabledKey = 'biometric_enabled';
  static const String hasOnboardedKey = 'has_onboarded';
  static const String multiChainViewKey = 'multi_chain_view_enabled';

  // ── SOL Decimals ──
  static const int solDecimals = 9;
  static const double lamportsPerSol = 1000000000;

  // ── Chain-specific Decimals ──
  static const int btcDecimals = 8;
  static const int ethDecimals = 18;
  static const double satoshisPerBtc = 100000000;
  static const double weiPerEth = 1e18;

  // ── Animation Durations ──
  static const Duration shortAnimation = Duration(milliseconds: 200);
  static const Duration mediumAnimation = Duration(milliseconds: 400);
  static const Duration longAnimation = Duration(milliseconds: 800);
}
