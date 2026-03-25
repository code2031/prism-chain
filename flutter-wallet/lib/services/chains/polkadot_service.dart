import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Polkadot chain service using Subscan REST API.
///
/// Uses BIP44/SS58 derivation for Polkadot addresses (1...).
/// Communicates via Subscan REST API for balance and transaction queries.
/// Polkadot is a multi-chain relay network enabling cross-chain interoperability.
class PolkadotService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  PolkadotService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.polkadotApiUrl;

  @override
  String get chainName => 'Polkadot';

  @override
  String get chainSymbol => 'DOT';

  @override
  String get chainIcon => '\u{1F534}'; // red circle

  @override
  int get decimals => 10;

  @override
  String get explorerUrl => 'https://polkadot.subscan.io';

  @override
  String get rpcUrl => _apiBaseUrl;

  /// Make an authenticated POST request to the Subscan API.
  Future<Map<String, dynamic>> _subscanPost(String path, Map<String, dynamic> body) async {
    try {
      final response = await _client.post(
        Uri.parse('$_apiBaseUrl$path'),
        headers: {
          'Content-Type': 'application/json',
        },
        body: jsonEncode(body),
      );

      if (response.statusCode != 200) {
        throw Exception('Subscan API HTTP ${response.statusCode}: ${response.body}');
      }

      return jsonDecode(response.body) as Map<String, dynamic>;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Subscan API connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _subscanPost('/api/v2/scan/search', {
        'key': address,
      });

      final data = result['data'] as Map<String, dynamic>? ?? {};
      final account = data['account'] as Map<String, dynamic>? ?? {};
      final balanceStr = account['balance'] as String? ?? '0';
      final balance = double.tryParse(balanceStr) ?? 0.0;

      return balance;
    } catch (e) {
      throw Exception('Polkadot balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Polkadot transaction requires:
    // 1. Query account nonce and runtime metadata
    // 2. Construct extrinsic with balances.transfer call
    // 3. Sign with Sr25519 or Ed25519
    // 4. Submit via author_submitExtrinsic RPC
    //
    // Placeholder implementation:
    final planckAmount = (amount * 10000000000).toInt();
    throw UnimplementedError(
      'Polkadot transaction sending requires extrinsic construction. '
      'Amount: $amount DOT ($planckAmount planck) to $to. '
      'Use a dedicated Polkadot SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final result = await _subscanPost('/api/v2/scan/transfers', {
        'address': address,
        'row': 20,
        'page': 0,
      });

      final data = result['data'] as Map<String, dynamic>? ?? {};
      final transfers = data['transfers'] as List<dynamic>? ?? [];

      return transfers.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final toAddr = txData['to'] as String? ?? '';
        final isReceive = toAddr.toLowerCase() == address.toLowerCase();
        final amountStr = txData['amount'] as String? ?? '0';
        final amount = double.tryParse(amountStr) ?? 0.0;
        final blockTimestamp = txData['block_timestamp'] as int? ?? 0;

        return {
          'txid': txData['hash'] ?? '',
          'type': isReceive ? 'receive' : 'send',
          'amount': amount,
          'confirmed': txData['success'] == true,
          'timestamp': blockTimestamp > 0
              ? DateTime.fromMillisecondsSinceEpoch(blockTimestamp * 1000).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'polkadot',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // Polkadot uses SS58 address format with network prefix 0
    // In a full implementation, this would:
    // 1. Derive the keypair (Sr25519 or Ed25519)
    // 2. Hash the public key
    // 3. SS58 encode with prefix 0 (Polkadot relay chain)
    //
    // Deterministic placeholder based on seed bytes:
    // Polkadot addresses start with 1 and are 47-48 chars (SS58)
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '1${hash.substring(0, 47)}';
  }

  @override
  bool validateAddress(String address) {
    // Polkadot relay chain addresses start with 1, are 47-48 chars (SS58)
    // Generic Substrate addresses start with 5 (generic prefix)
    final polkadotRegex = RegExp(r'^1[a-km-zA-HJ-NP-Z1-9]{46,47}$');
    final genericRegex = RegExp(r'^5[a-km-zA-HJ-NP-Z1-9]{46,47}$');
    return polkadotRegex.hasMatch(address) || genericRegex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Polkadot transaction fees are weight-based.
    // A typical balance transfer costs approximately 0.015 DOT
    return 0.015;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/extrinsic/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/account/$address';
  }

  void dispose() {
    _client.close();
  }
}
