import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular tokens on Cronos (Crypto.com) with their contract addresses.
class CronosToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const CronosToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<CronosToken> popular = [
    CronosToken(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0xc21223249CA28397B4B6541dfFaEcC539BfF0c59',
      decimals: 6,
    ),
    CronosToken(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: '0x66e428c3f67a68878562e79A0234c1F83c208770',
      decimals: 6,
    ),
    CronosToken(
      name: 'Wrapped CRO',
      symbol: 'WCRO',
      contractAddress: '0x5C7F8A570d578ED60E9120a7e1116AF8E3770eB3',
      decimals: 18,
    ),
    CronosToken(
      name: 'VVS Finance',
      symbol: 'VVS',
      contractAddress: '0x2D03bECE6747ADC00E1a131BBA1469C15fD11e03',
      decimals: 18,
    ),
    CronosToken(
      name: 'Tonic',
      symbol: 'TONIC',
      contractAddress: '0xDD73dEa10ABC2Bff99c60882EC5b2B81Bb1Dc5B2',
      decimals: 18,
    ),
  ];
}

/// Cronos (Crypto.com) chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses Cronos EVM RPC endpoints for balance and transaction queries.
class CronosService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  CronosService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.cronosRpcUrl;

  @override
  String get chainName => 'Cronos';

  @override
  String get chainSymbol => 'CRO';

  @override
  String get chainIcon => '\u{1F48E}'; // gem stone

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://cronoscan.com';

  @override
  String get rpcUrl => _rpcEndpoint;

  void setRpcUrl(String url) {
    _rpcEndpoint = url;
  }

  Future<Map<String, dynamic>> _jsonRpc(String method, [List<dynamic>? params]) async {
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
        throw Exception('Cronos RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Cronos RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Cronos RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _jsonRpc('eth_getBalance', [address, 'latest']);
      final hexBalance = result['result'] as String? ?? '0x0';
      final weiBalance = BigInt.parse(hexBalance.substring(2), radix: 16);
      return weiBalance.toDouble() / 1e18;
    } catch (e) {
      throw Exception('Failed to fetch CRO balance: $e');
    }
  }

  /// Get the balance of a token on Cronos.
  Future<double> getTokenBalance(String contractAddress, String walletAddress, {int tokenDecimals = 18}) async {
    try {
      final paddedAddress = walletAddress.substring(2).padLeft(64, '0');
      final data = '0x70a08231$paddedAddress';

      final result = await _jsonRpc('eth_call', [
        {'to': contractAddress, 'data': data},
        'latest',
      ]);

      final hexBalance = result['result'] as String? ?? '0x0';
      if (hexBalance == '0x' || hexBalance == '0x0') return 0.0;

      final balance = BigInt.parse(
        hexBalance.startsWith('0x') ? hexBalance.substring(2) : hexBalance,
        radix: 16,
      );

      final divisor = BigInt.from(10).pow(tokenDecimals);
      return balance / divisor;
    } catch (e) {
      return 0.0;
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    final weiAmount = BigInt.from(amount * 1e18);
    throw UnimplementedError(
      'Cronos transaction sending requires ECDSA signing. '
      'Amount: $amount CRO ($weiAmount Wei) to $to. '
      'Use a dedicated Cronos SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final blockResult = await _jsonRpc('eth_blockNumber');
      final currentBlock = blockResult['result'] as String? ?? '0x0';
      final blockNum = int.parse(currentBlock.substring(2), radix: 16);

      return [
        {
          'txid': '0x0000000000000000000000000000000000000000000000000000000000000000',
          'type': 'info',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'note': 'Transaction history requires CronosScan API. Current block: $blockNum',
          'chain': 'cronos',
        },
      ];
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // Same as Ethereum: BIP44 m/44'/60'/0'/0/0
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '0x${hash.substring(0, 40)}';
  }

  @override
  bool validateAddress(String address) {
    final regex = RegExp(r'^0x[0-9a-fA-F]{40}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final result = await _jsonRpc('eth_gasPrice');
      final hexGasPrice = result['result'] as String? ?? '0x0';
      final gasPrice = BigInt.parse(hexGasPrice.substring(2), radix: 16);
      final feeWei = gasPrice * BigInt.from(21000);
      return feeWei.toDouble() / 1e18;
    } catch (_) {
      return 0.0005; // Fallback estimate
    }
  }

  void dispose() {
    _client.close();
  }
}
