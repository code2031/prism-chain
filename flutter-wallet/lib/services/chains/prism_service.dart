import '../../utils/constants.dart';
import '../rpc_service.dart';
import 'chain_service.dart';

/// Prism chain service -- wraps the existing RpcService to conform to
/// the ChainService interface.
///
/// This is the default chain and delegates all operations to the original
/// Prism RPC infrastructure.
class PrismService extends ChainService {
  final RpcService _rpcService;

  PrismService(this._rpcService);

  @override
  String get chainName => 'Prism';

  @override
  String get chainSymbol => 'SOL';

  @override
  String get chainIcon => '\u{1F7E3}'; // purple circle

  @override
  int get decimals => 9;

  @override
  String get explorerUrl => 'https://explorer.solana.com';

  @override
  String get rpcUrl => _rpcService.rpcUrl;

  @override
  Future<double> getBalance(String address) async {
    try {
      final lamports = await _rpcService.getBalance(address);
      return lamports / AppConstants.lamportsPerSol;
    } catch (e) {
      throw Exception('Failed to fetch Prism balance: $e');
    }
  }

  @override
  Future<String> sendTransaction(String to, double amount) async {
    // Delegates to the existing Prism RPC service.
    // The actual transaction building/signing is handled by WalletProvider.
    throw UnimplementedError(
      'Use WalletProvider.sendSol() for Prism transactions. '
      'Amount: $amount SOL to $to.',
    );
  }

  @override
  Future<List<Map<String, dynamic>>> getTransactionHistory(String address) async {
    try {
      final signatures = await _rpcService.getSignaturesForAddress(
        address,
        limit: 20,
      );

      return signatures.map((sig) {
        final isError = sig['err'] != null;
        final blockTime = sig['blockTime'] as int?;

        return {
          'txid': sig['signature'] ?? '',
          'type': isError ? 'failed' : 'unknown',
          'amount': 0.0,
          'confirmed': !isError,
          'timestamp': blockTime != null
              ? DateTime.fromMillisecondsSinceEpoch(blockTime * 1000).toIso8601String()
              : DateTime.now().toIso8601String(),
          'slot': sig['slot'],
          'memo': sig['memo'],
          'chain': 'prism',
        };
      }).toList();
    } catch (e) {
      return [];
    }
  }

  @override
  String generateAddress(List<int> seed) {
    // Prism uses the same derivation as Solana (m/44'/501'/0'/0').
    // The actual address generation is handled by CryptoUtils.
    const base58Chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    final buffer = StringBuffer();
    for (int i = 0; i < 44 && i < seed.length; i++) {
      buffer.write(base58Chars[seed[i] % base58Chars.length]);
    }
    return buffer.toString();
  }

  @override
  bool validateAddress(String address) {
    // Same as Solana: Base58, 32-44 characters
    final regex = RegExp(r'^[1-9A-HJ-NP-Za-km-z]{32,44}$');
    return regex.hasMatch(address);
  }

  /// Request an airdrop (devnet/testnet/localnet).
  Future<String> requestAirdrop(String address, {int lamports = 1000000000}) async {
    return await _rpcService.requestAirdrop(address, lamports);
  }

  /// Confirm a transaction.
  Future<bool> confirmTransaction(String signature) async {
    return await _rpcService.confirmTransaction(signature);
  }

  @override
  Future<double> estimateFee() async {
    // Prism base fee: 5000 lamports
    return 0.000005;
  }
}
