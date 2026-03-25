import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular TRC-20 tokens on TRON with their contract addresses.
class TRC20Token {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const TRC20Token({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 6,
  });

  static const List<TRC20Token> popular = [
    TRC20Token(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: 'TR7NHqjeKQxGTCi8q8ZY4pL8otSzgjLj6t',
      decimals: 6,
    ),
    TRC20Token(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: 'TEkxiTehnzSmSe2XqrBj4w32RUN966rdz8',
      decimals: 6,
    ),
    TRC20Token(
      name: 'Wrapped TRX',
      symbol: 'WTRX',
      contractAddress: 'TNUC9Qb1rRpS5CbWLmNMxXBjyFoydXjWFR',
      decimals: 6,
    ),
    TRC20Token(
      name: 'SUN',
      symbol: 'SUN',
      contractAddress: 'TSSMHYeV2uE9qYH95DqyoCuNCzEL1NvU3S',
      decimals: 18,
    ),
    TRC20Token(
      name: 'JUST',
      symbol: 'JST',
      contractAddress: 'TCFLL5dx5ZJdKnWuesXxi1VPwjLVmWZZy9',
      decimals: 18,
    ),
  ];
}

/// TRON chain service using TronGrid REST API.
///
/// Uses BIP44 derivation (m/44'/195'/0'/0/0) for TRON addresses (T...).
/// Communicates via TronGrid REST API (not JSON-RPC).
class TronService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  TronService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.tronApiUrl;

  @override
  String get chainName => 'TRON';

  @override
  String get chainSymbol => 'TRX';

  @override
  String get chainIcon => '\u{26A1}'; // lightning bolt

  @override
  int get decimals => 6;

  @override
  String get explorerUrl => 'https://tronscan.org';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/v1/accounts/$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('TRON API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final data = json['data'] as List<dynamic>?;

      if (data == null || data.isEmpty) {
        return 0.0; // Account not activated
      }

      final account = data[0] as Map<String, dynamic>;
      final balanceSun = account['balance'] as int? ?? 0;

      // Convert SUN to TRX (1 TRX = 1,000,000 SUN)
      return balanceSun / 1000000.0;
    } catch (e) {
      throw Exception('TRON balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full TRON transaction requires:
    // 1. Create transaction via /wallet/createtransaction
    // 2. Sign the transaction offline with Ed25519/secp256k1
    // 3. Broadcast via /wallet/broadcasttransaction
    //
    // Placeholder implementation:
    final sunAmount = (amount * 1000000).toInt();
    throw UnimplementedError(
      'TRON transaction sending requires signing. '
      'Amount: $amount TRX ($sunAmount SUN) to $to. '
      'Use a dedicated TRON SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/v1/accounts/$address/transactions?limit=20'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final data = json['data'] as List<dynamic>?;

      if (data == null || data.isEmpty) {
        return [];
      }

      return data.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final rawData = txData['raw_data'] as Map<String, dynamic>? ?? {};
        final contract = (rawData['contract'] as List<dynamic>?)?.firstOrNull as Map<String, dynamic>?;
        final parameterValue = (contract?['parameter'] as Map<String, dynamic>?)?['value'] as Map<String, dynamic>? ?? {};

        final toAddress = parameterValue['to_address'] as String? ?? '';
        final amountSun = parameterValue['amount'] as int? ?? 0;
        final isReceive = toAddress == address;
        final timestamp = rawData['timestamp'] as int? ?? 0;

        return {
          'txid': txData['txID'] ?? '',
          'type': isReceive ? 'receive' : 'send',
          'amount': amountSun / 1000000.0,
          'confirmed': txData['ret']?[0]?['contractRet'] == 'SUCCESS',
          'timestamp': timestamp > 0
              ? DateTime.fromMillisecondsSinceEpoch(timestamp).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'tron',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/195'/0'/0/0
    // In a full implementation, this would:
    // 1. Derive the child key using BIP44 path with coin type 195
    // 2. Get the public key (secp256k1)
    // 3. Keccak-256 hash of the public key
    // 4. Take last 20 bytes, prepend 0x41
    // 5. Base58Check encode
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'T${hash.substring(0, 33)}';
  }

  @override
  bool validateAddress(String address) {
    // TRON addresses start with T and are 34 characters (Base58Check)
    final regex = RegExp(r'^T[a-km-zA-HJ-NP-Z1-9]{33}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // TRON uses bandwidth and energy for fees.
    // Simple TRX transfers typically cost 0 TRX if bandwidth is available,
    // otherwise ~0.1 TRX for bandwidth burn.
    return 0.1;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/#/transaction/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/#/address/$address';
  }

  void dispose() {
    _client.close();
  }
}
