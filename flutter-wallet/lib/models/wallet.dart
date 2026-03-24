import 'dart:typed_data';

class WalletModel {
  final String publicKey;
  final String? mnemonic;
  final Uint8List? privateKey;
  final double balanceSol;
  final double balanceUsd;
  final List<String> tokens;
  final DateTime createdAt;

  WalletModel({
    required this.publicKey,
    this.mnemonic,
    this.privateKey,
    this.balanceSol = 0,
    this.balanceUsd = 0,
    this.tokens = const [],
    DateTime? createdAt,
  }) : createdAt = createdAt ?? DateTime.now();

  WalletModel copyWith({
    String? publicKey,
    String? mnemonic,
    Uint8List? privateKey,
    double? balanceSol,
    double? balanceUsd,
    List<String>? tokens,
    DateTime? createdAt,
  }) {
    return WalletModel(
      publicKey: publicKey ?? this.publicKey,
      mnemonic: mnemonic ?? this.mnemonic,
      privateKey: privateKey ?? this.privateKey,
      balanceSol: balanceSol ?? this.balanceSol,
      balanceUsd: balanceUsd ?? this.balanceUsd,
      tokens: tokens ?? this.tokens,
      createdAt: createdAt ?? this.createdAt,
    );
  }

  /// Get a truncated display version of the public key.
  String get shortAddress {
    if (publicKey.length <= 11) return publicKey;
    return '${publicKey.substring(0, 4)}...${publicKey.substring(publicKey.length - 4)}';
  }
}
