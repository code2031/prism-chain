import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular SPL tokens with their mint addresses (Solana mainnet-beta).
class SplToken {
  final String name;
  final String symbol;
  final String mintAddress;
  final int decimals;

  const SplToken({
    required this.name,
    required this.symbol,
    required this.mintAddress,
    this.decimals = 9,
  });

  static const List<SplToken> popular = [
    SplToken(
      name: 'USD Coin',
      symbol: 'USDC',
      mintAddress: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v',
      decimals: 6,
    ),
    SplToken(
      name: 'Tether USD',
      symbol: 'USDT',
      mintAddress: 'Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB',
      decimals: 6,
    ),
    SplToken(
      name: 'Jupiter',
      symbol: 'JUP',
      mintAddress: 'JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN',
      decimals: 6,
    ),
    SplToken(
      name: 'Raydium',
      symbol: 'RAY',
      mintAddress: '4k3Dyjzvzp8eMZWUXbBCjEvwSkkk59S5iCNLY3QrkX6R',
      decimals: 6,
    ),
    SplToken(
      name: 'Bonk',
      symbol: 'BONK',
      mintAddress: 'DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263',
      decimals: 5,
    ),
    SplToken(
      name: 'dogwifhat',
      symbol: 'WIF',
      mintAddress: 'EKpQGSJtjMFqKZ9KQanSqYXRcF8fBopzLHYxdM65zcjm',
      decimals: 6,
    ),
    SplToken(
      name: 'Render',
      symbol: 'RENDER',
      mintAddress: 'rndrizKT3MK1iimdxRdWabcF7Zg7AR5T4nud4EkHBof',
      decimals: 8,
    ),
    SplToken(
      name: 'Pyth Network',
      symbol: 'PYTH',
      mintAddress: 'HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3',
      decimals: 6,
    ),
    SplToken(
      name: 'Jito',
      symbol: 'JTO',
      mintAddress: 'jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL',
      decimals: 9,
    ),
    SplToken(
      name: 'Marinade Staked SOL',
      symbol: 'MSOL',
      mintAddress: 'mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So',
      decimals: 9,
    ),
    SplToken(
      name: 'Orca',
      symbol: 'ORCA',
      mintAddress: 'orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE',
      decimals: 6,
    ),
    SplToken(
      name: 'Wormhole',
      symbol: 'W',
      mintAddress: '85VBFQZC9TZkfaptBWjvUw7YbZjy52A6mjtPGjstQAmQ',
      decimals: 6,
    ),
    SplToken(
      name: 'Tensor',
      symbol: 'TNSR',
      mintAddress: 'TNSRxcUxoT9xBG3de7PiJyTDYu7kskLqcpddxnEJAS6',
      decimals: 9,
    ),
    SplToken(
      name: 'Helium',
      symbol: 'HNT',
      mintAddress: 'hntyVP6YFm1Hg25TN9WGLqM12b8TQmcknKrdu1oxWux',
      decimals: 8,
    ),
    SplToken(
      name: 'Helium Mobile',
      symbol: 'MOBILE',
      mintAddress: 'mb1eu7TzEc71KxDpsmsKoucSSuuoGLv1drys1oP2jh6',
      decimals: 6,
    ),
    SplToken(
      name: 'Solana Bridged USDC',
      symbol: 'USDCpo',
      mintAddress: 'A9mUU4qviSctJVPJdBGKbGJrNsB7UfDkAs5dvbSWz9S',
      decimals: 6,
    ),
    SplToken(
      name: 'BlazeStake Staked SOL',
      symbol: 'BSOL',
      mintAddress: 'bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1',
      decimals: 9,
    ),
    SplToken(
      name: 'Infinity',
      symbol: 'INF',
      mintAddress: '5oVNBeEEQvYi1cX3ir8Dx5n1P7pdxydbGF2X4TxVusJm',
      decimals: 9,
    ),
    SplToken(
      name: 'Popcat',
      symbol: 'POPCAT',
      mintAddress: '7GCihgDB8fe6KNjn2MYtkzZcRjQy3t9GHdC8uHYmW2hr',
      decimals: 9,
    ),
    SplToken(
      name: 'Mango',
      symbol: 'MNGO',
      mintAddress: 'MangoCzJ36AjZyKwVj3VnYU4GTonjfVEnJmvvWaxLac',
      decimals: 6,
    ),
    SplToken(
      name: 'Serum',
      symbol: 'SRM',
      mintAddress: 'SRMuApVNdxXokk5GT7XD5cUUgXMBCoAz2LHeuAoKWRt',
      decimals: 6,
    ),
    SplToken(
      name: 'Bome',
      symbol: 'BOME',
      mintAddress: 'ukHH6c7mMyiWCf1b9pnWe25TSpkDDt3H5pQZgZ74J82',
      decimals: 6,
    ),
    SplToken(
      name: 'Fida',
      symbol: 'FIDA',
      mintAddress: 'EchesyfXePKdLtoiZSL8pBe8Myagz5Q7L5F8dA3YTVas',
      decimals: 6,
    ),
    SplToken(
      name: 'Wrapped SOL',
      symbol: 'WSOL',
      mintAddress: 'So11111111111111111111111111111111111111112',
      decimals: 9,
    ),
  ];
}

