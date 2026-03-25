import 'dart:convert';
import 'dart:math';
import 'package:http/http.dart' as http;

/// Live cryptocurrency price service using CoinGecko API (free, no key required).
///
/// Fetches real-time USD prices for all supported chains and tokens.
/// Falls back to cached/default prices on network failure.
class PriceService {
  final http.Client _client;
  final String _baseUrl;
  final Random _random = Random();

  /// CoinGecko IDs mapped to our ticker symbols.
  static const Map<String, String> _coinGeckoIds = {
    // Layer 1 chains
    'BTC': 'bitcoin',
    'ETH': 'ethereum',
    'SOL': 'solana',
    'BNB': 'binancecoin',
    'XRP': 'ripple',
    'ADA': 'cardano',
    'DOGE': 'dogecoin',
    'AVAX': 'avalanche-2',
    'DOT': 'polkadot',
    'ATOM': 'cosmos',
    'LTC': 'litecoin',
    'NEAR': 'near',
    'BCH': 'bitcoin-cash',
    'XMR': 'monero',
    'FIL': 'filecoin',
    'HBAR': 'hedera-hashgraph',
    'ALGO': 'algorand',
    'XLM': 'stellar',
    'SUI': 'sui',
    'APT': 'aptos',
    'TON': 'the-open-network',
    'KAS': 'kaspa',
    'TIA': 'celestia',
    'SEI': 'sei-network',
    'TRX': 'tron',
    'FTM': 'fantom',
    'CRO': 'crypto-com-chain',
    'POL': 'matic-network',
    // L2s
    'ARB': 'arbitrum',
    'OP': 'optimism',
    // ERC-20 / DeFi tokens
    'LINK': 'chainlink',
    'UNI': 'uniswap',
    'AAVE': 'aave',
    'MKR': 'maker',
    'COMP': 'compound-governance-token',
    'SNX': 'havven',
    'CRV': 'curve-dao-token',
    'LDO': 'lido-dao',
    'GRT': 'the-graph',
    'ENS': 'ethereum-name-service',
    '1INCH': '1inch',
    'SUSHI': 'sushi',
    'BAL': 'balancer',
    'RNDR': 'render-token',
    'FET': 'fetch-ai',
    'APE': 'apecoin',
    'SAND': 'the-sandbox',
    'MANA': 'decentraland',
    'SHIB': 'shiba-inu',
    'PEPE': 'pepe',
    'WBTC': 'wrapped-bitcoin',
    'STETH': 'staked-ether',
    // SPL tokens (Solana)
    'JUP': 'jupiter-exchange-solana',
    'RAY': 'raydium',
    'BONK': 'bonk',
    'WIF': 'dogwifcoin',
    'PYTH': 'pyth-network',
    'JTO': 'jito-governance-token',
    'ORCA': 'orca',
    'HNT': 'helium',
    'W': 'wormhole',
    'TNSR': 'tensor',
    'POPCAT': 'popcat',
    'MNGO': 'mango-markets',
    'BOME': 'book-of-meme',
    'FIDA': 'bonfida',
    // BNB tokens
    'CAKE': 'pancakeswap-token',
    'TWT': 'trust-wallet-token',
    'XVS': 'venus',
    // Polygon tokens
    'QUICK': 'quickswap',
    // Stablecoins
    'USDT': 'tether',
    'USDC': 'usd-coin',
    'DAI': 'dai',
    'BUSD': 'binance-usd',
  };

  /// Cached prices (symbol → USD).
  final Map<String, double> _prices = {};

  /// Cached 24h price changes (symbol → percentage).
  final Map<String, double> _priceChanges = {};

  /// Last fetch timestamp.
  DateTime? _lastFetch;

  /// Cache duration (refresh every 60 seconds).
  static const _cacheDuration = Duration(seconds: 60);

  PriceService({http.Client? client, String? baseUrl})
      : _client = client ?? http.Client(),
        _baseUrl = baseUrl ?? 'https://api.coingecko.com/api/v3' {
    _loadDefaults();
  }

