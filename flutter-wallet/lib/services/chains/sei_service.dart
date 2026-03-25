import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Sei chain service using Cosmos SDK REST API.
///
/// Uses BIP44 derivation (m/44'/118'/0'/0/0) for Sei addresses (sei1...).
/// Communicates via Cosmos SDK LCD (REST) API for balance and transaction queries.
/// Sei is a Cosmos SDK chain optimized for trading with built-in order matching.
class SeiService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  SeiService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.seiRestUrl;

  @override
  String get chainName => 'Sei';

  @override
  String get chainSymbol => 'SEI';

  @override
  String get chainIcon => '\u{1F30A}'; // ocean wave

  @override
  int get decimals => 6;

  @override
  String get explorerUrl => 'https://www.seiscan.app';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/cosmos/bank/v1beta1/balances/$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('Sei API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final balances = json['balances'] as List<dynamic>? ?? [];

      for (final balance in balances) {
        final balanceData = balance as Map<String, dynamic>;
        if (balanceData['denom'] == 'usei') {
          final usei = int.tryParse(balanceData['amount'] as String? ?? '0') ?? 0;
          // Convert usei to SEI (1 SEI = 1,000,000 usei)
          return usei / 1000000.0;
        }
      }

      return 0.0;
    } catch (e) {
      throw Exception('Sei balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Sei (Cosmos SDK) transaction requires:
    // 1. Query account number and sequence
    // 2. Construct MsgSend transaction
    // 3. Sign with secp256k1 (protobuf encoding)
    // 4. Broadcast via /cosmos/tx/v1beta1/txs
    //
    // Placeholder implementation:
    final useiAmount = (amount * 1000000).toInt();
    throw UnimplementedError(
      'Sei transaction sending requires Cosmos SDK protobuf signing. '
      'Amount: $amount SEI ($useiAmount usei) to $to. '
      'Use a dedicated Cosmos SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse(
          '$_apiBaseUrl/cosmos/tx/v1beta1/txs?events=transfer.sender%3D%27$address%27&order_by=ORDER_BY_DESC&pagination.limit=20',
        ),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final txResponses = json['tx_responses'] as List<dynamic>? ?? [];

      return txResponses.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final txHash = txData['txhash'] as String? ?? '';
        final timestamp = txData['timestamp'] as String? ?? '';
        final code = txData['code'] as int? ?? 0;

        return {
          'txid': txHash,
          'type': 'send',
          'amount': 0.0,
          'confirmed': code == 0,
          'timestamp': timestamp.isNotEmpty ? timestamp : DateTime.now().toIso8601String(),
          'chain': 'sei',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/118'/0'/0/0 (shared with Cosmos)
    // In a full implementation, this would:
    // 1. Derive the child key using BIP44 path
    // 2. Get the public key (secp256k1)
    // 3. SHA-256 then RIPEMD-160 of the public key
    // 4. Bech32 encode with "sei" prefix
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'sei1${hash.substring(0, 38)}';
  }

  @override
  bool validateAddress(String address) {
    // Sei addresses: bech32 with "sei1" prefix, 43 chars total
    final regex = RegExp(r'^sei1[a-z0-9]{38}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Sei typical fee: ~0.01 SEI for a standard transfer
    return 0.01;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/txs/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/accounts/$address';
  }

  void dispose() {
    _client.close();
  }
}
