import 'dart:math';

/// Mock price service for token prices.
/// In production, this would connect to CoinGecko, Jupiter, etc.
class PriceService {
  final Random _random = Random();
  double _solPrice = 148.50;
  final Map<String, double> _tokenPrices = {
    'SOL': 148.50,
    'USDC': 1.00,
    'USDT': 1.00,
    'RAY': 2.85,
    'SRM': 0.042,
    'BONK': 0.0000234,
    'JTO': 3.15,
    'JUP': 1.42,
    'PYTH': 0.48,
    'ORCA': 0.72,
  };

  final Map<String, double> _priceChanges = {
    'SOL': 3.24,
    'USDC': 0.01,
    'USDT': -0.02,
    'RAY': -1.45,
    'SRM': 5.67,
    'BONK': 12.34,
    'JTO': -2.10,
    'JUP': 4.56,
    'PYTH': -0.89,
    'ORCA': 1.23,
  };

  /// Get the current SOL price in USD.
  Future<double> getSolPrice() async {
    // Simulate slight price fluctuation
    _solPrice += (_random.nextDouble() - 0.5) * 0.5;
    _tokenPrices['SOL'] = _solPrice;
    return _solPrice;
  }

  /// Get the price of a token by symbol.
  Future<double> getTokenPrice(String symbol) async {
    return _tokenPrices[symbol] ?? 0.0;
  }

  /// Get the 24h price change percentage.
  Future<double> getPriceChange24h(String symbol) async {
    return _priceChanges[symbol] ?? 0.0;
  }

  /// Get all known token prices.
  Future<Map<String, double>> getAllPrices() async {
    await getSolPrice(); // Refresh SOL price
    return Map.from(_tokenPrices);
  }

  /// Get price chart data points (mock 7-day history).
  Future<List<PricePoint>> getPriceHistory(String symbol, {int days = 7}) async {
    final currentPrice = _tokenPrices[symbol] ?? 100.0;
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

    // Make the last point match the current price
    if (points.isNotEmpty) {
      points[points.length - 1] = PricePoint(
        timestamp: now,
        price: currentPrice,
      );
    }

    return points;
  }

  /// Get portfolio total value history (mock).
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
      points[points.length - 1] = PricePoint(
        timestamp: now,
        price: currentValue,
      );
    }

    return points;
  }
}

class PricePoint {
  final DateTime timestamp;
  final double price;

  PricePoint({required this.timestamp, required this.price});
}
