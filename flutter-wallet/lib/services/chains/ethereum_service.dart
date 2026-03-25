import 'dart:convert';
import 'package:http/http.dart' as http;
import '../../utils/constants.dart';
import 'chain_service.dart';

/// Popular ERC-20 tokens with their contract addresses (Ethereum mainnet).
class ERC20Token {
  final String name;
  final String symbol;
  final String contractAddress;
  final int decimals;

  const ERC20Token({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.decimals = 18,
  });

  static const List<ERC20Token> popular = [
    ERC20Token(
      name: 'Tether USD',
      symbol: 'USDT',
      contractAddress: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
      decimals: 6,
    ),
    ERC20Token(
      name: 'USD Coin',
      symbol: 'USDC',
      contractAddress: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
      decimals: 6,
    ),
    ERC20Token(
      name: 'Dai Stablecoin',
      symbol: 'DAI',
      contractAddress: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Wrapped Ether',
      symbol: 'WETH',
      contractAddress: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Uniswap',
      symbol: 'UNI',
      contractAddress: '0x1f9840a85d5aF5bf1D1762F925BDADdC4201F984',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Chainlink',
      symbol: 'LINK',
      contractAddress: '0x514910771AF9Ca656af840dff83E8264EcF986CA',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Shiba Inu',
      symbol: 'SHIB',
      contractAddress: '0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Pepe',
      symbol: 'PEPE',
      contractAddress: '0x6982508145454Ce325dDbE47a25d4ec3d2311933',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Aave',
      symbol: 'AAVE',
      contractAddress: '0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Lido DAO',
      symbol: 'LDO',
      contractAddress: '0x5A98FcBEA516Cf06857215779Fd812CA3beF1B32',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Maker',
      symbol: 'MKR',
      contractAddress: '0x9f8F72aA9304c8B593d555F12eF6589cC3A579A2',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Curve DAO',
      symbol: 'CRV',
      contractAddress: '0xD533a949740bb3306d119CC777fa900bA034cd52',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Synthetix',
      symbol: 'SNX',
      contractAddress: '0xC011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F',
      decimals: 18,
    ),
    ERC20Token(
      name: 'The Graph',
      symbol: 'GRT',
      contractAddress: '0xc944E90C64B2c07662A292be6244BDf05Cda44a7',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Fetch.ai',
      symbol: 'FET',
      contractAddress: '0xaea46A60368A7bD060eec7DF8CBa43b7EF41Ad85',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Render',
      symbol: 'RNDR',
      contractAddress: '0x6De037ef9aD2725EB40118Bb1702EBb27e4Aeb24',
      decimals: 18,
    ),
    ERC20Token(
      name: 'ApeCoin',
      symbol: 'APE',
      contractAddress: '0x4d224452801ACEd8B2F0aebE155379bb5D594381',
      decimals: 18,
    ),
    ERC20Token(
      name: 'The Sandbox',
      symbol: 'SAND',
      contractAddress: '0x3845badAde8e6dFF049820680d1F14bD3903a5d0',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Decentraland',
      symbol: 'MANA',
      contractAddress: '0x0F5D2fB29fb7d3CFeE444a200298f468908cC942',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Ethereum Name Service',
      symbol: 'ENS',
      contractAddress: '0xC18360217D8F7Ab5e7c516566761Ea12Ce7F9D72',
      decimals: 18,
    ),
    ERC20Token(
      name: '1inch',
      symbol: '1INCH',
      contractAddress: '0x111111111117dC0aa78b770fA6A738034120C302',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Compound',
      symbol: 'COMP',
      contractAddress: '0xc00e94Cb662C3520282E6f5717214004A7f26888',
      decimals: 18,
    ),
    ERC20Token(
      name: 'Wrapped Bitcoin',
      symbol: 'WBTC',
      contractAddress: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
      decimals: 8,
    ),
    ERC20Token(
      name: 'Staked Ether',
      symbol: 'STETH',
      contractAddress: '0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84',
      decimals: 18,
    ),
  ];
}

/// Ethereum chain service using JSON-RPC.
///
/// Uses BIP44 derivation (m/44'/60'/0'/0/0) for standard ETH addresses (0x...).
/// Communicates via standard Ethereum JSON-RPC for balance/tx queries.
class EthereumService extends ChainService {
  final http.Client _client;
  String _rpcEndpoint;

  EthereumService({http.Client? client, String? rpcUrl})
      : _client = client ?? http.Client(),
        _rpcEndpoint = rpcUrl ?? AppConstants.ethereumRpcUrl;

  int _requestId = 0;

  @override
  String get chainName => 'Ethereum';

  @override
  String get chainSymbol => 'ETH';

  @override
  String get chainIcon => '\u{1F535}'; // blue circle

  @override
  int get decimals => 18;

  @override
  String get explorerUrl => 'https://etherscan.io';

  @override
  String get rpcUrl => _rpcEndpoint;

  /// Set a custom RPC endpoint.
  void setRpcUrl(String url) {
    _rpcEndpoint = url;
  }

