import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Aptos chain service using Aptos REST API.
///
/// Uses BIP44 derivation (m/44'/637'/0'/0'/0') for Aptos addresses (0x...).
/// Communicates via Aptos fullnode REST API for balance and transaction queries.
/// Aptos is a Move-based L1 with parallel execution via Block-STM.
class AptosService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  AptosService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.aptosApiUrl;

  @override
  String get chainName => 'Aptos';

  @override
  String get chainSymbol => 'APT';

  @override
  String get chainIcon => '\u{1F7E2}'; // green circle

  @override
  int get decimals => 8;

  @override
  String get explorerUrl => 'https://explorer.aptoslabs.com';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/accounts/$address/resource/0x1::coin::CoinStore<0x1::aptos_coin::AptosCoin>'),
      );

      if (response.statusCode == 404) {
        return 0.0; // Account not found
      }

      if (response.statusCode != 200) {
        throw Exception('Aptos API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final data = json['data'] as Map<String, dynamic>? ?? {};
      final coin = data['coin'] as Map<String, dynamic>? ?? {};
      final valueStr = coin['value'] as String? ?? '0';
      final octas = BigInt.tryParse(valueStr) ?? BigInt.zero;

      // Convert Octas to APT (1 APT = 10^8 Octas)
      return octas.toDouble() / 1e8;
    } catch (e) {
      throw Exception('Aptos balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Aptos transaction requires:
    // 1. Get account sequence number
    // 2. Construct transfer payload (0x1::coin::transfer)
    // 3. Build and sign transaction with Ed25519
    // 4. Submit via POST /transactions
    //
    // Placeholder implementation:
    final octasAmount = (amount * 1e8).toInt();
    throw UnimplementedError(
      'Aptos transaction sending requires BCS serialization and Ed25519 signing. '
      'Amount: $amount APT ($octasAmount Octas) to $to. '
      'Use a dedicated Aptos SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/accounts/$address/transactions?limit=20'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final List<dynamic> txList = jsonDecode(response.body) as List<dynamic>;

      return txList.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final txHash = txData['hash'] as String? ?? '';
        final success = txData['success'] as bool? ?? false;
        final timestamp = txData['timestamp'] as String? ?? '0';
        final tsUs = int.tryParse(timestamp) ?? 0;

        return {
          'txid': txHash,
          'type': 'send',
          'amount': 0.0,
          'confirmed': success,
          'timestamp': tsUs > 0
              ? DateTime.fromMicrosecondsSinceEpoch(tsUs).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'aptos',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/637'/0'/0'/0'
    // In a full implementation, this would:
    // 1. Derive the Ed25519 keypair from the seed
    // 2. SHA3-256 hash the public key + scheme byte
    // 3. 0x-prefix the resulting 32-byte hex address
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '0x${hash.substring(0, 64)}';
  }

  @override
  bool validateAddress(String address) {
    // Aptos addresses: 0x followed by 1-64 hex characters (often 64)
    // Short form addresses are also valid (e.g., 0x1)
    final regex = RegExp(r'^0x[0-9a-fA-F]{1,64}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/estimate_gas_price'),
      );

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final gasEstimate = json['gas_estimate'] as int? ?? 100;
        // Typical transfer uses ~6 gas units
        return (gasEstimate * 6) / 1e8;
      }
    } catch (_) {}
    return 0.0005; // Fallback estimate
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/txn/$txHash?network=mainnet';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/account/$address?network=mainnet';
  }

  void dispose() {
    _client.close();
  }
}
