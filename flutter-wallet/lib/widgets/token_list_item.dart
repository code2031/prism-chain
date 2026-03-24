import 'package:flutter/material.dart';
import '../models/token.dart';
import '../theme/app_theme.dart';
import '../utils/formatters.dart';

class TokenListItem extends StatelessWidget {
  final TokenModel token;
  final VoidCallback? onTap;

  const TokenListItem({
    super.key,
    required this.token,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final isPositive = token.priceChangePercent24h >= 0;

    return Material(
      color: Colors.transparent,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 14),
          child: Row(
            children: [
              // Token icon
              Container(
                width: 44,
                height: 44,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  gradient: token.isNative
                      ? AppTheme.primaryGradient
                      : LinearGradient(
                          colors: [
                            AppTheme.darkCardLight,
                            AppTheme.darkCard,
                          ],
                        ),
                ),
                child: Center(
                  child: token.isNative
                      ? const Text(
                          '\u25C6',
                          style: TextStyle(
                            color: Colors.white,
                            fontSize: 20,
                          ),
                        )
                      : Text(
                          token.symbol.isNotEmpty ? token.symbol[0] : '?',
                          style: const TextStyle(
                            color: AppTheme.textPrimary,
                            fontSize: 18,
                            fontWeight: FontWeight.bold,
                          ),
                        ),
                ),
              ),
              const SizedBox(width: 14),

              // Token name and symbol
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      token.name,
                      style: const TextStyle(
                        color: AppTheme.textPrimary,
                        fontSize: 15,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Row(
                      children: [
                        Text(
                          '${token.balance.toStringAsFixed(token.balance < 1 ? 6 : 4)} ${token.symbol}',
                          style: const TextStyle(
                            color: AppTheme.textTertiary,
                            fontSize: 13,
                          ),
                        ),
                        if (token.priceUsd > 0) ...[
                          const SizedBox(width: 6),
                          Text(
                            '\u2022',
                            style: TextStyle(
                              color: AppTheme.textTertiary.withValues(alpha: 0.5),
                              fontSize: 8,
                            ),
                          ),
                          const SizedBox(width: 6),
                          Text(
                            Formatters.formatUsd(token.priceUsd),
                            style: const TextStyle(
                              color: AppTheme.textTertiary,
                              fontSize: 13,
                            ),
                          ),
                        ],
                      ],
                    ),
                  ],
                ),
              ),

              // Value and change
              Column(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: [
                  Text(
                    token.valueUsd > 0
                        ? Formatters.formatUsd(token.valueUsd)
                        : '-',
                    style: const TextStyle(
                      color: AppTheme.textPrimary,
                      fontSize: 15,
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                  if (token.priceChangePercent24h != 0) ...[
                    const SizedBox(height: 2),
                    Text(
                      Formatters.formatPercent(token.priceChangePercent24h),
                      style: TextStyle(
                        color: isPositive ? AppTheme.success : AppTheme.error,
                        fontSize: 13,
                        fontWeight: FontWeight.w500,
                      ),
                    ),
                  ],
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
