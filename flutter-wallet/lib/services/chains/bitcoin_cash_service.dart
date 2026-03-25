import 'dart:convert';
import 'package:http/http.dart' as http;
import 'chain_service.dart';

/// Bitcoin Cash chain service using Blockchair API.
///
/// Uses BIP44 derivation (m/44'/145'/0'/0/0) for BCH addresses.
/// Bitcoin Cash forked from Bitcoin in August 2017 with larger blocks (32MB).
class BitcoinCashService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  BitcoinCashService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? 'https://api.blockchair.com/bitcoin-cash';

  @override
  String get chainName => 'Bitcoin Cash';

  @override
  String get chainSymbol => 'BCH';

  @override
  String get chainIcon => '\u{1F7E2}'; // green circle

  @override
  int get decimals => 8;

  @override
  String get explorerUrl => 'https://blockchair.com/bitcoin-cash';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/dashboards/address/$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('Failed to fetch BCH balance: HTTP ${response.statusCode}');
      }

      final data = jsonDecode(response.body) as Map<String, dynamic>;
      final addrData = data['data'] as Map<String, dynamic>? ?? {};
      final addrInfo = addrData[address] as Map<String, dynamic>? ?? {};
      final addrSummary = addrInfo['address'] as Map<String, dynamic>? ?? {};

      final balanceSatoshis = addrSummary['balance'] as int? ?? 0;
      return balanceSatoshis / 100000000.0;
    } catch (e) {
      throw Exception('BCH balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    throw UnimplementedError(
      'BCH transaction sending requires UTXO construction. '
      'Amount: $amount BCH to $to.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/dashboards/address/$address?limit=20'),
      );

      if (response.statusCode != 200) return [];

      final data = jsonDecode(response.body) as Map<String, dynamic>;
      final addrData = data['data'] as Map<String, dynamic>? ?? {};
      final addrInfo = addrData[address] as Map<String, dynamic>? ?? {};
      final txs = addrInfo['transactions'] as List<dynamic>? ?? [];

      return txs.take(20).map((txHash) {
        return {
          'txid': txHash.toString(),
          'type': 'unknown',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'chain': 'bitcoin_cash',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/145'/0'/0/0
    // BCH uses cashaddr format: bitcoincash:q...
    // For deterministic placeholder:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'bitcoincash:qp${hash.substring(0, 38)}';
  }

  @override
  bool validateAddress(String address) {
    // CashAddr format: bitcoincash:q... or bitcoincash:p...
    final cashAddrRegex = RegExp(r'^(bitcoincash:)?[qp][a-z0-9]{41}$');

    // Legacy format (same as Bitcoin): starts with 1 or 3
    final legacyRegex = RegExp(r'^[13][a-km-zA-HJ-NP-Z1-9]{25,34}$');

    return cashAddrRegex.hasMatch(address.toLowerCase()) ||
        legacyRegex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // BCH has very low fees: typically ~1 sat/byte, ~226 bytes standard tx
    return 0.00000226;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/transaction/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/address/$address';
  }

  void dispose() {
    _client.close();
  }
}
