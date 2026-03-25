import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Hedera chain service using Mirror Node REST API.
///
/// Uses BIP44 derivation (m/44'/3030'/0'/0'/0') for Hedera account IDs (0.0.xxxxx).
/// Communicates via Hedera Mirror Node REST API for balance and transaction queries.
/// Hedera uses hashgraph consensus with council-governed nodes.
class HederaService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  HederaService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.hederaApiUrl;

  @override
  String get chainName => 'Hedera';

  @override
  String get chainSymbol => 'HBAR';

  @override
  String get chainIcon => '\u{2B23}'; // hexagon

  @override
  int get decimals => 8;

  @override
  String get explorerUrl => 'https://hashscan.io/mainnet';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/api/v1/accounts/$address'),
      );

      if (response.statusCode == 404) {
        return 0.0; // Account not found
      }

      if (response.statusCode != 200) {
        throw Exception('Hedera API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final balance = json['balance'] as Map<String, dynamic>? ?? {};
      final tinybars = balance['balance'] as int? ?? 0;

      // Convert tinybars to HBAR (1 HBAR = 100,000,000 tinybars)
      return tinybars / 100000000.0;
    } catch (e) {
      throw Exception('Hedera balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Hedera transaction requires:
    // 1. Create CryptoTransferTransaction
    // 2. Set node account IDs and transaction valid duration
    // 3. Sign with Ed25519 or ECDSA
    // 4. Submit to Hedera network
    //
    // Placeholder implementation:
    final tinybarAmount = (amount * 100000000).toInt();
    throw UnimplementedError(
      'Hedera transaction sending requires Hedera SDK. '
      'Amount: $amount HBAR ($tinybarAmount tinybars) to $to. '
      'Use a dedicated Hedera SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/api/v1/transactions?account.id=$address&limit=20&order=desc'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final transactions = json['transactions'] as List<dynamic>? ?? [];

      return transactions.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final txId = txData['transaction_id'] as String? ?? '';
        final consensusTimestamp = txData['consensus_timestamp'] as String? ?? '';
        final result = txData['result'] as String? ?? '';
        final transfers = txData['transfers'] as List<dynamic>? ?? [];

        // Calculate net transfer for this account
        int netTinybars = 0;
        for (final transfer in transfers) {
          final tData = transfer as Map<String, dynamic>;
          if (tData['account'] == address) {
            netTinybars += (tData['amount'] as int? ?? 0);
          }
        }

        final isReceive = netTinybars > 0;

        return {
          'txid': txId,
          'type': isReceive ? 'receive' : 'send',
          'amount': netTinybars.abs() / 100000000.0,
          'confirmed': result == 'SUCCESS',
          'timestamp': consensusTimestamp.isNotEmpty
              ? DateTime.fromMillisecondsSinceEpoch(
                  (double.tryParse(consensusTimestamp) ?? 0 * 1000).toInt(),
                ).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'hedera',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // Hedera uses account IDs in the format 0.0.xxxxx
    // In a full implementation, this would:
    // 1. Derive Ed25519 or ECDSA keypair from seed
    // 2. Create account on Hedera network (requires existing account to pay)
    // 3. Receive the account ID from the network
    //
    // Deterministic placeholder based on seed bytes:
    final accountNum = seed.take(4).fold<int>(0, (acc, b) => (acc << 8) | b) & 0x7FFFFFFF;
    return '0.0.${accountNum % 10000000}';
  }

  @override
  bool validateAddress(String address) {
    // Hedera account IDs: shard.realm.account (e.g., 0.0.12345)
    // Also supports alias addresses with 0.0.CIQAAAH... format
    final accountIdRegex = RegExp(r'^0\.0\.\d{1,10}$');
    final aliasRegex = RegExp(r'^0\.0\.[a-zA-Z0-9]{20,}$');
    return accountIdRegex.hasMatch(address) || aliasRegex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Hedera CryptoTransfer fee is typically $0.0001 USD
    // At current HBAR prices, approximately 0.001 HBAR
    return 0.001;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/transaction/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/account/$address';
  }

  void dispose() {
    _client.close();
  }
}