  /// Make an Ethereum JSON-RPC call.
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
        throw Exception('ETH RPC HTTP ${response.statusCode}: ${response.body}');
      }

      final json = jsonDecode(response.body) as Map<String, dynamic>;

      if (json.containsKey('error')) {
        final error = json['error'] as Map<String, dynamic>;
        throw Exception('ETH RPC Error: ${error['message']}');
      }

      return json;
    } catch (e) {
      if (e is Exception) rethrow;
      throw Exception('ETH RPC connection failed: $e');
    }
  }

  @override
  Future<double> getBalance(String address) async {
    try {
      final result = await _jsonRpc('eth_getBalance', [address, 'latest']);
      final hexBalance = result['result'] as String? ?? '0x0';
      final weiBalance = BigInt.parse(hexBalance.substring(2), radix: 16);
      // Convert Wei to ETH (1 ETH = 10^18 Wei)
      return weiBalance.toDouble() / 1e18;
    } catch (e) {
      throw Exception('Failed to fetch ETH balance: $e');
    }
  }

  /// Get the balance of an ERC-20 token.
  Future<double> getERC20Balance(String contractAddress, String walletAddress, {int tokenDecimals = 18}) async {
    try {
      // balanceOf(address) function selector: 0x70a08231
      // Pad the address to 32 bytes (remove 0x prefix, pad to 64 hex chars)
      final paddedAddress = walletAddress.substring(2).padLeft(64, '0');
      final data = '0x70a08231$paddedAddress';

      final result = await _jsonRpc('eth_call', [
        {
          'to': contractAddress,
          'data': data,
        },
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

  /// Get balances for all popular ERC-20 tokens.
  Future<Map<String, double>> getAllERC20Balances(String walletAddress) async {
    final balances = <String, double>{};

    for (final token in ERC20Token.popular) {
      try {
        final balance = await getERC20Balance(
          token.contractAddress,
          walletAddress,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          balances[token.symbol] = balance;
        }
      } catch (_) {
        continue;
      }
    }

    return balances;
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Full ETH transaction requires:
    // 1. Get nonce via eth_getTransactionCount
    // 2. Get gas price via eth_gasPrice
    // 3. Estimate gas via eth_estimateGas
    // 4. Construct RLP-encoded transaction
    // 5. Sign with ECDSA (secp256k1)
    // 6. Broadcast via eth_sendRawTransaction
    //
    // Placeholder implementation:
    final weiAmount = BigInt.from(amount * 1e18);
    throw UnimplementedError(
      'ETH transaction sending requires ECDSA signing. '
      'Amount: $amount ETH ($weiAmount Wei) to $to. '
      'Use a dedicated Ethereum SDK for production transactions.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    // Ethereum JSON-RPC does not natively support tx history.
    // In production, you would use Etherscan API, Alchemy, or similar.
    // This returns a mock structure for the UI.
    try {
      // Try to get the current block number for context
      final blockResult = await _jsonRpc('eth_blockNumber');
      final currentBlock = blockResult['result'] as String? ?? '0x0';
      final blockNum = int.parse(currentBlock.substring(2), radix: 16);

      // Return a placeholder indicating the feature needs an indexer
      return [
        {
          'txid': '0x0000000000000000000000000000000000000000000000000000000000000000',
          'type': 'info',
          'amount': 0.0,
          'confirmed': true,
          'timestamp': DateTime.now().toIso8601String(),
          'note': 'Transaction history requires an indexer API (e.g., Etherscan). Current block: $blockNum',
          'chain': 'ethereum',
        },
      ];
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // BIP44 derivation path: m/44'/60'/0'/0/0
    // In a full implementation, this would:
    // 1. Derive child key using BIP44 path
    // 2. Get the public key (secp256k1)
    // 3. Keccak-256 hash of the public key
    // 4. Take last 20 bytes as the address
    // 5. Add EIP-55 checksum
    //
    // Deterministic placeholder based on seed bytes:
    final hash = seed.take(20).map((b) => b.toRadixString(16).padLeft(2, '0')).join();
    return '0x${hash.substring(0, 40)}';
  }

  @override
  bool validateAddress(String address) {
    // Standard Ethereum address: 0x followed by 40 hex characters
    final regex = RegExp(r'^0x[0-9a-fA-F]{40}$');
    return regex.hasMatch(address);
  }

  @override
  Future<double> estimateFee() async {
    try {
      final result = await _jsonRpc('eth_gasPrice');
      final hexGasPrice = result['result'] as String? ?? '0x0';
      final gasPrice = BigInt.parse(hexGasPrice.substring(2), radix: 16);
      // Standard ETH transfer = 21000 gas
      final feeWei = gasPrice * BigInt.from(21000);
      return feeWei.toDouble() / 1e18;
    } catch (_) {
      return 0.001; // Fallback estimate
    }
  }

  /// Get the current gas price in Gwei.
  Future<double> getGasPriceGwei() async {
    try {
      final result = await _jsonRpc('eth_gasPrice');
      final hexGasPrice = result['result'] as String? ?? '0x0';
      final gasPrice = BigInt.parse(hexGasPrice.substring(2), radix: 16);
      return gasPrice.toDouble() / 1e9; // Wei to Gwei
    } catch (_) {
      return 20.0; // Fallback: 20 Gwei
    }
  }

  void dispose() {
    _client.close();
  }
}
