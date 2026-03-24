import 'dart:convert';
import 'package:http/http.dart' as http;
import '../utils/constants.dart';

/// JSON-RPC client for Solana-compatible blockchain.
class RpcService {
  String _rpcUrl;
  int _requestId = 0;
  final http.Client _client;

  RpcService({String? rpcUrl, http.Client? client})
      : _rpcUrl = rpcUrl ?? AppConstants.defaultRpcUrl,
        _client = client ?? http.Client();

  String get rpcUrl => _rpcUrl;

  void setRpcUrl(String url) {
    _rpcUrl = url;
  }

  /// Make a JSON-RPC call.
  Future<Map<String, dynamic>> _call(String method, [List<dynamic>? params]) async {
    _requestId++;
    final body = jsonEncode({
      'jsonrpc': '2.0',
      'id': _requestId,
      'method': method,
      'params': params ?? [],
    });

    try {
      final response = await _client.post(
        Uri.parse(_rpcUrl),
        headers: {'Content-Type': 'application/json'},
        body: body,
      );

      if (response.statusCode != 200) {
        throw RpcException('HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw RpcException(
          error['message'] as String? ?? 'Unknown RPC error',
          code: error['code'] as int?,
        );
      }

      return json;
    } catch (e) {
      if (e is RpcException) rethrow;
      throw RpcException('Connection failed: $e');
    }
  }

  /// Get the balance of an account in lamports.
  Future<int> getBalance(String publicKey) async {
    final result = await _call('getBalance', [publicKey]);
    return (result['result']?['value'] as int?) ?? 0;
  }

  /// Get account info.
  Future<Map<String, dynamic>?> getAccountInfo(String publicKey) async {
    final result = await _call('getAccountInfo', [
      publicKey,
      {'encoding': 'base64'},
    ]);
    return result['result']?['value'] as Map<String, dynamic>?;
  }

  /// Get recent blockhash.
  Future<String> getLatestBlockhash() async {
    final result = await _call('getLatestBlockhash', [
      {'commitment': 'finalized'},
    ]);
    return result['result']['value']['blockhash'] as String;
  }

  /// Send a signed transaction.
  Future<String> sendTransaction(String signedTransaction) async {
    final result = await _call('sendTransaction', [
      signedTransaction,
      {'encoding': 'base64', 'preflightCommitment': 'confirmed'},
    ]);
    return result['result'] as String;
  }

  /// Get transaction signatures for an address.
  Future<List<Map<String, dynamic>>> getSignaturesForAddress(
    String address, {
    int limit = 20,
  }) async {
    final result = await _call('getSignaturesForAddress', [
      address,
      {'limit': limit},
    ]);
    final list = result['result'] as List<dynamic>? ?? [];
    return list.cast<Map<String, dynamic>>();
  }

  /// Get transaction details by signature.
  Future<Map<String, dynamic>?> getTransaction(String signature) async {
    final result = await _call('getTransaction', [
      signature,
      {'encoding': 'json', 'maxSupportedTransactionVersion': 0},
    ]);
    return result['result'] as Map<String, dynamic>?;
  }

  /// Get token accounts by owner.
  Future<List<Map<String, dynamic>>> getTokenAccountsByOwner(
    String owner,
    String programId,
  ) async {
    final result = await _call('getTokenAccountsByOwner', [
      owner,
      {'programId': programId},
      {'encoding': 'jsonParsed'},
    ]);
    final list = result['result']?['value'] as List<dynamic>? ?? [];
    return list.cast<Map<String, dynamic>>();
  }

  /// Get minimum balance for rent exemption.
  Future<int> getMinimumBalanceForRentExemption(int dataLength) async {
    final result = await _call('getMinimumBalanceForRentExemption', [dataLength]);
    return result['result'] as int;
  }

  /// Request airdrop (devnet/testnet only).
  Future<String> requestAirdrop(String publicKey, int lamports) async {
    final result = await _call('requestAirdrop', [publicKey, lamports]);
    return result['result'] as String;
  }

  /// Get cluster version info.
  Future<Map<String, dynamic>> getVersion() async {
    final result = await _call('getVersion');
    return result['result'] as Map<String, dynamic>;
  }

  /// Get epoch info.
  Future<Map<String, dynamic>> getEpochInfo() async {
    final result = await _call('getEpochInfo');
    return result['result'] as Map<String, dynamic>;
  }

  /// Get slot.
  Future<int> getSlot() async {
    final result = await _call('getSlot');
    return result['result'] as int;
  }

  /// Get vote accounts (for staking info).
  Future<Map<String, dynamic>> getVoteAccounts() async {
    final result = await _call('getVoteAccounts');
    return result['result'] as Map<String, dynamic>;
  }

  /// Check cluster health.
  Future<bool> isHealthy() async {
    try {
      await _call('getHealth');
      return true;
    } catch (_) {
      return false;
    }
  }

  /// Confirm a transaction.
  Future<bool> confirmTransaction(String signature, {Duration timeout = const Duration(seconds: 30)}) async {
    final deadline = DateTime.now().add(timeout);
    while (DateTime.now().isBefore(deadline)) {
      try {
        final result = await _call('getSignatureStatuses', [
          [signature],
        ]);
        final statuses = result['result']?['value'] as List<dynamic>?;
        if (statuses != null && statuses.isNotEmpty && statuses[0] != null) {
          final status = statuses[0] as Map<String, dynamic>;
          if (status['confirmationStatus'] == 'confirmed' ||
              status['confirmationStatus'] == 'finalized') {
            return status['err'] == null;
          }
        }
      } catch (_) {
        // Retry
      }
      await Future.delayed(const Duration(seconds: 2));
    }
    return false;
  }

  void dispose() {
    _client.close();
  }
}

class RpcException implements Exception {
  final String message;
  final int? code;

  RpcException(this.message, {this.code});

  @override
  String toString() => 'RpcException: $message${code != null ? ' (code: $code)' : ''}';
}
