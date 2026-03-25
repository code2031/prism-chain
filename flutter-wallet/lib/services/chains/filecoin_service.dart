import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Filecoin chain service using Filecoin JSON-RPC (Glif nodes).
///
/// Uses BIP44 derivation (m/44'/461'/0'/0/0) for Filecoin addresses (f1/f4...).
/// Communicates via Filecoin Lotus JSON-RPC for balance and transaction queries.
/// Filecoin is a decentralized storage network with Proof of Replication/Spacetime.
class FilecoinService extends ChainService {
  final http.Client _client;
  final String _rpcEndpoint;

  FilecoinService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.filecoinRpcUrl;

  int _requestId = 0;

  @override
  String get chainName => 'Filecoin';

  @override
  String get chainSymbol => 'FIL';

  @override
  String get chainIcon => '\u{1F4BE}'; // floppy disk

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://filfox.info/en';

  @override
  String get rpcUrl => _rpcEndpoint;

  /// Make a Filecoin Lotus JSON-RPC call.
  Future<Map<String, dynamic>> _filRpc(String method, [List<dynamic>? params]) async {
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
        throw Exception('Filecoin RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Filecoin RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Filecoin RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _filRpc('Filecoin.WalletBalance', [address]);
      final balanceAttoFil = result['result'] as String? ?? '0';
      final attoFil = BigInt.tryParse(balanceAttoFil) ?? BigInt.zero;
      // Convert attoFIL to FIL (1 FIL = 10^18 attoFIL)
      return attoFil.toDouble() / 1e18;
    } catch (e) {
      throw Exception('Filecoin balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Filecoin transaction requires:
    // 1. Get nonce via Filecoin.MpoolGetNonce
    // 2. Construct Message (from, to, value, gas params)
    // 3. Estimate gas via Filecoin.GasEstimateMessageGas
    // 4. Sign with secp256k1 or BLS
    // 5. Submit via Filecoin.MpoolPush
    //
    // Placeholder implementation:
    final attoFilAmount = BigInt.from(amount * 1e6) * BigInt.from(10).pow(12);
    throw UnimplementedError(
      'Filecoin transaction sending requires CBOR construction and signing. '
      'Amount: $amount FIL ($attoFilAmount attoFIL) to $to. '
      'Use a dedicated Filecoin SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    // Filecoin JSON-RPC does not natively support tx history.
    // In production, use Filfox API or Glif indexer.
    try {
      return [
        {
          'txid': '',
          'type': 'info',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'note': 'Transaction history requires Filfox indexer API for address $address',
          'chain': 'filecoin',
        },
      ];
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/461'/0'/0/0
    // In a full implementation, this would:
    // 1. Derive the secp256k1 keypair from the seed
    // 2. Blake2b-160 hash of the public key
    // 3. Encode as f1... (secp256k1) address with checksum
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'f1${hash.substring(0, 38)}';
  }

  @override
  bool validateAddress(String address) {
    // Filecoin address types:
    // f0... - ID addresses
    // f1... - secp256k1 addresses (41 chars)
    // f2... - Actor addresses
    // f3... - BLS addresses (86 chars)
    // f4... - Delegated addresses (FEVM, variable length)
    // t-prefix for testnet
    final f1Regex = RegExp(r'^[ft]1[a-z2-7]{39}$');
    final f3Regex = RegExp(r'^[ft]3[a-z2-7]{84}$');
    final f0Regex = RegExp(r'^[ft]0\d{1,18}$');
    final f2Regex = RegExp(r'^[ft]2[a-z2-7]{39}$');
    final f4Regex = RegExp(r'^[ft]4[a-z0-9]{3,}$');

    return f1Regex.hasMatch(address) ||
        f3Regex.hasMatch(address) ||
        f0Regex.hasMatch(address) ||
        f2Regex.hasMatch(address) ||
        f4Regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Filecoin gas fees vary but a typical transfer costs ~0.0001 FIL
    return 0.0001;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/message/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/address/$address';
  }

  void dispose() {
    _client.close();
  }
}
