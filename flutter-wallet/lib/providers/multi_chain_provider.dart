import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/chain.dart';
import '../services/chains/chain_service.dart';
import '../services/chains/bitcoin_service.dart';
import '../services/chains/ethereum_service.dart';
import '../services/chains/solana_service.dart';
import '../services/chains/polygon_service.dart';
import '../services/chains/bnb_service.dart';
import '../services/chains/prism_service.dart';
import '../services/chains/avalanche_service.dart';
import '../services/chains/arbitrum_service.dart';
import '../services/chains/optimism_service.dart';
import '../services/chains/base_chain_service.dart';
import '../services/chains/fantom_service.dart';
import '../services/chains/cronos_service.dart';
import '../services/chains/tron_service.dart';
import '../services/chains/dogecoin_service.dart';
import '../services/chains/litecoin_service.dart';
import '../services/chains/cardano_service.dart';
import '../services/chains/xrp_service.dart';
import '../services/chains/cosmos_service.dart';
import '../services/chains/polkadot_service.dart';
import '../services/chains/near_service.dart';
import '../services/chains/sui_service.dart';
import '../services/chains/aptos_service.dart';
import '../services/chains/stellar_service.dart';
import '../services/chains/algorand_service.dart';
import '../services/chains/hedera_service.dart';
import '../services/chains/ton_service.dart';
import '../services/chains/kaspa_service.dart';
import '../services/chains/filecoin_service.dart';
import '../services/chains/celestia_service.dart';
import '../services/chains/sei_service.dart';
import '../services/chains/bitcoin_cash_service.dart';
import '../services/chains/monero_service.dart';
import '../services/rpc_service.dart';
import '../services/price_service.dart';

/// Manages multi-chain wallet state across all supported blockchains.
///
/// Tracks balances, active chains, and total portfolio value for a
/// Trust Wallet / Exodus-style universal wallet experience.
class MultiChainProvider extends ChangeNotifier {
  final RpcService _rpcService;
  final PriceService _priceService = PriceService();

  // Chain service instances
  late final PrismService _prismService;
  late final SolanaService _solanaService;
  late final BitcoinService _bitcoinService;
  late final EthereumService _ethereumService;
  late final PolygonService _polygonService;
  late final BnbService _bnbService;
  late final AvalancheService _avalancheService;
  late final ArbitrumService _arbitrumService;
  late final OptimismService _optimismService;
  late final BaseChainService _baseService;
  late final FantomService _fantomService;
  late final CronosService _cronosService;
  late final TronService _tronService;
  late final DogecoinService _dogecoinService;
  late final LitecoinService _litecoinService;
  late final CardanoService _cardanoService;
  late final XrpService _xrpService;
  late final CosmosService _cosmosService;
  late final PolkadotService _polkadotService;
  late final NearService _nearService;
  late final SuiService _suiService;
  late final AptosService _aptosService;
  late final StellarService _stellarService;
  late final AlgorandService _algorandService;
  late final HederaService _hederaService;
  late final TonService _tonService;
  late final KaspaService _kaspaService;
  late final FilecoinService _filecoinService;
  late final CelestiaService _celestiaService;
  late final SeiService _seiService;
  late final BitcoinCashService _bitcoinCashService;
  late final MoneroService _moneroService;

  // State
  Map<ChainType, Chain> _chains = {};
  Map<ChainType, ChainService> _services = {};
  bool _isLoading = false;
  bool _isRefreshing = false;
  String? _error;
  Timer? _refreshTimer;

