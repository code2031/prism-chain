import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Stellar chain service using Horizon REST API.
///
/// Uses BIP44 derivation (m/44'/148'/0') for Stellar addresses (G...).
/// Communicates via Stellar Horizon API for balance and transaction queries.
/// Stellar is a payments-focused network with fast finality and low fees.
class StellarService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  StellarService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.stellarApiUrl;

  @override
  String get chainName => 'Stellar';

  @override
  String get chainSymbol => 'XLM';

  @override
  String get chainIcon => '\u{2B50}'; // star

  @override
  int get decimals => 7;

  @override
  String get explorerUrl => 'https://stellarchain.io';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/accounts/$address'),
      );

      if (response.statusCode == 404) {
        return 0.0; // Account not funded
      }

      if (response.statusCode != 200) {
        throw Exception('Stellar API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final balances = json['balances'] as List<dynamic>? ?? [];

      for (final balance in balances) {
        final balanceData = balance as Map<String, dynamic>;
        if (balanceData['asset_type'] == 'native') {
          return double.tryParse(balanceData['balance'] as String? ?? '0') ?? 0.0;
        }
      }

      return 0.0;
    } catch (e) {
      throw Exception('Stellar balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Stellar transaction requires:
    // 1. Fetch account sequence number from Horizon
    // 2. Build Transaction with Payment operation
    // 3. Sign with Ed25519 secret key
    // 4. Submit via POST /transactions
    //
    // Placeholder implementation:
    final stroopsAmount = (amount * 10000000).toInt();
    throw UnimplementedError(
      'Stellar transaction sending requires XDR construction and Ed25519 signing. '
      'Amount: $amount XLM ($stroopsAmount stroops) to $to. '
      'Use a dedicated Stellar SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/accounts/$address/payments?limit=20&order=desc'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final embedded = json['_embedded'] as Map<String, dynamic>? ?? {};
      final records = embedded['records'] as List<dynamic>? ?? [];

      return records.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final type = txData['type'] as String? ?? '';
        final from = txData['from'] as String? ?? '';
        final to = txData['to'] as String? ?? '';
        final isReceive = to == address;
        final amountStr = txData['amount'] as String? ?? '0';
        final amount = double.tryParse(amountStr) ?? 0.0;
        final createdAt = txData['created_at'] as String? ?? '';

        return {
          'txid': txData['transaction_hash'] ?? '',
          'type': type == 'create_account'
              ? 'receive'
              : (isReceive ? 'receive' : 'send'),
          'amount': amount,
          'confirmed': true,
          'timestamp': createdAt.isNotEmpty ? createdAt : DateTime.now().toIso8601String(),
          'from': from,
          'to': to,
          'chain': 'stellar',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/148'/0'
    // In a full implementation, this would:
    // 1. Derive the Ed25519 keypair from the seed
    // 2. StrKey encode the public key with version byte 'G' (0x30 << 3)
    // 3. Append CRC16-XModem checksum
    //
    // Deterministic placeholder based on seed bytes:
    // Stellar addresses start with G and are 56 characters
    final hash = seed.take(28).map((b) => (b % 26 + 65)).toList();
    final chars = String.fromCharCodes(hash);
    return 'G${chars.substring(0, 55).padRight(55, 'A')}';
  }

  @override
  bool validateAddress(String address) {
    // Stellar public keys start with G and are 56 characters (StrKey encoded Ed25519)
    final regex = RegExp(r'^G[A-Z2-7]{55}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/fee_stats'),
      );

      if (response.statusCode == 200) {
        final json = jsonDecode(response.body) as Map<String, dynamic>;
        final feeCharged = json['fee_charged'] as Map<String, dynamic>? ?? {};
        final p50 = feeCharged['p50'] as String? ?? '100';
        final stroops = int.tryParse(p50) ?? 100;
        return stroops / 10000000.0;
      }
    } catch (_) {}
    // Base fee is 100 stroops (0.00001 XLM)
    return 0.00001;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/transactions/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/accounts/$address';
  }

  void dispose() {
    _client.close();
  }
}
