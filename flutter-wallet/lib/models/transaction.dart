enum TransactionType { send, receive, swap, stake, unstake, unknown }

enum TransactionStatus { confirmed, pending, failed }

class TransactionModel {
  final String signature;
  final TransactionType type;
  final TransactionStatus status;
  final String fromAddress;
  final String toAddress;
  final double amount;
  final String tokenSymbol;
  final double? fee;
  final DateTime timestamp;
  final int? slot;
  final String? memo;

  TransactionModel({
    required this.signature,
    required this.type,
    required this.status,
    required this.fromAddress,
    required this.toAddress,
    required this.amount,
    this.tokenSymbol = 'SOL',
    this.fee,
    required this.timestamp,
    this.slot,
    this.memo,
  });

  TransactionModel copyWith({
    String? signature,
    TransactionType? type,
    TransactionStatus? status,
    String? fromAddress,
    String? toAddress,
    double? amount,
    String? tokenSymbol,
    double? fee,
    DateTime? timestamp,
    int? slot,
    String? memo,
  }) {
    return TransactionModel(
      signature: signature ?? this.signature,
      type: type ?? this.type,
      status: status ?? this.status,
      fromAddress: fromAddress ?? this.fromAddress,
      toAddress: toAddress ?? this.toAddress,
      amount: amount ?? this.amount,
      tokenSymbol: tokenSymbol ?? this.tokenSymbol,
      fee: fee ?? this.fee,
      timestamp: timestamp ?? this.timestamp,
      slot: slot ?? this.slot,
      memo: memo ?? this.memo,
    );
  }

  /// Human-readable type label.
  String get typeLabel {
    switch (type) {
      case TransactionType.send:
        return 'Sent';
      case TransactionType.receive:
        return 'Received';
      case TransactionType.swap:
        return 'Swapped';
      case TransactionType.stake:
        return 'Staked';
      case TransactionType.unstake:
        return 'Unstaked';
      case TransactionType.unknown:
        return 'Transaction';
    }
  }

  /// Whether this is an outgoing transaction.
  bool get isOutgoing => type == TransactionType.send || type == TransactionType.stake;
}
