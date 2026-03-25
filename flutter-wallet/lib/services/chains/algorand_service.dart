import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Algorand chain service using Algod REST API.
///
/// Uses BIP44 derivation (m/44'/283'/0'/0'/0') for Algorand addresses.
/// Communicates via Algod/Algonode REST API for balance and transaction queries.
/// Algorand uses Pure Proof-of-Stake with instant finality.
class AlgorandService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  AlgorandService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.algorandApiUrl;

  @override
  String get chainName => 'Algorand';

  @override
  String get chainSymbol => 'ALGO';

  @override
  String get chainIcon => '\u{25B3}'; // triangle

  @override
  int get decimals => 6;

  @override
  String get explorerUrl => 'https://allo.info';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/v2/accounts/$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('Algorand API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final amountMicroAlgos = json['amount'] as int? ?? 0;

      // Convert microAlgos to ALGO (1 ALGO = 1,000,000 microAlgos)
      return amountMicroAlgos / 1000000.0;
    } catch (e) {
      throw Exception('Algorand balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Algorand transaction requires:
    // 1. Get suggested params (fee, first/last round, genesis info)
    // 2. Construct PaymentTxn
    // 3. Sign with Ed25519
    // 4. Submit via POST /v2/transactions
    //
    // Placeholder implementation:
    final microAlgoAmount = (amount * 1000000).toInt();
    throw UnimplementedError(
      'Algorand transaction sending requires msgpack encoding and Ed25519 signing. '
      'Amount: $amount ALGO ($microAlgoAmount microAlgos) to $to. '
      'Use a dedicated Algorand SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      // Algorand Indexer API for transaction history
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/v2/accounts/$address/transactions?limit=20'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final transactions = json['transactions'] as List<dynamic>? ?? [];

      return transactions.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final txId = txData['id'] as String? ?? '';
        final roundTime = txData['round-time'] as int? ?? 0;
        final paymentTxn = txData['payment-transaction'] as Map<String, dynamic>? ?? {};
        final receiver = paymentTxn['receiver'] as String? ?? '';
        final isReceive = receiver == address;
        final amountMicro = paymentTxn['amount'] as int? ?? 0;

        return {
          'txid': txId,
          'type': isReceive ? 'receive' : 'send',
          'amount': amountMicro / 1000000.0,
          'confirmed': txData['confirmed-round'] != null,
          'timestamp': roundTime > 0
              ? DateTime.fromMillisecondsSinceEpoch(roundTime * 1000).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'algorand',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/283'/0'/0'/0'
    // In a full implementation, this would:
    // 1. Derive the Ed25519 keypair from the seed
    // 2. SHA-512/256 hash of the public key
    // 3. Append last 4 bytes as checksum
    // 4. Base32 encode (no padding) — 58 characters
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(32).map((b) => (b % 26 + 65)).toList();
    final chars = String.fromCharCodes(hash);
    return chars.substring(0, 58).padRight(58, 'A').toUpperCase();
  }

  @override
  bool validateAddress(String address) {
    // Algorand addresses: 58 uppercase base32 characters (A-Z, 2-7)
    final regex = RegExp(r'^[A-Z2-7]{58}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/v2/transactions/params'),
      );

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final minFee = json['min-fee'] as int? ?? 1000;
        return minFee / 1000000.0;
      }
    } catch (_) {}
    // Minimum fee is 1000 microAlgos (0.001 ALGO)
    return 0.001;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/tx/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/account/$address';
  }

  void dispose() {
    _client.close();
  }
}
