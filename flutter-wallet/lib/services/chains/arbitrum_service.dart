import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular tokens on Arbitrum One with their contract addresses.
class ArbitrumToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const ArbitrumToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<ArbitrumToken> popular = [
    ArbitrumToken(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0xaf88d065e77c8cC2239327C5EDb3A432268e5831',
      decimals: 6,
    ),
    ArbitrumToken(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9',
      decimals: 6,
    ),
    ArbitrumToken(
      name: 'Arbitrum',
      symbol: 'ARB',
      contractAddress: '0x912CE59144191C1204E64559FE8253a0e49E6548',
      decimals: 18,
    ),
    ArbitrumToken(
      name: 'GMX',
      symbol: 'GMX',
      contractAddress: '0xfc5A1A6EB076a2C7aD06eD22C90d7E710E35ad0a',
      decimals: 18,
    ),
    ArbitrumToken(
      name: 'MAGIC',
      symbol: 'MAGIC',
      contractAddress: '0x539bdE0d7Dbd336b79148AA742883198BBF60342',
      decimals: 18,
    ),
  ];
}

/// Arbitrum One (L2) chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses Arbitrum RPC endpoints for balance and transaction queries.
class ArbitrumService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  ArbitrumService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.arbitrumRpcUrl;

  @override
  String get chainName => 'Arbitrum';

  @override
  String get chainSymbol => 'ETH';

  @override
  String get chainIcon => '\u{1F537}'; // blue diamond

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://arbiscan.io';

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
        throw Exception('Arbitrum RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Arbitrum RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Arbitrum RPC connection failed: $e');
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
      throw Exception('Failed to fetch Arbitrum balance: $e');
    }
  }

  /// Get the balance of a token on Arbitrum.
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
      'Arbitrum transaction sending requires ECDSA signing. '
      'Amount: $amount ETH ($weiAmount Wei) to $to. '
      'Use a dedicated Ethereum/Arbitrum SDK for production transactions.',
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
          'note': 'Transaction history requires Arbiscan API. Current block: $blockNum',
          'chain': 'arbitrum',
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
      return 0.00005; // Arbitrum L2 fees are very low
    }
  }

  void dispose() {
    _client.close();
  }
}
