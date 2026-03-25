import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import 'providers/wallet_provider.dart';
import 'providers/network_provider.dart';
import 'services/rpc_service.dart';
import 'services/wallet_service.dart';
import 'services/price_service.dart';
import 'theme/app_theme.dart';
import 'screens/splash/splash_screen.dart';
import 'screens/onboarding/welcome_screen.dart';
import 'screens/onboarding/create_wallet_screen.dart';
import 'screens/onboarding/import_wallet_screen.dart';
import 'screens/home/home_screen.dart';
import 'screens/send/send_screen.dart';
import 'screens/receive/receive_screen.dart';
import 'screens/staking/staking_screen.dart';
import 'screens/dapp_browser/dapp_browser_screen.dart';
import 'screens/create_token/create_token_screen.dart';
import 'screens/create_token/mint_nft_screen.dart';
import 'screens/create_token/manage_tokens_screen.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();

  // Lock orientation to portrait
  SystemChrome.setPreferredOrientations([
    DeviceOrientation.portraitUp,
    DeviceOrientation.portraitDown,
  ]);

  // Set system UI overlay style for the dark theme
  SystemChrome.setSystemUIOverlayStyle(const SystemUiOverlayStyle(
    statusBarColor: Colors.transparent,
    statusBarIconBrightness: Brightness.light,
    systemNavigationBarColor: AppTheme.darkSurface,
    systemNavigationBarIconBrightness: Brightness.light,
  ));

  // Create shared service instances
  final rpcService = RpcService();
  final walletService = WalletService();
  final priceService = PriceService();

  runApp(SolCloneWallet(
    rpcService: rpcService,
    walletService: walletService,
    priceService: priceService,
  ));
}

class SolCloneWallet extends StatelessWidget {
  final RpcService rpcService;
  final WalletService walletService;
  final PriceService priceService;

  const SolCloneWallet({
    super.key,
    required this.rpcService,
    required this.walletService,
    required this.priceService,
  });

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        Provider<RpcService>.value(value: rpcService),
        Provider<WalletService>.value(value: walletService),
        Provider<PriceService>.value(value: priceService),
        ChangeNotifierProvider<NetworkProvider>(
          create: (_) => NetworkProvider(rpcService),
        ),
        ChangeNotifierProvider<WalletProvider>(
          create: (_) => WalletProvider(walletService, rpcService, priceService),
        ),
      ],
      child: MaterialApp(
        title: 'SolClone Wallet',
        debugShowCheckedModeBanner: false,
        theme: AppTheme.darkTheme(),
        home: const SplashScreen(),
        routes: {
          '/welcome': (context) => const WelcomeScreen(),
          '/create-wallet': (context) => const CreateWalletScreen(),
          '/import-wallet': (context) => const ImportWalletScreen(),
          '/home': (context) => const HomeScreen(),
          '/send': (context) => const SendScreen(),
          '/receive': (context) => const ReceiveScreen(),
          '/staking': (context) => const StakingScreen(),
          '/dapp-browser': (context) => const DappBrowserScreen(),
          '/create-token': (context) => const CreateTokenScreen(),
          '/mint-nft': (context) => const MintNftScreen(),
          '/manage-tokens': (context) => const ManageTokensScreen(),
        },
      ),
    );
  }
}
