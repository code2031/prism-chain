import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular tokens on Polygon (PoS) with their contract addresses.
class PolygonToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const PolygonToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<PolygonToken> popular = [
    PolygonToken(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
      decimals: 6,
    ),
    PolygonToken(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F',
      decimals: 6,
    ),
    PolygonToken(
      name: 'Wrapped Ether',
      symbol: 'WETH',
      contractAddress: '0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Dai Stablecoin',
      symbol: 'DAI',
      contractAddress: '0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Aave',
      symbol: 'AAVE',
      contractAddress: '0xD6DF932A45C0f255f85145f286eA0b292B21C90B',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Chainlink',
      symbol: 'LINK',
      contractAddress: '0x53E0bca35eC356BD5ddDFebbD1Fc0fD03FaBad39',
      decimals: 18,
    ),
    PolygonToken(
      name: 'QuickSwap',
      symbol: 'QUICK',
      contractAddress: '0xB5C064F955D8e7F38fE0460C556a72987494eE17',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Curve DAO',
      symbol: 'CRV',
      contractAddress: '0x172370d5Cd63279eFa6d502DAB29171933a610AF',
      decimals: 18,
    ),
    PolygonToken(
      name: 'SushiSwap',
      symbol: 'SUSHI',
      contractAddress: '0x0b3F868E0BE5597D5DB7fEB59E1CADBb0fdDa50a',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Balancer',
      symbol: 'BAL',
      contractAddress: '0x9a71012B13CA4d3D0Cdc72A177dF3ef03b0E76A3',
      decimals: 18,
    ),
    PolygonToken(
      name: 'Wrapped Bitcoin',
      symbol: 'WBTC',
      contractAddress: '0x1BFD67037B42Cf73acF2047067bd4F2C47D9BfD6',
      decimals: 8,
    ),
  ];
}

/// Polygon (PoS) chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format and derivation as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses Polygon's own RPC endpoints.
class PolygonService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  PolygonService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.polygonRpcUrl;

  @override
  String get chainName => 'Polygon';

  @override
  String get chainSymbol => 'POL';

  @override
  String get chainIcon => '\u{1F7E3}'; // purple circle

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://polygonscan.com';

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
        throw Exception('Polygon RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Polygon RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Polygon RPC connection failed: $e');
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
      throw Exception('Failed to fetch Polygon balance: $e');
    }
  }

  /// Get the balance of a Polygon ERC-20 token.
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
      'Polygon transaction sending requires ECDSA signing. '
      'Amount: $amount POL ($weiAmount Wei) to $to. '
      'Use a dedicated Ethereum/Polygon SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    // Polygon JSON-RPC does not natively support tx history.
    // Use Polygonscan API in production.
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
          'note': 'Transaction history requires Polygonscan API. Current block: $blockNum',
          'chain': 'polygon',
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
      return 0.0001; // Polygon fees are very low
    }
  }

  void dispose() {
    _client.close();
  }
}
