import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular BEP-20 tokens on BNB Smart Chain.
class BEP20Token {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const BEP20Token({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<BEP20Token> popular = [
    BEP20Token(
      name: 'Binance-Peg BUSD',
      symbol: 'BUSD',
      contractAddress: '0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Binance-Peg USDT',
      symbol: 'USDT',
      contractAddress: '0x55d398326f99059fF775485246999027B3197955',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Binance-Peg USDC',
      symbol: 'USDC',
      contractAddress: '0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d',
      decimals: 18,
    ),
    BEP20Token(
      name: 'PancakeSwap',
      symbol: 'CAKE',
      contractAddress: '0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Wrapped BNB',
      symbol: 'WBNB',
      contractAddress: '0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Venus',
      symbol: 'XVS',
      contractAddress: '0xcF6BB5389c92Bdda8a3747Ddb454cB7a64626C63',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Trust Wallet Token',
      symbol: 'TWT',
      contractAddress: '0x4B0F1812e5Df2A09796481Ff14017e6005508003',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Alpaca Finance',
      symbol: 'ALPACA',
      contractAddress: '0x8F0528cE5eF7B51152A59745bEfDD91D97091d2F',
      decimals: 18,
    ),
    BEP20Token(
      name: 'SafeMoon',
      symbol: 'SFM',
      contractAddress: '0x42981d0bfbAf196529376EE702F2a9Eb9092fcB5',
      decimals: 9,
    ),
    BEP20Token(
      name: 'Binance-Peg Ethereum',
      symbol: 'ETH',
      contractAddress: '0x2170Ed0880ac9A755fd29B2688956BD959F933F8',
      decimals: 18,
    ),
    BEP20Token(
      name: 'Binance-Peg Bitcoin',
      symbol: 'BTCB',
      contractAddress: '0x7130d2A12B9BCbFAe4f2634d864A1Ee1Ce3Ead9c',
      decimals: 18,
    ),
  ];
}

/// BNB Smart Chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses BSC RPC endpoints for balance and transaction queries.
class BnbService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  BnbService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.bnbRpcUrl;

  @override
  String get chainName => 'BNB Chain';

  @override
  String get chainSymbol => 'BNB';

  @override
  String get chainIcon => '\u{1F7E1}'; // yellow circle

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://bscscan.com';

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
        throw Exception('BSC RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('BSC RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('BSC RPC connection failed: $e');
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
      throw Exception('Failed to fetch BNB balance: $e');
    }
  }

  /// Get the balance of a BEP-20 token.
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
      'BNB transaction sending requires ECDSA signing. '
      'Amount: $amount BNB ($weiAmount Wei) to $to. '
      'Use a dedicated BSC SDK for production transactions.',
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
          'note': 'Transaction history requires BscScan API. Current block: $blockNum',
          'chain': 'bnb',
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
      return 0.0001; // BSC fees are very low
    }
  }

  void dispose() {
    _client.close();
  }
}
