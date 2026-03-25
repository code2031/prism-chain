import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:provider/provider.dart';
import '../../providers/wallet_provider.dart';
import '../../services/rpc_service.dart';
import '../../services/wallet_service.dart';
import '../../services/token_service.dart';
import '../../theme/app_theme.dart';
import '../../utils/formatters.dart';
import '../../widgets/gradient_button.dart';

class CreateTokenScreen extends StatefulWidget {
  const CreateTokenScreen({super.key});

  @override
  State<CreateTokenScreen> createState() => _CreateTokenScreenState();
}

class _CreateTokenScreenState extends State<CreateTokenScreen>
    with SingleTickerProviderStateMixin {
  final _formKey = GlobalKey<FormState>();
  final _nameController = TextEditingController();
  final _symbolController = TextEditingController();
  final _supplyController = TextEditingController(text: '1000000');
  int _decimals = 9;

  bool _isCreating = false;
  String _progressStep = '';
  CreateTokenResult? _result;
  String? _errorMessage;

  late AnimationController _successAnimController;
  late Animation<double> _successFadeAnim;

  @override
  void initState() {
    super.initState();
    _successAnimController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 600),
    );
    _successFadeAnim = CurvedAnimation(
      parent: _successAnimController,
      curve: Curves.easeOut,
    );
  }

  @override
  void dispose() {
    _nameController.dispose();
    _symbolController.dispose();
    _supplyController.dispose();
    _successAnimController.dispose();
    super.dispose();
  }

  Future<void> _createToken() async {
    if (!_formKey.currentState!.validate()) return;

    setState(() {
      _isCreating = true;
      _progressStep = '';
      _result = null;
      _errorMessage = null;
    });

    try {
      final walletProvider = context.read<WalletProvider>();

      final tokenService = TokenService(
        rpcService: Provider.of<RpcService>(context, listen: false),
        walletService: Provider.of<WalletService>(context, listen: false),
      );

      final supply = double.tryParse(_supplyController.text) ?? 0;

      final result = await tokenService.createToken(
        decimals: _decimals,
        name: _nameController.text.trim(),
        symbol: _symbolController.text.trim().toUpperCase(),
        initialSupply: supply,
        onProgress: (step) {
          if (mounted) {
            setState(() => _progressStep = step);
          }
        },
      );

      if (mounted) {
        setState(() {
          _result = result;
          _isCreating = false;
        });
        _successAnimController.forward();
        // Refresh wallet to pick up new token
        walletProvider.refreshAll();
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _isCreating = false;
          _errorMessage = _formatError(e.toString());
        });
      }
    }
  }

  String _formatError(String error) {
    if (error.contains('insufficient')) {
      return 'Insufficient SOL balance. Request an airdrop first.';
    }
    if (error.contains('Connection failed')) {
      return 'Could not connect to the network. Check your RPC settings.';
    }
    if (error.contains('No wallet loaded')) {
      return 'No wallet loaded. Please create or import a wallet first.';
    }
    // Strip exception type prefix for cleaner display.
    return error
        .replaceFirst('TokenServiceException: ', '')
        .replaceFirst('RpcException: ', '')
        .replaceFirst('Exception: ', '');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: AppTheme.darkBg,
      appBar: AppBar(
        title: const Text('Create Token'),
        backgroundColor: AppTheme.darkBg,
        elevation: 0,
      ),
      body: SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(20),
          child: _result != null ? _buildSuccessView() : _buildForm(),
        ),
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Form
  // ---------------------------------------------------------------------------

  Widget _buildForm() {
    return Form(
      key: _formKey,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Header illustration
          Center(
            child: Container(
              width: 80,
              height: 80,
              decoration: BoxDecoration(
                shape: BoxShape.circle,
                gradient: AppTheme.primaryGradient,
                boxShadow: [
                  BoxShadow(
                    color: AppTheme.solanaPurple.withValues(alpha: 0.3),
                    blurRadius: 24,
                    offset: const Offset(0, 8),
                  ),
                ],
              ),
              child: const Icon(Icons.token_rounded, color: Colors.white, size: 40),
            ),
          ),
          const SizedBox(height: 24),
          const Center(
            child: Text(
              'Create a New SPL Token',
              style: TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 22,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          const SizedBox(height: 8),
          const Center(
            child: Text(
              'Deploy your own token on the SolClone network',
              style: TextStyle(color: AppTheme.textTertiary, fontSize: 14),
            ),
          ),
          const SizedBox(height: 32),

          // Token Name
          _buildLabel('Token Name'),
          const SizedBox(height: 8),
          TextFormField(
            controller: _nameController,
            style: const TextStyle(color: AppTheme.textPrimary),
            decoration: const InputDecoration(
              hintText: 'e.g. My Token',
              prefixIcon: Icon(Icons.label_outline, color: AppTheme.textTertiary),
            ),
            validator: (v) =>
                (v == null || v.trim().isEmpty) ? 'Token name is required' : null,
          ),
          const SizedBox(height: 20),

          // Token Symbol
          _buildLabel('Token Symbol'),
          const SizedBox(height: 8),
          TextFormField(
            controller: _symbolController,
            style: const TextStyle(color: AppTheme.textPrimary),
            textCapitalization: TextCapitalization.characters,
            inputFormatters: [
              LengthLimitingTextInputFormatter(10),
              FilteringTextInputFormatter.allow(RegExp(r'[a-zA-Z0-9]')),
            ],
            decoration: const InputDecoration(
              hintText: 'e.g. MTK',
              prefixIcon: Icon(Icons.short_text_rounded, color: AppTheme.textTertiary),
            ),
            validator: (v) =>
                (v == null || v.trim().isEmpty) ? 'Symbol is required' : null,
          ),
          const SizedBox(height: 20),

          // Decimals
          _buildLabel('Decimals'),
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            decoration: BoxDecoration(
              color: AppTheme.darkCard,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: AppTheme.darkCardLight),
            ),
            child: Row(
              children: [
                const Icon(Icons.decimal_increase, color: AppTheme.textTertiary, size: 20),
                const SizedBox(width: 12),
                Expanded(
                  child: Slider(
                    value: _decimals.toDouble(),
                    min: 0,
                    max: 18,
                    divisions: 18,
                    activeColor: AppTheme.solanaPurple,
                    inactiveColor: AppTheme.darkCardLight,
                    onChanged: _isCreating
                        ? null
                        : (v) => setState(() => _decimals = v.round()),
                  ),
                ),
                Container(
                  width: 36,
                  alignment: Alignment.center,
                  child: Text(
                    '$_decimals',
                    style: const TextStyle(
                      color: AppTheme.solanaGreen,
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(height: 20),

          // Initial Supply
          _buildLabel('Initial Supply'),
          const SizedBox(height: 8),
          TextFormField(
            controller: _supplyController,
            style: const TextStyle(color: AppTheme.textPrimary),
            keyboardType: const TextInputType.numberWithOptions(decimal: true),
            inputFormatters: [
              FilteringTextInputFormatter.allow(RegExp(r'[0-9.]')),
            ],
            decoration: const InputDecoration(
              hintText: '0',
              prefixIcon:
                  Icon(Icons.inventory_2_outlined, color: AppTheme.textTertiary),
            ),
            validator: (v) {
              if (v != null && v.isNotEmpty) {
                final parsed = double.tryParse(v);
                if (parsed == null || parsed < 0) return 'Enter a valid number';
              }
              return null;
            },
          ),
          const SizedBox(height: 8),
          Text(
            'Leave at 0 to create a mint without minting tokens initially.',
            style: TextStyle(
              color: AppTheme.textTertiary,
              fontSize: 12,
            ),
          ),
          const SizedBox(height: 32),

          // Error
          if (_errorMessage != null) ...[
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(14),
              decoration: BoxDecoration(
                color: AppTheme.error.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(color: AppTheme.error.withValues(alpha: 0.3)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.error_outline, color: AppTheme.error, size: 20),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      _errorMessage!,
                      style: const TextStyle(color: AppTheme.error, fontSize: 13),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 16),
          ],

          // Progress indicator
          if (_isCreating) ...[
            Container(
              width: double.infinity,
              padding: const EdgeInsets.all(16),
              decoration: BoxDecoration(
                color: AppTheme.solanaPurple.withValues(alpha: 0.08),
                borderRadius: BorderRadius.circular(12),
                border: Border.all(
                  color: AppTheme.solanaPurple.withValues(alpha: 0.2),
                ),
              ),
              child: Row(
                children: [
                  const SizedBox(
                    width: 20,
                    height: 20,
                    child: CircularProgressIndicator(
                      strokeWidth: 2,
                      valueColor:
                          AlwaysStoppedAnimation<Color>(AppTheme.solanaPurple),
                    ),
                  ),
                  const SizedBox(width: 14),
                  Expanded(
                    child: Text(
                      _progressStep,
                      style: const TextStyle(
                        color: AppTheme.textSecondary,
                        fontSize: 14,
                      ),
                    ),
                  ),
                ],
              ),
            ),
            const SizedBox(height: 20),
          ],

          // Create button
          GradientButton(
            text: 'Create Token',
            icon: Icons.rocket_launch_rounded,
            isLoading: _isCreating,
            onPressed: _isCreating ? null : _createToken,
          ),
          const SizedBox(height: 40),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Success View
  // ---------------------------------------------------------------------------

  Widget _buildSuccessView() {
    return FadeTransition(
      opacity: _successFadeAnim,
      child: Column(
        children: [
          const SizedBox(height: 20),
          // Success icon
          Container(
            width: 88,
            height: 88,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: AppTheme.success.withValues(alpha: 0.12),
            ),
            child: const Icon(
              Icons.check_circle_rounded,
              color: AppTheme.success,
              size: 56,
            ),
          ),
          const SizedBox(height: 20),
          const Text(
            'Token Created!',
            style: TextStyle(
              color: AppTheme.textPrimary,
              fontSize: 24,
              fontWeight: FontWeight.bold,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            '${_nameController.text.trim()} (${_symbolController.text.trim().toUpperCase()})',
            style: const TextStyle(color: AppTheme.textSecondary, fontSize: 16),
          ),
          const SizedBox(height: 32),

          // Details card
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(20),
            decoration: AppTheme.glassDecoration(),
            child: Column(
              children: [
                _detailRow('Mint Address', _result!.mintAddress, copyable: true),
                const Divider(color: AppTheme.darkCardLight, height: 24),
                _detailRow('Token Account', _result!.tokenAccountAddress,
                    copyable: true),
                const Divider(color: AppTheme.darkCardLight, height: 24),
                _detailRow('Decimals', '$_decimals'),
                if (_result!.supply > 0) ...[
                  const Divider(color: AppTheme.darkCardLight, height: 24),
                  _detailRow(
                    'Initial Supply',
                    Formatters.compactNumber(_result!.supply),
                  ),
                ],
                const Divider(color: AppTheme.darkCardLight, height: 24),
                _detailRow('Signature', _result!.signature, copyable: true),
              ],
            ),
          ),
          const SizedBox(height: 24),

          // Action buttons
          Row(
            children: [
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: () => Navigator.of(context).pop(),
                  icon: const Icon(Icons.arrow_back, size: 18),
                  label: const Text('Back'),
                  style: OutlinedButton.styleFrom(
                    foregroundColor: AppTheme.textPrimary,
                    side: const BorderSide(color: AppTheme.darkCardLight),
                    padding: const EdgeInsets.symmetric(vertical: 14),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: GradientButton(
                  text: 'Create Another',
                  icon: Icons.add,
                  onPressed: () {
                    setState(() {
                      _result = null;
                      _nameController.clear();
                      _symbolController.clear();
                      _supplyController.text = '1000000';
                      _decimals = 9;
                      _errorMessage = null;
                    });
                    _successAnimController.reset();
                  },
                  height: 48,
                ),
              ),
            ],
          ),
          const SizedBox(height: 40),
        ],
      ),
    );
  }

  // ---------------------------------------------------------------------------
  // Shared widgets
  // ---------------------------------------------------------------------------

  Widget _buildLabel(String text) {
    return Text(
      text,
      style: const TextStyle(
        color: AppTheme.textSecondary,
        fontSize: 14,
        fontWeight: FontWeight.w500,
      ),
    );
  }

  Widget _detailRow(String label, String value, {bool copyable = false}) {
    final displayValue = value.length > 20
        ? '${value.substring(0, 8)}...${value.substring(value.length - 8)}'
        : value;

    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: const TextStyle(color: AppTheme.textTertiary, fontSize: 13),
        ),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              displayValue,
              style: const TextStyle(
                color: AppTheme.textPrimary,
                fontSize: 13,
                fontFamily: 'monospace',
              ),
            ),
            if (copyable) ...[
              const SizedBox(width: 6),
              GestureDetector(
                onTap: () {
                  Clipboard.setData(ClipboardData(text: value));
                  ScaffoldMessenger.of(context).showSnackBar(
                    SnackBar(
                      content: Text('$label copied'),
                      backgroundColor: AppTheme.darkCard,
                      behavior: SnackBarBehavior.floating,
                      duration: const Duration(seconds: 1),
                      shape: RoundedRectangleBorder(
                        borderRadius: BorderRadius.circular(10),
                      ),
                    ),
                  );
                },
                child: const Icon(Icons.copy_rounded,
                    color: AppTheme.solanaPurple, size: 16),
              ),
            ],
          ],
        ),
      ],
    );
  }
}