  // Mock USD prices for portfolio calculation
  final Map<String, double> _prices = {
    'BTC': 65000.0,
    'ETH': 3200.0,
    'SOL': 140.0,
    'POL': 0.55,
    'BNB': 580.0,
    'AVAX': 35.0,
    'FTM': 0.40,
    'CRO': 0.09,
    'TRX': 0.12,
    'DOGE': 0.15,
    'LTC': 85.0,
    'ADA': 0.45,
    'XRP': 0.55,
    'ARB': 1.10,
    'OP': 2.50,
    'USDT': 1.0,
    'USDC': 1.0,
    'DAI': 1.0,
    'ATOM': 9.50,
    'DOT': 7.20,
    'NEAR': 5.80,
    'SUI': 3.40,
    'APT': 9.00,
    'XLM': 0.12,
    'ALGO': 0.22,
    'HBAR': 0.08,
    'TON': 5.50,
    'KAS': 0.14,
    'FIL': 5.60,
    'TIA': 11.00,
    'SEI': 0.45,
    'BCH': 240.0,
    'XMR': 165.0,
    // ERC-20 tokens
    'SHIB': 0.000025,
    'PEPE': 0.000012,
    'AAVE': 95.0,
    'LDO': 2.20,
    'MKR': 1500.0,
    'CRV': 0.55,
    'SNX': 3.00,
    'GRT': 0.25,
    'FET': 2.30,
    'RNDR': 8.50,
    'APE': 1.50,
    'SAND': 0.45,
    'MANA': 0.42,
    'ENS': 18.0,
    '1INCH': 0.45,
    'COMP': 55.0,
    'WBTC': 65000.0,
    'STETH': 3200.0,
    'WETH': 3200.0,
    'UNI': 7.50,
    'LINK': 15.0,
    // SPL tokens (Solana)
    'JUP': 0.85,
    'RAY': 1.80,
    'BONK': 0.000025,
    'WIF': 2.40,
    'RENDER': 8.50,
    'PYTH': 0.38,
    'JTO': 3.20,
    'MSOL': 155.0,
    'ORCA': 4.50,
    'W': 0.60,
    'TNSR': 0.95,
    'HNT': 6.50,
    'MOBILE': 0.001,
    'BSOL': 150.0,
    'INF': 155.0,
    'POPCAT': 0.70,
    'MNGO': 0.03,
    'SRM': 0.04,
    'BOME': 0.01,
    'FIDA': 0.30,
    'WSOL': 140.0,
    // BNB tokens
    'CAKE': 2.50,
    'WBNB': 580.0,
    'XVS': 8.0,
    'TWT': 1.20,
    'ALPACA': 0.20,
    'SFM': 0.0003,
    'BTCB': 65000.0,
    // Polygon tokens
    'QUICK': 0.05,
    'SUSHI': 1.10,
    'BAL': 3.50,
    'BUSD': 1.0,
  };

  MultiChainProvider(this._rpcService) {
    _initServices();
    _initChains();
  }

  // -- Getters --

  Map<ChainType, Chain> get chains => Map.unmodifiable(_chains);
  List<Chain> get enabledChains =>
      _chains.values.where((c) => c.isEnabled).toList();
  List<Chain> get allChains => _chains.values.toList();
  bool get isLoading => _isLoading;
  bool get isRefreshing => _isRefreshing;
  String? get error => _error;

  ChainService getService(ChainType type) => _services[type]!;
  Chain getChain(ChainType type) => _chains[type]!;

  /// Total portfolio value in USD across all enabled chains.
  double get totalPortfolioUsd {
    return _chains.values
        .where((c) => c.isEnabled)
        .fold(0.0, (sum, chain) => sum + chain.totalUsdValue);
  }

  /// Get the native balance on a specific chain.
  double getChainBalance(ChainType type) {
    return _chains[type]?.balance ?? 0.0;
  }

  /// Get the USD value on a specific chain (native + tokens).
  double getChainUsdValue(ChainType type) {
    return _chains[type]?.totalUsdValue ?? 0.0;
  }

  // -- Initialization --

  void _initServices() {
    _prismService = PrismService(_rpcService);
    _solanaService = SolanaService();
    _bitcoinService = BitcoinService();
    _ethereumService = EthereumService();
    _polygonService = PolygonService();
    _bnbService = BnbService();
    _avalancheService = AvalancheService();
    _arbitrumService = ArbitrumService();
    _optimismService = OptimismService();
    _baseService = BaseChainService();
    _fantomService = FantomService();
    _cronosService = CronosService();
    _tronService = TronService();
    _dogecoinService = DogecoinService();
    _litecoinService = LitecoinService();
    _cardanoService = CardanoService();
    _xrpService = XrpService();
    _cosmosService = CosmosService();
    _polkadotService = PolkadotService();
    _nearService = NearService();
    _suiService = SuiService();
    _aptosService = AptosService();
    _stellarService = StellarService();
    _algorandService = AlgorandService();
    _hederaService = HederaService();
    _tonService = TonService();
    _kaspaService = KaspaService();
    _filecoinService = FilecoinService();
    _celestiaService = CelestiaService();
    _seiService = SeiService();
    _bitcoinCashService = BitcoinCashService();
    _moneroService = MoneroService();

    _services = {
      ChainType.prism: _prismService,
      ChainType.solana: _solanaService,
      ChainType.bitcoin: _bitcoinService,
      ChainType.ethereum: _ethereumService,
      ChainType.polygon: _polygonService,
      ChainType.bnb: _bnbService,
      ChainType.avalanche: _avalancheService,
      ChainType.arbitrum: _arbitrumService,
      ChainType.optimism: _optimismService,
      ChainType.base: _baseService,
      ChainType.fantom: _fantomService,
      ChainType.cronos: _cronosService,
      ChainType.tron: _tronService,
      ChainType.dogecoin: _dogecoinService,
      ChainType.litecoin: _litecoinService,
      ChainType.cardano: _cardanoService,
      ChainType.xrp: _xrpService,
      ChainType.cosmos: _cosmosService,
      ChainType.polkadot: _polkadotService,
      ChainType.near: _nearService,
      ChainType.sui: _suiService,
      ChainType.aptos: _aptosService,
      ChainType.stellar: _stellarService,
      ChainType.algorand: _algorandService,
      ChainType.hedera: _hederaService,
      ChainType.ton: _tonService,
      ChainType.kaspa: _kaspaService,
      ChainType.filecoin: _filecoinService,
      ChainType.celestia: _celestiaService,
      ChainType.sei: _seiService,
      ChainType.bitcoinCash: _bitcoinCashService,
      ChainType.monero: _moneroService,
    };
  }

