import 'dart:convert';
import 'dart:typed_data';
import 'package:bip39/bip39.dart' as bip39;
import 'package:ed25519_hd_key/ed25519_hd_key.dart';
import 'package:pinenacl/ed25519.dart';

class CryptoUtils {
  /// Generate a new 12-word mnemonic phrase.
  static String generateMnemonic() {
    return bip39.generateMnemonic();
  }

  /// Validate a mnemonic phrase.
  static bool validateMnemonic(String mnemonic) {
    return bip39.validateMnemonic(mnemonic);
  }

  /// Derive an Ed25519 keypair from a mnemonic phrase.
  /// Uses the Solana derivation path: m/44'/501'/0'/0'
  static Future<KeypairData> keypairFromMnemonic(String mnemonic) async {
    final seed = bip39.mnemonicToSeed(mnemonic);
    final derivationPath = "m/44'/501'/0'/0'";
    final keyData = await ED25519_HD_KEY.derivePath(derivationPath, seed);
    final signingKey = SigningKey.fromSeed(Uint8List.fromList(keyData.key));
    final publicKey = signingKey.verifyKey;

    return KeypairData(
      privateKey: Uint8List.fromList(signingKey.prefix),
      publicKey: Uint8List.fromList(publicKey.asTypedList),
    );
  }

  /// Derive a keypair from a raw private key (64-byte or 32-byte seed).
  static KeypairData keypairFromPrivateKey(Uint8List privateKeyBytes) {
    Uint8List seed;
    if (privateKeyBytes.length == 64) {
      seed = Uint8List.fromList(privateKeyBytes.sublist(0, 32));
    } else if (privateKeyBytes.length == 32) {
      seed = privateKeyBytes;
    } else {
      throw ArgumentError('Invalid private key length: ${privateKeyBytes.length}');
    }

    final signingKey = SigningKey.fromSeed(seed);
    final publicKey = signingKey.verifyKey;

    return KeypairData(
      privateKey: Uint8List.fromList(signingKey.prefix),
      publicKey: Uint8List.fromList(publicKey.asTypedList),
    );
  }

  /// Sign a message with a private key.
  static Uint8List sign(Uint8List message, Uint8List privateKey) {
    Uint8List seed;
    if (privateKey.length == 64) {
      seed = Uint8List.fromList(privateKey.sublist(0, 32));
    } else {
      seed = privateKey;
    }
    final signingKey = SigningKey.fromSeed(seed);
    final signedMessage = signingKey.sign(message);
    return Uint8List.fromList(signedMessage.signature.asTypedList);
  }

  /// Encode bytes to base58.
  static String toBase58(Uint8List bytes) {
    const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    if (bytes.isEmpty) return '';

    // Count leading zeros
    var leadingZeros = 0;
    for (var b in bytes) {
      if (b == 0) {
        leadingZeros++;
      } else {
        break;
      }
    }

    // Convert to big integer
    var num = BigInt.zero;
    for (var b in bytes) {
      num = num * BigInt.from(256) + BigInt.from(b);
    }

    // Encode
    final result = StringBuffer();
    while (num > BigInt.zero) {
      final remainder = (num % BigInt.from(58)).toInt();
      num = num ~/ BigInt.from(58);
      result.write(alphabet[remainder]);
    }

    // Add leading '1's for each leading zero byte
    for (var i = 0; i < leadingZeros; i++) {
      result.write('1');
    }

    return result.toString().split('').reversed.join();
  }

  /// Decode base58 string to bytes.
  static Uint8List fromBase58(String encoded) {
    const alphabet = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    if (encoded.isEmpty) return Uint8List(0);

    var num = BigInt.zero;
    for (var char in encoded.split('')) {
      final index = alphabet.indexOf(char);
      if (index < 0) throw FormatException('Invalid base58 character: $char');
      num = num * BigInt.from(58) + BigInt.from(index);
    }

    // Convert BigInt to bytes
    final hexStr = num.toRadixString(16).padLeft(2, '0');
    final paddedHex = hexStr.length.isOdd ? '0$hexStr' : hexStr;
    final rawBytes = <int>[];
    for (var i = 0; i < paddedHex.length; i += 2) {
      rawBytes.add(int.parse(paddedHex.substring(i, i + 2), radix: 16));
    }

    // Count leading '1's
    var leadingOnes = 0;
    for (var char in encoded.split('')) {
      if (char == '1') {
        leadingOnes++;
      } else {
        break;
      }
    }

    final result = List<int>.filled(leadingOnes, 0) + rawBytes;
    return Uint8List.fromList(result);
  }

  /// Convert bytes to hex string.
  static String toHex(Uint8List bytes) {
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }

  /// Convert hex string to bytes.
  static Uint8List fromHex(String hex) {
    final result = Uint8List(hex.length ~/ 2);
    for (var i = 0; i < hex.length; i += 2) {
      result[i ~/ 2] = int.parse(hex.substring(i, i + 2), radix: 16);
    }
    return result;
  }

  /// Encode bytes to base64.
  static String toBase64(Uint8List bytes) => base64Encode(bytes);

  /// Decode base64 to bytes.
  static Uint8List fromBase64(String encoded) => base64Decode(encoded);
}

/// Holds a keypair's public and private keys.
class KeypairData {
  final Uint8List privateKey;
  final Uint8List publicKey;

  KeypairData({required this.privateKey, required this.publicKey});

  String get publicKeyBase58 => CryptoUtils.toBase58(publicKey);
  String get privateKeyBase58 => CryptoUtils.toBase58(privateKey);
}
