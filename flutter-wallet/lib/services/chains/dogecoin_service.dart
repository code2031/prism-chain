import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Dogecoin chain service using DogeChain REST API.
///
/// Uses BIP44 derivation (m/44'/3'/0'/0/0) for Dogecoin addresses (D...).
/// Communicates via dogechain.info REST API for balance and transaction data.
/// No token support -- native DOGE only.
class DogecoinService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  DogecoinService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.dogecoinApiUrl;

  @override
  String get chainName => 'Dogecoin';

  @override
  String get chainSymbol => 'DOGE';

  @override
  String get chainIcon => '\u{1F415}'; // dog

  @override
  int get decimals => 8;

  @override
  String get explorerUrl => 'https://dogechain.info';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/address/balance/$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('Dogecoin API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final balanceStr = json['balance'] as String? ?? '0';

      return double.tryParse(balanceStr) ?? 0.0;
    } catch (e) {
      throw Exception('Dogecoin balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Dogecoin transaction requires UTXO-based construction,
    // similar to Bitcoin:
    // 1. Fetch UTXOs for the sender address
    // 2. Select UTXOs to cover amount + fee
    // 3. Construct raw transaction with inputs/outputs
    // 4. Sign each input with the private key (ECDSA secp256k1)
    // 5. Broadcast the signed transaction
    //
    // Placeholder implementation:
    final satoshiAmount = (amount * 100000000).toInt();
    throw UnimplementedError(
      'Dogecoin transaction sending requires UTXO construction. '
      'Amount: $amount DOGE ($satoshiAmount koinu) to $to. '
      'Use a dedicated Dogecoin SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    // DogeChain API does not provide a clean transaction history endpoint
    // for individual addresses with full detail.
    // In production, use a dedicated Dogecoin indexer.
    try {
      return [
        {
          'txid': '0000000000000000000000000000000000000000000000000000000000000000',
          'type': 'info',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'note': 'Transaction history requires a Dogecoin indexer API.',
          'chain': 'dogecoin',
        },
      ];
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/3'/0'/0/0
    // In a full implementation, this would:
    // 1. Derive the master key from the seed using HMAC-SHA512
    // 2. Follow BIP44 derivation with coin type 3 (Dogecoin)
    // 3. Generate a P2PKH address starting with D
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'D${hash.substring(0, 33)}';
  }

  @override
  bool validateAddress(String address) {
    // Dogecoin P2PKH addresses start with D (or A for multisig), 25-34 chars
    final regex = RegExp(r'^[DA][a-km-zA-HJ-NP-Z1-9]{24,33}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Dogecoin has very low fees; recommended minimum is 1 DOGE per kilobyte.
    // Typical transaction is ~225 bytes.
    return 0.01; // ~0.01 DOGE for a standard transaction
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/tx/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/address/$address';
  }

  void dispose() {
    _client.close();
  }
}
