class AppConstants {
  static const String appName = 'SolClone Wallet';
  static const String appVersion = '1.0.0';

  // Default RPC URLs
  static const String mainnetRpcUrl = 'https://api.mainnet-beta.solana.com';
  static const String testnetRpcUrl = 'https://api.testnet.solana.com';
  static const String devnetRpcUrl = 'https://api.devnet.solana.com';
  static const String localRpcUrl = 'http://localhost:8899';

  // Default network
  static const String defaultRpcUrl = localRpcUrl;
  static const String defaultNetwork = 'localnet';

  // Program IDs
  static const String systemProgramId = '11111111111111111111111111111111';
  static const String tokenProgramId = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
  static const String associatedTokenProgramId = 'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL';
  static const String stakeProgramId = 'Stake11111111111111111111111111111111111111';

  // Storage keys
  static const String walletMnemonicKey = 'wallet_mnemonic';
  static const String walletPrivateKeyKey = 'wallet_private_key';
  static const String walletPublicKeyKey = 'wallet_public_key';
  static const String selectedNetworkKey = 'selected_network';
  static const String customRpcUrlKey = 'custom_rpc_url';
  static const String biometricEnabledKey = 'biometric_enabled';
  static const String hasOnboardedKey = 'has_onboarded';

  // SOL decimals
  static const int solDecimals = 9;
  static const double lamportsPerSol = 1000000000;

  // Animation durations
  static const Duration shortAnimation = Duration(milliseconds: 200);
  static const Duration mediumAnimation = Duration(milliseconds: 400);
  static const Duration longAnimation = Duration(milliseconds: 800);
}
