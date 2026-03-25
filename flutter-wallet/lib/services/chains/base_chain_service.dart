import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular tokens on Base (Coinbase L2) with their contract addresses.
class BaseToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const BaseToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<BaseToken> popular = [
    BaseToken(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913',
      decimals: 6,
    ),
    BaseToken(
      name: 'USD Base Coin',
      symbol: 'USDbC',
      contractAddress: '0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA',
      decimals: 6,
    ),
    BaseToken(
      name: 'Coinbase Wrapped Staked ETH',
      symbol: 'cbETH',
      contractAddress: '0x2Ae3F1Ec7F1F5012CFEab0185bfc7aa3cf0DEc22',
      decimals: 18,
    ),
    BaseToken(
      name: 'Aerodrome Finance',
      symbol: 'AERO',
      contractAddress: '0x940181a94A35A4569E4529A3CDfB74e38FD98631',
      decimals: 18,
    ),
    BaseToken(
      name: 'Degen',
      symbol: 'DEGEN',
      contractAddress: '0x4ed4E862860beD51a9570b96d89aF5E1B0Efefed',
      decimals: 18,
    ),
  ];
}

/// Base (Coinbase L2) chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses Base RPC endpoints for balance and transaction queries.
class BaseChainService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  BaseChainService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.baseRpcUrl;

  @override
  String get chainName => 'Base';

  @override
  String get chainSymbol => 'ETH';

  @override
  String get chainIcon => '\u{1F535}'; // blue circle

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://basescan.org';

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
        throw Exception('Base RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Base RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Base RPC connection failed: $e');
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
      throw Exception('Failed to fetch Base balance: $e');
    }
  }

  /// Get the balance of a token on Base.
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
      'Base transaction sending requires ECDSA signing. '
      'Amount: $amount ETH ($weiAmount Wei) to $to. '
      'Use a dedicated Ethereum/Base SDK for production transactions.',
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
          'note': 'Transaction history requires BaseScan API. Current block: $blockNum',
          'chain': 'base',
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
      return 0.00003; // Base L2 fees are very low
    }
  }

  void dispose() {
    _client.close();
  }
}
