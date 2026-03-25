import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// NEAR Protocol chain service using NEAR JSON-RPC.
///
/// Uses BIP44 derivation (m/44'/397'/0') for NEAR accounts.
/// NEAR supports human-readable account IDs (e.g., "alice.near") and
/// implicit accounts (64-char hex from Ed25519 public key).
class NearService extends ChainService {
  final http.Client _client;
  final String _rpcEndpoint;

  NearService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.nearRpcUrl;

  int _requestId = 0;

  @override
  String get chainName => 'NEAR';

  @override
  String get chainSymbol => 'NEAR';

  @override
  String get chainIcon => '\u{1F30D}'; // globe

  @override
  int get decimals => 24;

  @override
  String get explorerUrl => 'https://nearblocks.io';

  @override
  String get rpcUrl => _rpcEndpoint;

  /// Make a NEAR JSON-RPC call.
  Future<Map<String, dynamic>> _nearRpc(String method, Map<String, dynamic> params) async {
    _requestId++;
    final body = jsonEncode({
      'jsonrpc': '2.0',
      'id': _requestId,
      'method': method,
      'params': params,
    });

    try {
      final response = await _client.post(
        Uri.parse(_rpcEndpoint),
        headers: {'Content-Type': 'application/json'},
        body: body,
      );

      if (response.statusCode != 200) {
        throw Exception('NEAR RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('NEAR RPC Error: ${error['data'] ?? error['message']}');
      }

      return json['result'] as Map<String, dynamic>? ?? {};
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('NEAR RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _nearRpc('query', {
        'request_type': 'view_account',
        'finality': 'final',
        'account_id': address,
      });

      final amountStr = result['amount'] as String? ?? '0';
      final amount = BigInt.tryParse(amountStr) ?? BigInt.zero;
      // Convert yoctoNEAR to NEAR (1 NEAR = 10^24 yoctoNEAR)
      final nearBalance = amount / BigInt.from(10).pow(24);
      final remainder = amount % BigInt.from(10).pow(24);
      return nearBalance.toDouble() + (remainder.toDouble() / 1e24);
    } catch (e) {
      // Account not found means 0 balance
      if (e.toString().contains('does not exist') ||
          e.toString().contains('UNKNOWN_ACCOUNT')) {
        return 0.0;
      }
      throw Exception('NEAR balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full NEAR transaction requires:
    // 1. Get access key nonce and recent block hash
    // 2. Construct Transaction with Transfer action
    // 3. Sign with Ed25519
    // 4. Broadcast via broadcast_tx_commit
    //
    // Placeholder implementation:
    final yoctoAmount = BigInt.from(amount * 1e6) * BigInt.from(10).pow(18);
    throw UnimplementedError(
      'NEAR transaction sending requires Ed25519 signing. '
      'Amount: $amount NEAR ($yoctoAmount yoctoNEAR) to $to. '
      'Use a dedicated NEAR SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    // NEAR JSON-RPC does not natively support tx history.
    // In production, use NEAR Indexer, NearBlocks API, or similar.
    try {
      return [
        {
          'txid': '',
          'type': 'info',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'note': 'Transaction history requires NearBlocks indexer API for account $address',
          'chain': 'near',
        },
      ];
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/397'/0'
    // NEAR supports:
    // 1. Implicit accounts: 64 hex chars of Ed25519 public key
    // 2. Named accounts: human-readable .near addresses
    //
    // Generate an implicit account from seed:
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return hash.substring(0, 64);
  }

  @override
  bool validateAddress(String address) {
    // Implicit accounts: 64 hex characters
    final implicitRegex = RegExp(r'^[0-9a-f]{64}$');

    // Named accounts: alphanumeric, underscores, hyphens, dots; 2-64 chars
    // Must end with .near or .testnet for top-level, or be a sub-account
    final namedRegex = RegExp(r'^[a-z0-9][a-z0-9_\-]*(\.[a-z0-9][a-z0-9_\-]*)*\.near$');

    // System/top-level accounts without .near suffix (2-64 chars)
    final systemRegex = RegExp(r'^[a-z0-9][a-z0-9_\-\.]{1,63}$');

    return implicitRegex.hasMatch(address) ||
        namedRegex.hasMatch(address) ||
        systemRegex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // NEAR transaction fees are gas-based.
    // A typical transfer costs ~0.00045 NEAR in gas.
    return 0.00045;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/txns/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/address/$address';
  }

  void dispose() {
    _client.close();
  }
}
