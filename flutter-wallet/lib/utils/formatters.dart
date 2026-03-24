import 'package:intl/intl.dart';
import 'constants.dart';

class Formatters {
  /// Format lamports to SOL string with specified decimal places.
  static String lamportsToSol(int lamports, {int decimals = 4}) {
    final sol = lamports / AppConstants.lamportsPerSol;
    return sol.toStringAsFixed(decimals);
  }

  /// Format SOL amount with comma grouping.
  static String formatSol(double sol, {int decimals = 4}) {
    final formatter = NumberFormat('#,##0.${'0' * decimals}', 'en_US');
    return formatter.format(sol);
  }

  /// Format USD amount.
  static String formatUsd(double amount) {
    final formatter = NumberFormat.currency(locale: 'en_US', symbol: '\$');
    return formatter.format(amount);
  }

  /// Format percentage.
  static String formatPercent(double percent, {int decimals = 2}) {
    final sign = percent >= 0 ? '+' : '';
    return '$sign${percent.toStringAsFixed(decimals)}%';
  }

  /// Truncate a wallet address for display.
  static String truncateAddress(String address, {int prefixLen = 4, int suffixLen = 4}) {
    if (address.length <= prefixLen + suffixLen + 3) return address;
    return '${address.substring(0, prefixLen)}...${address.substring(address.length - suffixLen)}';
  }

  /// Format a timestamp to a relative time string.
  static String timeAgo(DateTime dateTime) {
    final now = DateTime.now();
    final diff = now.difference(dateTime);

    if (diff.inSeconds < 60) return 'Just now';
    if (diff.inMinutes < 60) return '${diff.inMinutes}m ago';
    if (diff.inHours < 24) return '${diff.inHours}h ago';
    if (diff.inDays < 7) return '${diff.inDays}d ago';
    if (diff.inDays < 30) return '${(diff.inDays / 7).floor()}w ago';
    return DateFormat('MMM d, yyyy').format(dateTime);
  }

  /// Format date for transaction detail.
  static String formatDate(DateTime dateTime) {
    return DateFormat('MMM d, yyyy \'at\' h:mm a').format(dateTime);
  }

  /// Format large numbers with K, M, B suffixes.
  static String compactNumber(double number) {
    final formatter = NumberFormat.compact(locale: 'en_US');
    return formatter.format(number);
  }

  /// Convert SOL string to lamports.
  static int solToLamports(String solString) {
    final sol = double.tryParse(solString) ?? 0;
    return (sol * AppConstants.lamportsPerSol).round();
  }
}
