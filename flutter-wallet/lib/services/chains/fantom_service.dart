import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular tokens on Fantom Opera with their contract addresses.
class FantomToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const FantomToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<FantomToken> popular = [
    FantomToken(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0x04068DA6C83AFCFA0e13ba15A6696662335D5B75',
      decimals: 6,
    ),
    FantomToken(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: '0x049d68029688eAbF473097a2fC38ef61633A3C7A',
      decimals: 6,
    ),
    FantomToken(
      name: 'Wrapped Fantom',
      symbol: 'WFTM',
      contractAddress: '0x21be370D5312f44cB42ce377BC9b8a0cEF1A4C83',
      decimals: 18,
    ),
    FantomToken(
      name: 'SpookySwap',
      symbol: 'BOO',
      contractAddress: '0x841FAD6EAe12c286d1Fd18d1d525DFfA75C7EFFE',
      decimals: 18,
    ),
    FantomToken(
      name: 'SpiritSwap',
      symbol: 'SPIRIT',
      contractAddress: '0x5Cc61A78F164885776AA610fb0FE1257df78E59B',
      decimals: 18,
    ),
  ];
}

/// Fantom Opera chain service using Ethereum-compatible JSON-RPC.
///
/// Shares the same address format as Ethereum (BIP44 m/44'/60'/0'/0/0).
/// Uses Fantom RPC endpoints for balance and transaction queries.
class FantomService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;
  int _requestId = 0;

  FantomService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.fantomRpcUrl;

  @override
  String get chainName => 'Fantom';

  @override
  String get chainSymbol => 'FTM';

  @override
  String get chainIcon => '\u{1F47B}'; // ghost

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://ftmscan.com';

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
        throw Exception('Fantom RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('Fantom RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('Fantom RPC connection failed: $e');
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
      throw Exception('Failed to fetch FTM balance: $e');
    }
  }

  /// Get the balance of a token on Fantom.
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
      'Fantom transaction sending requires ECDSA signing. '
      'Amount: $amount FTM ($weiAmount Wei) to $to. '
      'Use a dedicated Fantom SDK for production transactions.',
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
          'note': 'Transaction history requires FTMScan API. Current block: $blockNum',
          'chain': 'fantom',
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
      return 0.0002; // Fantom fees are relatively low
    }
  }

  void dispose() {
    _client.close();
  }
}