  void _initChains() {
    _chains = Map.fromEntries(
      Chain.defaults.entries.map(
        (e) => MapEntry(e.key, e.value),
      ),
    );
  }

  /// Initialize the provider: load saved preferences and set addresses.
  Future<void> init(String walletPublicKey, {List<int>? seed}) async {
    _isLoading = true;
    notifyListeners();

    try {
      // Load enabled chain preferences
      await _loadPreferences();

      // Generate addresses for each chain
      _generateAddresses(walletPublicKey, seed ?? []);

      // Initial balance refresh
      await refreshAllBalances();

      _startAutoRefresh();
    } catch (e) {
      _error = e.toString();
    }

    _isLoading = false;
    notifyListeners();
  }

  /// Generate chain-specific addresses from the wallet seed.
  void _generateAddresses(String prismAddress, List<int> seed) {
    for (final type in ChainType.values) {
      String address;
      if (type == ChainType.prism) {
        // Prism uses the existing wallet address
        address = prismAddress;
      } else {
        address = _services[type]!.generateAddress(
          seed.isNotEmpty ? seed : prismAddress.codeUnits,
        );
      }

      _chains[type] = _chains[type]!.copyWith(address: address);
    }
  }

  // -- Balance Refresh --

  /// Refresh balances across all enabled chains.
  Future<void> refreshAllBalances() async {
    _isRefreshing = true;
    _error = null;
    notifyListeners();

    // Fetch live prices in parallel with chain balances
    final futures = <Future>[
      _priceService.fetchAllPrices(),
    ];
    for (final type in ChainType.values) {
      if (_chains[type]?.isEnabled ?? false) {
        futures.add(_refreshChainBalance(type));
      }
    }

    try {
      await Future.wait(futures);
    } catch (e) {
      _error = 'Some chain balances could not be refreshed';
    }

    _isRefreshing = false;
    notifyListeners();
  }

  /// Refresh the balance for a single chain.
  Future<void> _refreshChainBalance(ChainType type) async {
    final chain = _chains[type];
    if (chain == null || chain.address.isEmpty) return;

    try {
      final service = _services[type]!;
      final balance = await service.getBalance(chain.address);
      final price = _getPriceForSymbol(chain.symbol);
      final usdValue = balance * price;

      // Fetch tokens for EVM chains
      List<ChainToken> tokens = [];
      if (type == ChainType.ethereum) {
        tokens = await _fetchEthereumTokens(chain.address);
      } else if (type == ChainType.polygon) {
        tokens = await _fetchPolygonTokens(chain.address);
      } else if (type == ChainType.bnb) {
        tokens = await _fetchBnbTokens(chain.address);
      } else if (type == ChainType.avalanche) {
        tokens = await _fetchAvalancheTokens(chain.address);
      } else if (type == ChainType.arbitrum) {
        tokens = await _fetchArbitrumTokens(chain.address);
      } else if (type == ChainType.optimism) {
        tokens = await _fetchOptimismTokens(chain.address);
      } else if (type == ChainType.base) {
        tokens = await _fetchBaseTokens(chain.address);
      } else if (type == ChainType.fantom) {
        tokens = await _fetchFantomTokens(chain.address);
      } else if (type == ChainType.cronos) {
        tokens = await _fetchCronosTokens(chain.address);
      }
      // TRON TRC-20 tokens could be fetched here in the future
      // Dogecoin, Litecoin, Cardano, XRP have no token support

      _chains[type] = chain.copyWith(
        balance: balance,
        usdValue: usdValue,
        tokens: tokens,
      );
    } catch (e) {
      // Keep existing balance on error; don't overwrite with zero
      debugPrint('Failed to refresh ${chain.name}: $e');
    }
  }

