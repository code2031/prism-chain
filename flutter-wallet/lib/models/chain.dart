/// Supported blockchain types.
enum ChainType {
  prism,
  solana,
  bitcoin,
  ethereum,
  polygon,
  bnb,
  avalanche,
  arbitrum,
  optimism,
  base,
  fantom,
  cronos,
  tron,
  dogecoin,
  litecoin,
  cardano,
  xrp,
}

/// Represents a blockchain network with its current state.
class Chain {
  final ChainType type;
  final String name;
  final String symbol;
  final String iconEmoji;
  final String color; // hex color for UI
  final bool isEnabled;
  final double balance;
  final double usdValue;
  final String address;
  final List<ChainToken> tokens;

  const Chain({
    required this.type,
    required this.name,
    required this.symbol,
    required this.iconEmoji,
    required this.color,
    this.isEnabled = true,
    this.balance = 0,
    this.usdValue = 0,
    this.address = '',
    this.tokens = const [],
  });

  Chain copyWith({
    ChainType? type,
    String? name,
    String? symbol,
    String? iconEmoji,
    String? color,
    bool? isEnabled,
    double? balance,
    double? usdValue,
    String? address,
    List<ChainToken>? tokens,
  }) {
    return Chain(
      type: type ?? this.type,
      name: name ?? this.name,
      symbol: symbol ?? this.symbol,
      iconEmoji: iconEmoji ?? this.iconEmoji,
      color: color ?? this.color,
      isEnabled: isEnabled ?? this.isEnabled,
      balance: balance ?? this.balance,
      usdValue: usdValue ?? this.usdValue,
      address: address ?? this.address,
      tokens: tokens ?? this.tokens,
    );
  }

  /// Total USD value including all tokens on this chain.
  double get totalUsdValue {
    final tokenValue = tokens.fold(0.0, (sum, t) => sum + t.usdValue);
    return usdValue + tokenValue;
  }

  /// Short display address.
  String get shortAddress {
    if (address.length <= 11) return address;
    return '${address.substring(0, 6)}...${address.substring(address.length - 4)}';
  }

  /// Default chain configurations.
  static const Map<ChainType, Chain> defaults = {
    ChainType.prism: Chain(
      type: ChainType.prism,
      name: 'Prism',
      symbol: 'SOL',
      iconEmoji: '\u{1F7E3}', // purple circle
      color: 'FF9945FF',
    ),
    ChainType.solana: Chain(
      type: ChainType.solana,
      name: 'Solana',
      symbol: 'SOL',
      iconEmoji: '\u{1F7E3}', // purple circle
      color: 'FF7B61FF',
    ),
    ChainType.bitcoin: Chain(
      type: ChainType.bitcoin,
      name: 'Bitcoin',
      symbol: 'BTC',
      iconEmoji: '\u{1F7E0}', // orange circle
      color: 'FFF7931A',
    ),
    ChainType.ethereum: Chain(
      type: ChainType.ethereum,
      name: 'Ethereum',
      symbol: 'ETH',
      iconEmoji: '\u{1F535}', // blue circle
      color: 'FF627EEA',
    ),
    ChainType.polygon: Chain(
      type: ChainType.polygon,
      name: 'Polygon',
      symbol: 'POL',
      iconEmoji: '\u{1F7E3}', // purple circle
      color: 'FF8247E5',
    ),
    ChainType.bnb: Chain(
      type: ChainType.bnb,
      name: 'BNB Chain',
      symbol: 'BNB',
      iconEmoji: '\u{1F7E1}', // yellow circle
      color: 'FFF3BA2F',
    ),
    ChainType.avalanche: Chain(
      type: ChainType.avalanche,
      name: 'Avalanche',
      symbol: 'AVAX',
      iconEmoji: '\u{1F53A}', // red triangle
      color: 'FFE84142',
      isEnabled: false,
    ),
    ChainType.arbitrum: Chain(
      type: ChainType.arbitrum,
      name: 'Arbitrum',
      symbol: 'ETH',
      iconEmoji: '\u{1F537}', // blue diamond
      color: 'FF28A0F0',
      isEnabled: false,
    ),
    ChainType.optimism: Chain(
      type: ChainType.optimism,
      name: 'Optimism',
      symbol: 'ETH',
      iconEmoji: '\u{1F534}', // red circle
      color: 'FFFF0420',
      isEnabled: false,
    ),
    ChainType.base: Chain(
      type: ChainType.base,
      name: 'Base',
      symbol: 'ETH',
      iconEmoji: '\u{1F535}', // blue circle
      color: 'FF0052FF',
      isEnabled: false,
    ),
    ChainType.fantom: Chain(
      type: ChainType.fantom,
      name: 'Fantom',
      symbol: 'FTM',
      iconEmoji: '\u{1F47B}', // ghost
      color: 'FF1969FF',
      isEnabled: false,
    ),
    ChainType.cronos: Chain(
      type: ChainType.cronos,
      name: 'Cronos',
      symbol: 'CRO',
      iconEmoji: '\u{1F48E}', // gem stone
      color: 'FF002D74',
      isEnabled: false,
    ),
    ChainType.tron: Chain(
      type: ChainType.tron,
      name: 'TRON',
      symbol: 'TRX',
      iconEmoji: '\u{26A1}', // lightning bolt
      color: 'FFEB0029',
      isEnabled: false,
    ),
    ChainType.dogecoin: Chain(
      type: ChainType.dogecoin,
      name: 'Dogecoin',
      symbol: 'DOGE',
      iconEmoji: '\u{1F415}', // dog
      color: 'FFC3A634',
      isEnabled: false,
    ),
    ChainType.litecoin: Chain(
      type: ChainType.litecoin,
      name: 'Litecoin',
      symbol: 'LTC',
      iconEmoji: '\u{1FA99}', // coin
      color: 'FFA6A9AA',
      isEnabled: false,
    ),
    ChainType.cardano: Chain(
      type: ChainType.cardano,
      name: 'Cardano',
      symbol: 'ADA',
      iconEmoji: '\u{1F0CF}', // joker card
      color: 'FF0033AD',
      isEnabled: false,
    ),
    ChainType.xrp: Chain(
      type: ChainType.xrp,
      name: 'XRP Ledger',
      symbol: 'XRP',
      iconEmoji: '\u{1F4A7}', // droplet
      color: 'FF23292F',
      isEnabled: false,
    ),
  };
}

/// Represents a token on a specific chain (ERC-20, SPL, BEP-20, etc.).
class ChainToken {
  final String name;
  final String symbol;
  final String contractAddress;
  final double balance;
  final double usdValue;
  final int decimals;
  final String? iconUrl;

  const ChainToken({
    required this.name,
    required this.symbol,
    required this.contractAddress,
    this.balance = 0,
    this.usdValue = 0,
    this.decimals = 18,
    this.iconUrl,
  });

  ChainToken copyWith({
    String? name,
    String? symbol,
    String? contractAddress,
    double? balance,
    double? usdValue,
    int? decimals,
    String? iconUrl,
  }) {
    return ChainToken(
      name: name ?? this.name,
      symbol: symbol ?? this.symbol,
      contractAddress: contractAddress ?? this.contractAddress,
      balance: balance ?? this.balance,
      usdValue: usdValue ?? this.usdValue,
      decimals: decimals ?? this.decimals,
      iconUrl: iconUrl ?? this.iconUrl,
    );
  }
}
