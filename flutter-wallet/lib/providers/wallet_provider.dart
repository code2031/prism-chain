import 'dart:async';
import 'package:flutter/foundation.dart';
import '../models/token.dart';
import '../models/transaction.dart';
import '../models/wallet.dart';
import '../services/price_service.dart';
import '../services/rpc_service.dart';
import '../services/wallet_service.dart';
import '../utils/constants.dart';
import '../utils/formatters.dart';

class WalletProvider extends ChangeNotifier {
  final WalletService _walletService;
  final RpcService _rpcService;
  final PriceService _priceService;

  WalletModel? _wallet;
  List<TokenModel> _tokens = [];
  List<TransactionModel> _transactions = [];
  bool _isLoading = false;
  bool _isRefreshing = false;
  String? _error;
  Timer? _refreshTimer;

  WalletProvider(this._walletService, this._rpcService, this._priceService);

  WalletModel? get wallet => _wallet;
  List<TokenModel> get tokens => _tokens;
  List<TransactionModel> get transactions => _transactions;
  bool get isLoading => _isLoading;
  bool get isRefreshing => _isRefreshing;
  String? get error => _error;
  bool get hasWallet => _wallet != null;

  double get totalBalanceUsd {
    return _tokens.fold(0.0, (sum, t) => sum + t.valueUsd);
  }

  double get solBalance => _wallet?.balanceSol ?? 0;

  /// Initialize the wallet provider.
  Future<bool> init() async {
    _isLoading = true;
    notifyListeners();

    try {
      _wallet = await _walletService.loadWallet();
      if (_wallet != null) {
        await refreshAll();
        _startAutoRefresh();
      }
      _isLoading = false;
      notifyListeners();
      return _wallet != null;
    } catch (e) {
      _error = e.toString();
      _isLoading = false;
      notifyListeners();
      return false;
    }
  }

