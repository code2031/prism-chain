import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';
import '../services/rpc_service.dart';
import '../utils/constants.dart';

enum NetworkType { mainnet, testnet, devnet, localnet, custom }

class NetworkInfo {
  final NetworkType type;
  final String name;
  final String rpcUrl;
  final bool canAirdrop;

  const NetworkInfo({
    required this.type,
    required this.name,
    required this.rpcUrl,
    this.canAirdrop = false,
  });
}

class NetworkProvider extends ChangeNotifier {
  final RpcService _rpcService;

  NetworkType _currentNetwork = NetworkType.localnet;
  String _customRpcUrl = '';
  bool _isConnected = false;
  String? _clusterVersion;

  static const Map<NetworkType, NetworkInfo> networks = {
    NetworkType.mainnet: NetworkInfo(
      type: NetworkType.mainnet,
      name: 'Mainnet Beta',
      rpcUrl: AppConstants.mainnetRpcUrl,
    ),
    NetworkType.testnet: NetworkInfo(
      type: NetworkType.testnet,
      name: 'Testnet',
      rpcUrl: AppConstants.testnetRpcUrl,
      canAirdrop: true,
    ),
    NetworkType.devnet: NetworkInfo(
      type: NetworkType.devnet,
      name: 'Devnet',
      rpcUrl: AppConstants.devnetRpcUrl,
      canAirdrop: true,
    ),
    NetworkType.localnet: NetworkInfo(
      type: NetworkType.localnet,
      name: 'Localnet',
      rpcUrl: AppConstants.localRpcUrl,
      canAirdrop: true,
    ),
  };

  NetworkProvider(this._rpcService);

  NetworkType get currentNetwork => _currentNetwork;
  bool get isConnected => _isConnected;
  String? get clusterVersion => _clusterVersion;
  String get customRpcUrl => _customRpcUrl;

  NetworkInfo get currentNetworkInfo {
    if (_currentNetwork == NetworkType.custom) {
      return NetworkInfo(
        type: NetworkType.custom,
        name: 'Custom',
        rpcUrl: _customRpcUrl,
        canAirdrop: true,
      );
    }
    return networks[_currentNetwork]!;
  }

  bool get canAirdrop => currentNetworkInfo.canAirdrop;

  /// Initialize: load saved network preference.
  Future<void> init() async {
    final prefs = await SharedPreferences.getInstance();
    final savedNetwork = prefs.getString(AppConstants.selectedNetworkKey);
    final savedCustomUrl = prefs.getString(AppConstants.customRpcUrlKey);

    if (savedCustomUrl != null) _customRpcUrl = savedCustomUrl;

    if (savedNetwork != null) {
      _currentNetwork = NetworkType.values.firstWhere(
        (n) => n.name == savedNetwork,
        orElse: () => NetworkType.localnet,
      );
    }

    _rpcService.setRpcUrl(currentNetworkInfo.rpcUrl);
    await _checkConnection();
  }

  /// Switch to a different network.
  Future<void> switchNetwork(NetworkType network, {String? customUrl}) async {
    _currentNetwork = network;
    if (network == NetworkType.custom && customUrl != null) {
      _customRpcUrl = customUrl;
    }

    _rpcService.setRpcUrl(currentNetworkInfo.rpcUrl);

    // Persist selection
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(AppConstants.selectedNetworkKey, network.name);
    if (customUrl != null) {
      await prefs.setString(AppConstants.customRpcUrlKey, customUrl);
    }

    await _checkConnection();
    notifyListeners();
  }

  /// Check if the RPC node is reachable.
  Future<void> _checkConnection() async {
    try {
      _isConnected = await _rpcService.isHealthy();
      if (_isConnected) {
        final version = await _rpcService.getVersion();
        _clusterVersion = version['solana-core'] as String?;
      }
    } catch (_) {
      _isConnected = false;
      _clusterVersion = null;
    }
    notifyListeners();
  }

  /// Manually refresh connection status.
  Future<void> refreshConnection() async {
    await _checkConnection();
  }
}