  Future<List<ChainToken>> _fetchEthereumTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in ERC20Token.popular) {
      try {
        final balance = await _ethereumService.getERC20Balance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchPolygonTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in PolygonToken.popular) {
      try {
        final balance = await _polygonService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchBnbTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in BEP20Token.popular) {
      try {
        final balance = await _bnbService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchAvalancheTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in AvalancheToken.popular) {
      try {
        final balance = await _avalancheService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchArbitrumTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in ArbitrumToken.popular) {
      try {
        final balance = await _arbitrumService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchOptimismTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in OptimismToken.popular) {
      try {
        final balance = await _optimismService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchBaseTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in BaseToken.popular) {
      try {
        final balance = await _baseService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchFantomTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in FantomToken.popular) {
      try {
        final balance = await _fantomService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  Future<List<ChainToken>> _fetchCronosTokens(String address) async {
    final tokens = <ChainToken>[];
    for (final token in CronosToken.popular) {
      try {
        final balance = await _cronosService.getTokenBalance(
          token.contractAddress,
          address,
          tokenDecimals: token.decimals,
        );
        if (balance > 0) {
          final price = _getPriceForSymbol(token.symbol);
          tokens.add(ChainToken(
            name: token.name,
            symbol: token.symbol,
            contractAddress: token.contractAddress,
            balance: balance,
            usdValue: balance * price,
            decimals: token.decimals,
          ));
        }
      } catch (_) {}
    }
    return tokens;
  }

  double _getPriceForSymbol(String symbol) {
    // Try live price first, fall back to static defaults
    final livePrice = _priceService.getPrice(symbol);
    if (livePrice > 0) return livePrice;
    return _prices[symbol.toUpperCase()] ?? 0.0;
  }

  /// Update the mock price for a token.
  void updatePrice(String symbol, double price) {
    _prices[symbol.toUpperCase()] = price;
  }

  // -- Chain Enable/Disable --

  /// Toggle a chain on or off.
  Future<void> toggleChain(ChainType type, bool enabled) async {
    _chains[type] = _chains[type]!.copyWith(isEnabled: enabled);
    await _savePreferences();
    notifyListeners();

    if (enabled && _chains[type]!.balance == 0) {
      await _refreshChainBalance(type);
      notifyListeners();
    }
  }

  /// Enable a chain.
  Future<void> enableChain(ChainType type) async {
    await toggleChain(type, true);
  }

  /// Disable a chain.
  Future<void> disableChain(ChainType type) async {
    await toggleChain(type, false);
  }

  // -- Validation --

  /// Validate an address for a specific chain.
  bool validateAddress(ChainType type, String address) {
    return _services[type]!.validateAddress(address);
  }

  /// Estimate the fee for a transaction on a specific chain.
  Future<double> estimateFee(ChainType type) async {
    return await _services[type]!.estimateFee();
  }

  /// Get transaction history for a chain.
  Future<List<Map<String, dynamic>>> getTransactionHistory(ChainType type) async {
    final chain = _chains[type];
    if (chain == null || chain.address.isEmpty) return [];
    return await _services[type]!.getTransactionHistory(chain.address);
  }

  // -- Persistence --

  Future<void> _loadPreferences() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      for (final type in ChainType.values) {
        final key = 'chain_enabled_${type.name}';
        final enabled = prefs.getBool(key);
        if (enabled != null) {
          _chains[type] = _chains[type]!.copyWith(isEnabled: enabled);
        }
      }
    } catch (_) {}
  }

  Future<void> _savePreferences() async {
    try {
      final prefs = await SharedPreferences.getInstance();
      for (final type in ChainType.values) {
        final key = 'chain_enabled_${type.name}';
        await prefs.setBool(key, _chains[type]!.isEnabled);
      }
    } catch (_) {}
  }

  // -- Auto Refresh --

  void _startAutoRefresh() {
    _stopAutoRefresh();
    _refreshTimer = Timer.periodic(
      const Duration(seconds: 60),
      (_) => refreshAllBalances(),
    );
  }

  void _stopAutoRefresh() {
    _refreshTimer?.cancel();
    _refreshTimer = null;
  }

  @override
  void dispose() {
    _stopAutoRefresh();
    super.dispose();
  }
}