  /// Create a new wallet.
  Future<String> createWallet() async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      final result = await _walletService.createWallet();
      _wallet = result.wallet;
      await _walletService.setOnboarded();
      await refreshAll();
      _startAutoRefresh();
      _isLoading = false;
      notifyListeners();
      return result.mnemonic;
    } catch (e) {
      _error = e.toString();
      _isLoading = false;
      notifyListeners();
      rethrow;
    }
  }

  /// Import wallet from mnemonic.
  Future<void> importFromMnemonic(String mnemonic) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _wallet = await _walletService.importFromMnemonic(mnemonic);
      await _walletService.setOnboarded();
      await refreshAll();
      _startAutoRefresh();
      _isLoading = false;
      notifyListeners();
    } catch (e) {
      _error = e.toString();
      _isLoading = false;
      notifyListeners();
      rethrow;
    }
  }

  /// Import wallet from private key.
  Future<void> importFromPrivateKey(String privateKey) async {
    _isLoading = true;
    _error = null;
    notifyListeners();

    try {
      _wallet = await _walletService.importFromPrivateKey(privateKey);
      await _walletService.setOnboarded();
      await refreshAll();
      _startAutoRefresh();
      _isLoading = false;
      notifyListeners();
    } catch (e) {
      _error = e.toString();
      _isLoading = false;
      notifyListeners();
      rethrow;
    }
  }

  /// Refresh all wallet data.
  Future<void> refreshAll() async {
    if (_wallet == null) return;
    _isRefreshing = true;
    notifyListeners();

    try {
      await Future.wait([
        _refreshBalance(),
        _refreshTokens(),
        _refreshTransactions(),
      ]);
      _error = null;
    } catch (e) {
      _error = e.toString();
    }

    _isRefreshing = false;
    notifyListeners();
  }

  /// Refresh SOL balance.
  Future<void> _refreshBalance() async {
    if (_wallet == null) return;
    try {
      final lamports = await _rpcService.getBalance(_wallet!.publicKey);
      final solBalance = lamports / AppConstants.lamportsPerSol;
      final solPrice = await _priceService.getSolPrice();
      final usdBalance = solBalance * solPrice;

      _walletService.updateBalance(solBalance, usdBalance);
      _wallet = _wallet!.copyWith(
        balanceSol: solBalance,
        balanceUsd: usdBalance,
      );
    } catch (_) {
      // Keep existing balance on error
    }
  }

  /// Refresh token list.
  Future<void> _refreshTokens() async {
    if (_wallet == null) return;
    try {
      final solPrice = await _priceService.getSolPrice();
      final solChange = await _priceService.getPriceChange24h('SOL');

      final solToken = TokenModel.sol(
        balance: _wallet!.balanceSol,
        priceUsd: solPrice,
        priceChangePercent24h: solChange,
      );

      final tokenAccounts = await _rpcService.getTokenAccountsByOwner(
        _wallet!.publicKey,
        AppConstants.tokenProgramId,
      );

      final splTokens = <TokenModel>[];
      for (final account in tokenAccounts) {
        try {
          final parsed = account['account']?['data']?['parsed']?['info'];
          if (parsed == null) continue;

          final mint = parsed['mint'] as String? ?? '';
          final tokenAmount = parsed['tokenAmount'];
          final balance = double.tryParse(tokenAmount?['uiAmountString'] ?? '0') ?? 0;
          final decimals = tokenAmount?['decimals'] as int? ?? 9;

          if (balance > 0) {
            splTokens.add(TokenModel(
              mintAddress: mint,
              symbol: mint.substring(0, 4).toUpperCase(),
              name: 'SPL Token',
              balance: balance,
              decimals: decimals,
            ));
          }
        } catch (_) {
          continue;
        }
      }

      _tokens = [solToken, ...splTokens];
    } catch (_) {
      // Keep SOL token at minimum
      final solPrice = await _priceService.getSolPrice();
      _tokens = [
        TokenModel.sol(
          balance: _wallet?.balanceSol ?? 0,
          priceUsd: solPrice,
        ),
      ];
    }
  }

  /// Refresh transaction history.
  Future<void> _refreshTransactions() async {
    if (_wallet == null) return;
    try {
      final signatures = await _rpcService.getSignaturesForAddress(
        _wallet!.publicKey,
        limit: 20,
      );

      _transactions = signatures.map((sig) {
        final isError = sig['err'] != null;
        final blockTime = sig['blockTime'] as int?;
        final timestamp = blockTime != null
            ? DateTime.fromMillisecondsSinceEpoch(blockTime * 1000)
            : DateTime.now();

        return TransactionModel(
          signature: sig['signature'] as String? ?? '',
          type: TransactionType.unknown,
          status: isError ? TransactionStatus.failed : TransactionStatus.confirmed,
          fromAddress: _wallet!.publicKey,
          toAddress: '',
          amount: 0,
          timestamp: timestamp,
          slot: sig['slot'] as int?,
          memo: sig['memo'] as String?,
        );
      }).toList();
    } catch (_) {
      // Keep existing transactions
    }
  }

  /// Send SOL to an address.
  Future<String> sendSol(String toAddress, double amount) async {
    if (_wallet == null) throw Exception('No wallet loaded');

    _isLoading = true;
    notifyListeners();

    try {
      // For a real implementation, we would construct, sign, and send
      // a proper Solana transaction. This is a simplified version.
      final lamports = Formatters.solToLamports(amount.toString());

      // Get recent blockhash
      await _rpcService.getLatestBlockhash();

      // In a real implementation, build and sign the transaction here
      // For now, throw an informative error
      throw Exception(
        'Transaction building requires full Solana SDK. '
        'Amount: $amount SOL ($lamports lamports) to $toAddress'
      );
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Request airdrop (devnet/testnet/localnet).
  Future<String> requestAirdrop({int lamports = 1000000000}) async {
    if (_wallet == null) throw Exception('No wallet loaded');

    try {
      final signature = await _rpcService.requestAirdrop(
        _wallet!.publicKey,
        lamports,
      );

      // Wait for confirmation
      await _rpcService.confirmTransaction(signature);
      await refreshAll();

      return signature;
    } catch (e) {
      throw Exception('Airdrop failed: $e');
    }
  }

  /// Delete wallet and reset state.
  Future<void> deleteWallet() async {
    _stopAutoRefresh();
    await _walletService.deleteWallet();
    _wallet = null;
    _tokens = [];
    _transactions = [];
    _error = null;
    notifyListeners();
  }

  void _startAutoRefresh() {
    _stopAutoRefresh();
    _refreshTimer = Timer.periodic(
      const Duration(seconds: 30),
      (_) => refreshAll(),
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