  /// Get the current USD price for a symbol.
  double getPrice(String symbol) {
    return _prices[symbol.toUpperCase()] ?? 0.0;
  }

  /// Get the current SOL price in USD.
  Future<double> getSolPrice() async {
    if (_shouldRefresh()) {
      await fetchAllPrices();
    }
    return _prices['SOL'] ?? 140.0;
  }

  /// Get the price of a token by symbol.
  Future<double> getTokenPrice(String symbol) async {
    if (_shouldRefresh()) {
      await fetchAllPrices();
    }
    return _prices[symbol.toUpperCase()] ?? 0.0;
  }

  /// Get the 24h price change percentage.
  Future<double> getPriceChange24h(String symbol) async {
    return _priceChanges[symbol.toUpperCase()] ?? 0.0;
  }

  /// Get all known token prices.
  Future<Map<String, double>> getAllPrices() async {
    if (_shouldRefresh()) {
      await fetchAllPrices();
    }
    return Map.from(_prices);
  }

  /// Whether prices have been fetched at least once from the API.
  bool get hasPrices => _lastFetch != null;

  bool _shouldRefresh() {
    return _lastFetch == null ||
        DateTime.now().difference(_lastFetch!) > _cacheDuration;
  }

  /// Fetch all prices from CoinGecko.
  Future<Map<String, double>> fetchAllPrices() async {
    if (!_shouldRefresh() && _prices.isNotEmpty) {
      return Map.from(_prices);
    }

    final ids = _coinGeckoIds.values.toSet().toList();

    try {
      final idsParam = ids.join(',');
      final url =
          '$_baseUrl/simple/price?ids=$idsParam&vs_currencies=usd&include_24hr_change=true';

      final response = await _client.get(
        Uri.parse(url),
        headers: {'Accept': 'application/json'},
      ).timeout(const Duration(seconds: 10));

      if (response.statusCode != 200) {
        throw Exception('CoinGecko API error: HTTP ${response.statusCode}');
      }

      final data = jsonDecode(response.body) as Map<String, dynamic>;

      // Reverse-map: CoinGecko ID → our symbol → price
      for (final entry in _coinGeckoIds.entries) {
        final symbol = entry.key;
        final geckoId = entry.value;
        final coinData = data[geckoId] as Map<String, dynamic>?;
        if (coinData != null) {
          if (coinData['usd'] != null) {
            _prices[symbol] = (coinData['usd'] as num).toDouble();
          }
          if (coinData['usd_24h_change'] != null) {
            _priceChanges[symbol] =
                (coinData['usd_24h_change'] as num).toDouble();
          }
        }
      }

      _lastFetch = DateTime.now();
    } catch (e) {
      // Keep existing prices on failure
    }

    return Map.from(_prices);
  }