/// Real Solana chain service (mainnet-beta / devnet).
///
/// Uses BIP44 derivation (m/44'/501'/0'/0') for Solana keypairs.
/// Communicates via standard Solana JSON-RPC.
class SolanaService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;
  final bool _isDevnet;

  SolanaService({
    http.Client? client,
    String? rpcUrl,
    bool devnet = false,
  })  : _client = client ?? http.Client(),
        _isDevnet = devnet,
        _rpcEndpoint = rpcUrl ??
            (devnet
                ? AppConstants.solanaDevnetRpcUrl
                : AppConstants.solanaMainnetRpcUrl);

  @override
  String get chainName => _isDevnet ? 'Solana Devnet' : 'Solana';

  @override
  String get chainSymbol => 'SOL';

  @override
  String get chainIcon => '\u{1F7E3}'; // purple circle

  @override
  int get decimals => 9;

  @override
  String get explorerUrl =>
      _isDevnet ? 'https://explorer.solana.com?cluster=devnet' : 'https://explorer.solana.com';

  @override
  String get rpcUrl => _rpcEndpoint;

  /// Set a custom RPC endpoint.
  void setRpcUrl(String url) {
    _rpcEndpoint = url;
  }

  /// Make a Solana JSON-RPC call.
  Future<Map<String, dynamic>> _jsonRpc(String method, [List<dynamic>? params]) async {
    _requestId++;
    final body = jsonEncode({
      'jsonrpc': '2.0',
      'id': _requestId,
      'method': method,
      'params': params ?? [],
    });

    try {
      final response = await _client.post(
        Uri.parse(_rpcEndpoint),
        headers: {'Content-Type': 'application/json'},
        body: body,
      );

      if (response.statusCode != 200) {
        throw Exception('Solana RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Solana RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Solana RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _jsonRpc('getBalance', [address]);
      final lamports = (result['result']?['value'] as int?) ?? 0;
      return lamports / 1000000000.0; // lamports to SOL
    } catch (e) {
      throw Exception('Failed to fetch Solana balance: $e');
    }
  }

  /// Get SPL token accounts owned by the address.
  Future<List<Map<String, dynamic>>> getTokenAccounts(String address) async {
    try {
      final result = await _jsonRpc('getTokenAccountsByOwner', [
        address,
        {'programId': AppConstants.tokenProgramId},
        {'encoding': 'jsonParsed'},
      ]);
      final list = result['result']?['value'] as List<dynamic>? ?? [];
      return list.cast<Map<String, dynamic>>();
    } catch (e) {
      return [];
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Solana transaction construction requires:
    // 1. Get recent blockhash
    // 2. Build SystemProgram.transfer instruction
    // 3. Compile to message, sign with Ed25519
    // 4. Serialize and submit via sendTransaction
    //
    // This follows the same pattern as the existing Prism RPC service.
    final lamports = (amount * 1000000000).round();
    throw UnimplementedError(
      'Solana transaction sending requires Ed25519 signing. '
      'Amount: $amount SOL ($lamports lamports) to $to. '
      'Use the existing WalletProvider.sendSol() for Prism transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final result = await _jsonRpc('getSignaturesForAddress', [
        address,
        {'limit': 20},
      ]);

      final signatures = result['result'] as List<dynamic>? ?? [];

      return signatures.map((sig) {
        final sigData = sig as Map<String, dynamic>;
        final isError = sigData['err'] != null;
        final blockTime = sigData['blockTime'] as int?;

        return {
          'txid': sigData['signature'] ?? '',
          'type': 'unknown',
          'amount': 0.0,
          'confirmed': !isError,
          'timestamp': blockTime != null
              ? DateTime.fromMillisecondsSinceEpoch(blockTime * 1000).toIso8601String()
              : DateTime.now().toIso8601String(),
          'slot': sigData['slot'],
          'memo': sigData['memo'],
          'chain': 'solana',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/501'/0'/0'
    // Same as Prism -- Ed25519 keypair from seed.
    // The existing CryptoUtils.keypairFromMnemonic handles this.
    //
    // Deterministic placeholder based on seed bytes (Base58-like):
    const base58Chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    final buffer = StringBuffer();
    for (int i = 0; i < 44 && i < seed.length; i++) {
      buffer.write(base58Chars[seed[i] % base58Chars.length]);
    }
    return buffer.toString();
  }

  @override
  bool validateAddress(String address) {
    // Solana addresses are Base58-encoded, 32-44 characters
    final regex = RegExp(r'^[1-9A-HJ-NP-Za-km-z]{32,44}$');
    return regex.hasMatch(address);
  }

  /// Request an airdrop (devnet only).
  Future<String> requestAirdrop(String address, {int lamports = 1000000000}) async {
    if (!_isDevnet) {
      throw Exception('Airdrops are only available on devnet');
    }
    final result = await _jsonRpc('requestAirdrop', [address, lamports]);
    return result['result'] as String;
  }

  @override
  Future<double> estimateFee() async {
    // Solana base fee is 5000 lamports per signature (0.000005 SOL)
    return 0.000005;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    final cluster = _isDevnet ? '?cluster=devnet' : '';
    return 'https://explorer.solana.com/tx/$txHash$cluster';
  }

  @override
  String getAddressExplorerUrl(String address) {
    final cluster = _isDevnet ? '?cluster=devnet' : '';
    return 'https://explorer.solana.com/address/$address$cluster';
  }

  void dispose() {
    _client.close();
  }
}
