import 'dart:typed_data';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../models/wallet.dart';
import '../utils/constants.dart';
import '../utils/crypto_utils.dart';

/// Manages wallet creation, import, storage, and signing.
class WalletService {
  final FlutterSecureStorage _secureStorage;
  KeypairData? _currentKeypair;
  WalletModel? _currentWallet;

  WalletService({FlutterSecureStorage? secureStorage})
      : _secureStorage = secureStorage ?? const FlutterSecureStorage();

  WalletModel? get currentWallet => _currentWallet;
  KeypairData? get currentKeypair => _currentKeypair;

  /// Create a new wallet with a fresh mnemonic.
  Future<({WalletModel wallet, String mnemonic})> createWallet() async {
    final mnemonic = CryptoUtils.generateMnemonic();
    final keypair = await CryptoUtils.keypairFromMnemonic(mnemonic);

    final wallet = WalletModel(
      publicKey: keypair.publicKeyBase58,
      mnemonic: mnemonic,
      privateKey: keypair.privateKey,
    );

    await _saveWallet(mnemonic, keypair);
    _currentKeypair = keypair;
    _currentWallet = wallet;

    return (wallet: wallet, mnemonic: mnemonic);
  }

  /// Import a wallet from a mnemonic phrase.
  Future<WalletModel> importFromMnemonic(String mnemonic) async {
    final trimmed = mnemonic.trim().toLowerCase();
    if (!CryptoUtils.validateMnemonic(trimmed)) {
      throw WalletException('Invalid mnemonic phrase');
    }

    final keypair = await CryptoUtils.keypairFromMnemonic(trimmed);

    final wallet = WalletModel(
      publicKey: keypair.publicKeyBase58,
      mnemonic: trimmed,
      privateKey: keypair.privateKey,
    );

    await _saveWallet(trimmed, keypair);
    _currentKeypair = keypair;
    _currentWallet = wallet;

    return wallet;
  }

  /// Import a wallet from a private key (base58 encoded).
  Future<WalletModel> importFromPrivateKey(String privateKeyBase58) async {
    try {
      final privateKeyBytes = CryptoUtils.fromBase58(privateKeyBase58.trim());
      final keypair = CryptoUtils.keypairFromPrivateKey(privateKeyBytes);

      final wallet = WalletModel(
        publicKey: keypair.publicKeyBase58,
        privateKey: keypair.privateKey,
      );

      await _saveWalletFromKey(keypair);
      _currentKeypair = keypair;
      _currentWallet = wallet;

      return wallet;
    } catch (e) {
      throw WalletException('Invalid private key: $e');
    }
  }

  /// Load a previously saved wallet.
  Future<WalletModel?> loadWallet() async {
    try {
      final publicKey = await _secureStorage.read(key: AppConstants.walletPublicKeyKey);
      if (publicKey == null) return null;

      final mnemonic = await _secureStorage.read(key: AppConstants.walletMnemonicKey);
      final privateKeyBase58 = await _secureStorage.read(key: AppConstants.walletPrivateKeyKey);

      KeypairData? keypair;
      if (mnemonic != null) {
        keypair = await CryptoUtils.keypairFromMnemonic(mnemonic);
      } else if (privateKeyBase58 != null) {
        final privateKeyBytes = CryptoUtils.fromBase58(privateKeyBase58);
        keypair = CryptoUtils.keypairFromPrivateKey(privateKeyBytes);
      }

      final wallet = WalletModel(
        publicKey: publicKey,
        mnemonic: mnemonic,
        privateKey: keypair?.privateKey,
      );

      _currentKeypair = keypair;
      _currentWallet = wallet;

      return wallet;
    } catch (e) {
      return null;
    }
  }

  /// Check if a wallet exists in secure storage.
  Future<bool> hasWallet() async {
    final publicKey = await _secureStorage.read(key: AppConstants.walletPublicKeyKey);
    return publicKey != null;
  }

  /// Sign a message with the current wallet's private key.
  Uint8List sign(Uint8List message) {
    if (_currentKeypair == null) {
      throw WalletException('No wallet loaded');
    }
    return CryptoUtils.sign(message, _currentKeypair!.privateKey);
  }

  /// Get the public key as bytes.
  Uint8List getPublicKeyBytes() {
    if (_currentKeypair == null) {
      throw WalletException('No wallet loaded');
    }
    return _currentKeypair!.publicKey;
  }

  /// Update the wallet balance.
  void updateBalance(double balanceSol, double balanceUsd) {
    if (_currentWallet != null) {
      _currentWallet = _currentWallet!.copyWith(
        balanceSol: balanceSol,
        balanceUsd: balanceUsd,
      );
    }
  }

  /// Delete the wallet from storage.
  Future<void> deleteWallet() async {
    await _secureStorage.delete(key: AppConstants.walletMnemonicKey);
    await _secureStorage.delete(key: AppConstants.walletPrivateKeyKey);
    await _secureStorage.delete(key: AppConstants.walletPublicKeyKey);

    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(AppConstants.hasOnboardedKey);

    _currentKeypair = null;
    _currentWallet = null;
  }

  /// Check if the user has completed onboarding.
  Future<bool> hasOnboarded() async {
    final prefs = await SharedPreferences.getInstance();
    return prefs.getBool(AppConstants.hasOnboardedKey) ?? false;
  }

  /// Mark onboarding as complete.
  Future<void> setOnboarded() async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(AppConstants.hasOnboardedKey, true);
  }

  // Private helpers

  Future<void> _saveWallet(String mnemonic, KeypairData keypair) async {
    await _secureStorage.write(
      key: AppConstants.walletMnemonicKey,
      value: mnemonic,
    );
    await _secureStorage.write(
      key: AppConstants.walletPrivateKeyKey,
      value: keypair.privateKeyBase58,
    );
    await _secureStorage.write(
      key: AppConstants.walletPublicKeyKey,
      value: keypair.publicKeyBase58,
    );
  }

  Future<void> _saveWalletFromKey(KeypairData keypair) async {
    await _secureStorage.write(
      key: AppConstants.walletPrivateKeyKey,
      value: keypair.privateKeyBase58,
    );
    await _secureStorage.write(
      key: AppConstants.walletPublicKeyKey,
      value: keypair.publicKeyBase58,
    );
  }
}

class WalletException implements Exception {
  final String message;
  WalletException(this.message);

  @override
  String toString() => 'WalletException: $message';
}
