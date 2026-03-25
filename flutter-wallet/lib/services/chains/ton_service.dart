import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// TON (The Open Network) chain service using toncenter REST API.
///
/// Uses BIP44 derivation (m/44'/607'/0') for TON addresses (EQ/UQ...).
/// Communicates via toncenter HTTP API for balance and transaction queries.
/// TON is the Telegram-associated blockchain with sharding and instant payments.
class TonService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  TonService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.tonApiUrl;

  @override
  String get chainName => 'TON';

  @override
  String get chainSymbol => 'TON';

  @override
  String get chainIcon => '\u{1F48E}'; // gem

  @override
  int get decimals => 9;

  @override
  String get explorerUrl => 'https://tonscan.org';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/getAddressBalance?address=$address'),
      );

      if (response.statusCode != 200) {
        throw Exception('TON API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json['ok'] != true) {
        throw Exception('TON API error: ${json['error']}');
      }

      final balanceStr = json['result'] as String? ?? '0';
      final nanotons = BigInt.tryParse(balanceStr) ?? BigInt.zero;

      // Convert nanotons to TON (1 TON = 10^9 nanotons)
      return nanotons.toDouble() / 1e9;
    } catch (e) {
      throw Exception('TON balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full TON transaction requires:
    // 1. Create internal message with transfer body
    // 2. Create external message wrapping the internal one
    // 3. Sign with Ed25519
    // 4. Send via sendBoc method
    //
    // Placeholder implementation:
    final nanotonAmount = (amount * 1e9).toInt();
    throw UnimplementedError(
      'TON transaction sending requires BOC construction and Ed25519 signing. '
      'Amount: $amount TON ($nanotonAmount nanotons) to $to. '
      'Use a dedicated TON SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/getTransactions?address=$address&limit=20'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json['ok'] != true) {
        return [];
      }

      final transactions = json['result'] as List<dynamic>? ?? [];

      return transactions.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final inMsg = txData['in_msg'] as Map<String, dynamic>? ?? {};
        final outMsgs = txData['out_msgs'] as List<dynamic>? ?? [];
        final utime = txData['utime'] as int? ?? 0;

        final inValue = int.tryParse(inMsg['value'] as String? ?? '0') ?? 0;
        final isReceive = inValue > 0 && outMsgs.isEmpty;

        double amount = 0.0;
        if (isReceive) {
          amount = inValue / 1e9;
        } else if (outMsgs.isNotEmpty) {
          final outMsg = outMsgs[0] as Map<String, dynamic>;
          amount = (int.tryParse(outMsg['value'] as String? ?? '0') ?? 0) / 1e9;
        }

        return {
          'txid': '${txData['transaction_id']?['hash'] ?? ''}',
          'type': isReceive ? 'receive' : 'send',
          'amount': amount,
          'confirmed': true,
          'timestamp': utime > 0
              ? DateTime.fromMillisecondsSinceEpoch(utime * 1000).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'ton',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/607'/0'
    // In a full implementation, this would:
    // 1. Derive the Ed25519 keypair from the seed
    // 2. Compute the wallet contract state init
    // 3. Hash to get the raw address (workchain:hash)
    // 4. Encode as user-friendly EQ... (bounceable) or UQ... (non-bounceable)
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'EQ${hash.substring(0, 46)}';
  }

  @override
  bool validateAddress(String address) {
    // TON user-friendly addresses:
    // EQ... (bounceable, 48 chars) or UQ... (non-bounceable, 48 chars)
    // Base64url encoded
    final userFriendlyRegex = RegExp(r'^[EU]Q[A-Za-z0-9_\-]{46}$');

    // Raw addresses: workchain:hex_hash
    final rawRegex = RegExp(r'^-?[0-9]:[0-9a-fA-F]{64}$');

    return userFriendlyRegex.hasMatch(address) || rawRegex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // TON transaction fees are typically very low
    // A standard transfer costs approximately 0.005 TON
    return 0.005;
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