  /// Load default fallback prices.
  void _loadDefaults() {
    _prices.addAll({
      'BTC': 65000.0, 'ETH': 3200.0, 'SOL': 140.0, 'BNB': 580.0,
      'XRP': 0.55, 'ADA': 0.45, 'DOGE': 0.15, 'AVAX': 35.0,
      'DOT': 7.20, 'LINK': 15.0, 'ATOM': 9.50, 'LTC': 85.0,
      'NEAR': 5.80, 'BCH': 240.0, 'XMR': 165.0, 'FIL': 5.60,
      'HBAR': 0.08, 'ALGO': 0.22, 'XLM': 0.12, 'SUI': 3.40,
      'APT': 9.00, 'TON': 5.50, 'KAS': 0.14, 'TIA': 11.0,
      'SEI': 0.45, 'TRX': 0.12, 'FTM': 0.40, 'CRO': 0.09,
      'POL': 0.55, 'ARB': 1.10, 'OP': 2.50,
      'UNI': 7.50, 'AAVE': 95.0, 'MKR': 1500.0, 'COMP': 55.0,
      'SNX': 3.0, 'CRV': 0.55, 'LDO': 2.20, 'GRT': 0.25,
      'ENS': 18.0, '1INCH': 0.45, 'SUSHI': 1.10, 'BAL': 3.50,
      'RNDR': 8.50, 'FET': 2.30, 'APE': 1.50, 'SAND': 0.45,
      'MANA': 0.42, 'SHIB': 0.000025, 'PEPE': 0.000012,
      'WBTC': 65000.0, 'STETH': 3200.0,
      'JUP': 0.85, 'RAY': 1.80, 'BONK': 0.000025, 'WIF': 2.40,
      'PYTH': 0.38, 'JTO': 3.20, 'ORCA': 4.50, 'HNT': 6.50,
      'W': 0.60, 'TNSR': 0.95, 'POPCAT': 0.70,
      'CAKE': 2.50, 'TWT': 1.20, 'XVS': 8.0, 'QUICK': 0.05,
      'USDT': 1.0, 'USDC': 1.0, 'DAI': 1.0, 'BUSD': 1.0,
    });
  }

  /// Get price chart data points (7-day history from CoinGecko).
  Future<List<PricePoint>> getPriceHistory(String symbol, {int days = 7}) async {
    final geckoId = _coinGeckoIds[symbol.toUpperCase()];
    if (geckoId != null) {
      try {
        final url = '$_baseUrl/coins/$geckoId/market_chart?vs_currency=usd&days=$days';
        final response = await _client.get(Uri.parse(url)).timeout(const Duration(seconds: 10));

        if (response.statusCode == 200) {
          final data = jsonDecode(response.body) as Map<String, dynamic>;
          final priceList = data['prices'] as List<dynamic>? ?? [];

          return priceList.map((point) {
            final p = point as List<dynamic>;
            return PricePoint(
              timestamp: DateTime.fromMillisecondsSinceEpoch((p[0] as num).toInt()),
              price: (p[1] as num).toDouble(),
            );
          }).toList();
        }
      } catch (_) {}
    }

    // Fallback: generate mock history
    return _mockPriceHistory(symbol, days: days);
  }

  /// Get portfolio total value history (mock — requires historical balances).
  Future<List<PricePoint>> getPortfolioHistory(double currentValue, {int days = 30}) async {
    final points = <PricePoint>[];
    final now = DateTime.now();

    double value = currentValue * (0.85 + _random.nextDouble() * 0.1);
    for (int i = days; i >= 0; i--) {
      value += ((_random.nextDouble() - 0.45) * currentValue * 0.01);
      value = value.clamp(currentValue * 0.7, currentValue * 1.1);
      points.add(PricePoint(
        timestamp: now.subtract(Duration(days: i)),
        price: value,
      ));
    }

    if (points.isNotEmpty) {
      points[points.length - 1] = PricePoint(timestamp: now, price: currentValue);
    }

    return points;
  }

  List<PricePoint> _mockPriceHistory(String symbol, {int days = 7}) {
    final currentPrice = _prices[symbol.toUpperCase()] ?? 100.0;
    final points = <PricePoint>[];
    final now = DateTime.now();

    double price = currentPrice * (0.9 + _random.nextDouble() * 0.1);
    for (int i = days * 24; i >= 0; i--) {
      price += ((_random.nextDouble() - 0.48) * currentPrice * 0.005);
      price = price.clamp(currentPrice * 0.8, currentPrice * 1.2);
      points.add(PricePoint(
        timestamp: now.subtract(Duration(hours: i)),
        price: price,
      ));
    }

    if (points.isNotEmpty) {
      points[points.length - 1] = PricePoint(timestamp: now, price: currentPrice);
    }

    return points;
  }

  void dispose() {
    _client.close();
  }
}

class PricePoint {
  final DateTime timestamp;
  final double price;

  PricePoint({required this.timestamp, required this.price});
}
