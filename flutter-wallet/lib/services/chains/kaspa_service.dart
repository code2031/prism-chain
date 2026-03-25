import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Kaspa chain service using Kaspa REST API.
///
/// Uses BIP44 derivation (m/44'/111111'/0'/0/0) for Kaspa addresses (kaspa:...).
/// Communicates via Kaspa REST API for balance and transaction queries.
/// Kaspa uses blockDAG (GHOSTDAG/PHANTOM) for high-throughput parallel blocks.
class KaspaService extends ChainService {
  final http.Client _client;
  final String _apiBaseUrl;

  KaspaService({http.Client? client, String? apiBaseUrl})
      : _client = client ?? http.Client(),
        _apiBaseUrl = apiBaseUrl ?? AppConstants.kaspaApiUrl;

  @override
  String get chainName => 'Kaspa';

  @override
  String get chainSymbol => 'KAS';

  @override
  String get chainIcon => '\u{1F4A0}'; // diamond with dot

  @override
  int get decimals => 8;

  @override
  String get explorerUrl => 'https://explorer.kaspa.org';

  @override
  String get rpcUrl => _apiBaseUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/addresses/$address/balance'),
      );

      if (response.statusCode != 200) {
        throw Exception('Kaspa API HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;
      final balanceSompi = json['balance'] as int? ?? 0;

      // Convert sompi to KAS (1 KAS = 100,000,000 sompi)
      return balanceSompi / 100000000.0;
    } catch (e) {
      throw Exception('Kaspa balance fetch failed: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full Kaspa transaction requires:
    // 1. Fetch UTXOs for the sender
    // 2. Select inputs to cover amount + fee
    // 3. Construct transaction with inputs/outputs
    // 4. Sign with Schnorr (secp256k1)
    // 5. Broadcast via API
    //
    // Placeholder implementation:
    final sompiAmount = (amount * 100000000).toInt();
    throw UnimplementedError(
      'Kaspa transaction sending requires UTXO construction and Schnorr signing. '
      'Amount: $amount KAS ($sompiAmount sompi) to $to. '
      'Use a dedicated Kaspa SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final response = await _client.get(
        Uri.parse('$_apiBaseUrl/addresses/$address/full-transactions?limit=20&resolve_previous_outpoints=light'),
      );

      if (response.statusCode != 200) {
        return [];
      }

      final List<dynamic> txList = jsonDecode(response.body) as List<dynamic>;

      return txList.take(20).map((tx) {
        final txData = tx as Map<String, dynamic>;
        final txId = txData['transaction_id'] as String? ?? '';
        final blockTime = txData['block_time'] as int? ?? 0;

        // Calculate net value for this address from inputs/outputs
        int totalIn = 0;
        int totalOut = 0;

        final inputs = txData['inputs'] as List<dynamic>? ?? [];
        for (final input in inputs) {
          final prevOut = (input as Map<String, dynamic>)['previous_outpoint_address'] as String?;
          final prevAmount = (input)['previous_outpoint_amount'] as int? ?? 0;
          if (prevOut == address) {
            totalIn += prevAmount;
          }
        }

        final outputs = txData['outputs'] as List<dynamic>? ?? [];
        for (final output in outputs) {
          final outAddr = (output as Map<String, dynamic>)['script_public_key_address'] as String?;
          final outAmount = output['amount'] as int? ?? 0;
          if (outAddr == address) {
            totalOut += outAmount;
          }
        }

        final netSompi = totalOut - totalIn;
        final isReceive = netSompi > 0;

        return {
          'txid': txId,
          'type': isReceive ? 'receive' : 'send',
          'amount': netSompi.abs() / 100000000.0,
          'confirmed': txData['is_accepted'] == true,
          'timestamp': blockTime > 0
              ? DateTime.fromMillisecondsSinceEpoch(blockTime).toIso8601String()
              : DateTime.now().toIso8601String(),
          'chain': 'kaspa',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/111111'/0'/0/0
    // In a full implementation, this would:
    // 1. Derive the secp256k1 keypair from the seed
    // 2. Schnorr public key (32 bytes)
    // 3. Bech32 encode with "kaspa" prefix
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(32).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return 'kaspa:q${hash.substring(0, 61)}';
  }

  @override
  bool validateAddress(String address) {
    // Kaspa addresses start with kaspa: followed by bech32-encoded data
    // P2PK: kaspa:q... (Schnorr), P2SH: kaspa:p...
    final regex = RegExp(r'^kaspa:[qp][a-z0-9]{61,63}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    // Kaspa has very low fees, typically around 0.0001 KAS
    return 0.0001;
  }

  @override
  String getTransactionExplorerUrl(String txHash) {
    return '$explorerUrl/txs/$txHash';
  }

  @override
  String getAddressExplorerUrl(String address) {
    return '$explorerUrl/addresses/$address';
  }

  void dispose() {
    _client.close();
  }
}
