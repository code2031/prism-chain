import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../../providers/wallet_provider.dart';
import '../../providers/network_provider.dart';
import '../../theme/app_theme.dart';
import '../../widgets/balance_card.dart';
import '../../widgets/token_list_item.dart';
import '../../widgets/network_badge.dart';
import '../../widgets/loading_shimmer.dart';
import '../send/send_screen.dart';
import '../receive/receive_screen.dart';
import '../token/token_detail_screen.dart';
import '../staking/staking_screen.dart';
import '../create_token/manage_tokens_screen.dart';

class PortfolioTab extends StatelessWidget {
  const PortfolioTab({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer2<WalletProvider, NetworkProvider>(
      builder: (context, walletProvider, networkProvider, _) {
        return Stack(
          children: [
          SafeArea(
          child: RefreshIndicator(
            onRefresh: () => walletProvider.refreshAll(),
            color: AppTheme.solanaPurple,
            backgroundColor: AppTheme.darkCard,
            child: CustomScrollView(
              physics: const AlwaysScrollableScrollPhysics(),
              slivers: [
                // App bar area
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(20, 16, 20, 0),
                    child: Row(
                      mainAxisAlignment: MainAxisAlignment.spaceBetween,
                      children: [
                        Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            const Text(
                              'SolClone Wallet',
                              style: TextStyle(
                                color: AppTheme.textPrimary,
                                fontSize: 20,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            const SizedBox(height: 4),
                            if (walletProvider.wallet != null)
                              Text(
                                walletProvider.wallet!.shortAddress,
                                style: const TextStyle(
                                  color: AppTheme.textTertiary,
                                  fontSize: 13,
                                  fontFamily: 'monospace',
                                ),
                              ),
                          ],
                        ),
                        NetworkBadge(
                          network: networkProvider.currentNetwork,
                          isConnected: networkProvider.isConnected,
                        ),
                      ],
                    ),
                  ),
                ),

                // Balance card
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.only(top: 20),
                    child: walletProvider.isLoading
                        ? LoadingShimmer.balanceCard()
                        : BalanceCard(
                            balanceSol: walletProvider.wallet?.balanceSol ?? 0,
                            balanceUsd: walletProvider.wallet?.balanceUsd ?? 0,
                            showAirdrop: networkProvider.canAirdrop,
                            onSend: () {
                              Navigator.of(context).push(
                                MaterialPageRoute(
                                  builder: (_) => const SendScreen(),
                                ),
                              );
                            },
                            onReceive: () {
                              Navigator.of(context).push(
                                MaterialPageRoute(
                                  builder: (_) => const ReceiveScreen(),
                                ),
                              );
                            },
                            onAirdrop: () async {
                              try {
                                await walletProvider.requestAirdrop();
                                if (context.mounted) {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      content: const Text(
                                          'Airdrop of 1 SOL requested!'),
                                      backgroundColor: AppTheme.darkCard,
                                      behavior: SnackBarBehavior.floating,
                                      shape: RoundedRectangleBorder(
                                        borderRadius:
                                            BorderRadius.circular(12),
                                      ),
                                    ),
                                  );
                                }
                              } catch (e) {
                                if (context.mounted) {
                                  ScaffoldMessenger.of(context).showSnackBar(
                                    SnackBar(
                                      content: Text('Airdrop failed: $e'),
                                      backgroundColor: AppTheme.error,
                                      behavior: SnackBarBehavior.floating,
                                      shape: RoundedRectangleBorder(
                                        borderRadius:
                                            BorderRadius.circular(12),
                                      ),
                                    ),
                                  );
                                }
                              }
                            },
                          ),
                  ),
                ),

                // Quick actions row
                SliverToBoxAdapter(
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(16, 20, 16, 4),
                    child: Row(
                      children: [
                        _QuickAction(
                          icon: Icons.lock_rounded,
                          label: 'Stake',
                          color: AppTheme.solanaPurple,
                          onTap: () {
                            Navigator.of(context).push(
                              MaterialPageRoute(
                                builder: (_) => const StakingScreen(),
                              ),
                            );
                          },
                        ),
                        const SizedBox(width: 10),
                        _QuickAction(
                          icon: Icons.add_circle_outline_rounded,
                          label: 'Create',
                          color: AppTheme.solanaGreen,
                          onTap: () {
                            Navigator.of(context).push(
                              MaterialPageRoute(
                                builder: (_) => const ManageTokensScreen(),
                              ),
                            );
                          },
                        ),
                      ],
                    ),
                  ),
                ),

                // Token list header
                const SliverToBoxAdapter(
                  child: Padding(
                    padding: EdgeInsets.fromLTRB(20, 20, 20, 10),
                    child: Text(
                      'Tokens',
                      style: TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 18,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ),
                ),

                // Token list
                if (walletProvider.isLoading)
                  SliverToBoxAdapter(
                    child: Column(
                      children: List.generate(
                          3, (_) => LoadingShimmer.tokenRow()),
                    ),
                  )
                else if (walletProvider.tokens.isEmpty)
                  const SliverToBoxAdapter(
                    child: Padding(
                      padding: EdgeInsets.all(40),
                      child: Center(
                        child: Column(
                          children: [
                            Icon(Icons.token_rounded,
                                color: AppTheme.textTertiary, size: 48),
                            SizedBox(height: 12),
                            Text(
                              'No tokens yet',
                              style: TextStyle(
                                color: AppTheme.textTertiary,
                                fontSize: 16,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  )
                else
                  SliverPadding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    sliver: SliverList(
                      delegate: SliverChildBuilderDelegate(
                        (context, index) {
                          final token = walletProvider.tokens[index];
                          return Padding(
                            padding: const EdgeInsets.only(bottom: 8),
                            child: TokenListItem(
                              token: token,
                              onTap: () {
                                Navigator.of(context).push(
                                  MaterialPageRoute(
                                    builder: (_) =>
                                        TokenDetailScreen(token: token),
                                  ),
                                );
                              },
                            ),
                          );
                        },
                        childCount: walletProvider.tokens.length,
                      ),
                    ),
                  ),

                // Bottom padding for FAB clearance
                const SliverToBoxAdapter(child: SizedBox(height: 80)),
              ],
            ),
          ),
        ),
          // Floating action button — Create Token / NFT
          Positioned(
            right: 20,
            bottom: 20,
            child: FloatingActionButton(
              heroTag: 'create_token_fab',
              onPressed: () {
                Navigator.of(context).push(
                  MaterialPageRoute(
                    builder: (_) => const ManageTokensScreen(),
                  ),
                );
              },
              backgroundColor: AppTheme.solanaPurple,
              child: const Icon(Icons.add_rounded, color: Colors.white, size: 28),
            ),
          ),
          ],
        );
      },
    );
  }
}

class _QuickAction extends StatelessWidget {
  final IconData icon;
  final String label;
  final Color color;
  final VoidCallback? onTap;

  const _QuickAction({
    required this.icon,
    required this.label,
    required this.color,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(12),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.1),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(
              color: color.withValues(alpha: 0.2),
            ),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(icon, color: color, size: 18),
              const SizedBox(width: 8),
              Text(
                label,
                style: TextStyle(
                  color: color,
                  fontSize: 13,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
