class TokenModel {
  final String mintAddress;
  final String symbol;
  final String name;
  final String? iconUrl;
  final double balance;
  final int decimals;
  final double priceUsd;
  final double priceChangePercent24h;
  final double valueUsd;

  TokenModel({
    required this.mintAddress,
    required this.symbol,
    required this.name,
    this.iconUrl,
    this.balance = 0,
    this.decimals = 9,
    this.priceUsd = 0,
    this.priceChangePercent24h = 0,
    this.valueUsd = 0,
  });

  TokenModel copyWith({
    String? mintAddress,
    String? symbol,
    String? name,
    String? iconUrl,
    double? balance,
    int? decimals,
    double? priceUsd,
    double? priceChangePercent24h,
    double? valueUsd,
  }) {
    return TokenModel(
      mintAddress: mintAddress ?? this.mintAddress,
      symbol: symbol ?? this.symbol,
      name: name ?? this.name,
      iconUrl: iconUrl ?? this.iconUrl,
      balance: balance ?? this.balance,
      decimals: decimals ?? this.decimals,
      priceUsd: priceUsd ?? this.priceUsd,
      priceChangePercent24h: priceChangePercent24h ?? this.priceChangePercent24h,
      valueUsd: valueUsd ?? this.valueUsd,
    );
  }

  /// SOL native token representation.
  static TokenModel sol({
    double balance = 0,
    double priceUsd = 0,
    double priceChangePercent24h = 0,
  }) {
    return TokenModel(
      mintAddress: 'So11111111111111111111111111111111111111112',
      symbol: 'SOL',
      name: 'Solana',
      balance: balance,
      decimals: 9,
      priceUsd: priceUsd,
      priceChangePercent24h: priceChangePercent24h,
      valueUsd: balance * priceUsd,
    );
  }

  /// Whether this is the native SOL token.
  bool get isNative => symbol == 'SOL';
}
