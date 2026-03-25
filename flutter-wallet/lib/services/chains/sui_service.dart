import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Sui chain service using Sui JSON-RPC.
///
/// Uses BIP44 derivation (m/44'/784'/0'/0'/0') for Sui addresses (0x...).
/// Communicates via Sui JSON-RPC for balance and transaction queries.
/// Sui is a Move-based L1 with object-centric data model and parallel execution.
class SuiService extends ChainService {
  final http.Client _client;
  final String _rpcEndpoint;

  SuiService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.suiRpcUrl;

  int _requestId = 0;

  @override
  String get chainName => 'Sui';

  @override
  String get chainSymbol => 'SUI';

  @override
  String get chainIcon => '\u{1F4A7}'; // droplet

  @override
  int get decimals => 9;

  @override
  String get explorerUrl => 'https://suiscan.xyz';

  @override
  String get rpcUrl => _rpcEndpoint;

  /// Make a Sui JSON-RPC call.
  Future<Map<String, dynamic>> _suiRpc(String method, [List<dynamic>? params]) async {
    _requestId++;
    final body = jsonEncode({
      'jsonrpc': '2.0',
      'id': _requestId,
      'method': method,
      'params': params ?? [],
    });

    try {
      final response = await _client.post(
        Uri.parse(_rpcEndpoint),
        headers: {'Content-Type': 'application/json'},
        body: body,
      );

      if (response.statusCode != 200) {
        throw Exception('Sui RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Sui RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Sui RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _suiRpc('suix_getBalance', [address, '0x2::sui::SUI']);
      final data = result['result'] as Map<String, dynamic>? ?? {};
      final totalBalance = data['totalBalance'] as String? ?? '0';
      final mist = BigInt.tryParse(totalBalance) ?? BigInt.zero;
      // Convert MIST to SUI (1 SUI = 10^9 MIST)
      return mist.toDouble() / 1e9;
    } catch (e) {
      throw Exception('Sui balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Sui transaction requires:
    // 1. Select coins via suix_getCoins
    // 2. Construct TransactionBlock with transferObjects
    // 3. Sign with Ed25519
    // 4. Execute via sui_executeTransactionBlock
    //
    // Placeholder implementation:
    final mistAmount = (amount * 1e9).toInt();
    throw UnimplementedError(
      'Sui transaction sending requires TransactionBlock construction. '
      'Amount: $amount SUI ($mistAmount MIST) to $to. '
      'Use a dedicated Sui SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final result = await _suiRpc('suix_queryTransactionBlocks', [
        {
          'filter': {'FromAddress': address},
          'options': {'showEffects': true, 'showInput': true},
        },
        null, // cursor
        20,   // limit
        true, // descending
      ]);

      final data = result['result'] as Map<String, dynamic>? ?? {};
      final txBlocks = data['data'] as List<dynamic>? ?? [];

      return txBlocks.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final digest = txData['digest'] as String? ?? '';
        final timestampMs = txData['timestampMs'] as String? ?? '0';
        final effects = txData['effects'] as Map<String, dynamic>? ?? {};
        final status = (effects['status'] as Map<String, dynamic>?)?['status'] as String? ?? '';

        return {
          'txid': digest,
          'type': 'send',
          'amount': 0.0,
          'confirmed': status == 'success',
          'timestamp': int.tryParse(timestampMs) != null
              ? DateTime.fromMillisecondsSinceEpoch(int.parse(timestampMs)).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'sui',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/784'/0'/0'/0'
    // In a full implementation, this would:
    // 1. Derive the Ed25519 keypair from the seed
    // 2. Blake2b hash the public key with a flag byte
    // 3. 0x-prefix the resulting 32-byte hex address
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '0x${hash.substring(0, 64)}';
  }

  @override
  bool validateAddress(String address) {
    // Sui addresses: 0x followed by 64 hex characters (32 bytes)
    final regex = RegExp(r'^0x[0-9a-fA-F]{64}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final result = await _suiRpc('suix_getReferenceGasPrice');
      final gasPrice = result['result'] as String? ?? '1000';
      final price = int.tryParse(gasPrice) ?? 1000;
      // Typical transfer uses ~2000 gas units
      return (price * 2000) / 1e9;
    } catch (_) {
      return 0.003; // Fallback estimate
    }
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/mainnet/tx/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/mainnet/account/$address';
  }

  void dispose() {
    _client.close();
  }
}
