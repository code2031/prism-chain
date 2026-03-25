/**
 * WalletModal — Wallet selection modal for SolClone DApps.
 *
 * Features:
 *   - Lists all detected Wallet Standard wallets with icons
 *   - "WalletConnect" option with QR code display
 *   - "Install Wallet" links for popular wallets not detected
 *   - Search/filter wallets by name
 *   - Recently connected wallet pinned at top
 */

import React, { useState, useEffect, useCallback, useRef } from "react";
import { detectWallets, type DetectedWallet } from "@solclone/wallet-standard";
import { useSolCloneContext } from "./SolCloneProvider";

// ── Types ───────────────────────────────────────────────────────────────────

export interface WalletModalProps {
  /** Whether the modal is visible. */
  open: boolean;
  /** Callback to close the modal. */
  onClose: () => void;
  /** Color theme. Default: "dark". */
  theme?: "dark" | "light";
  /** WalletConnect project ID (enables QR code option). */
  walletConnectProjectId?: string;
}

interface PopularWallet {
  name: string;
  installUrl: string;
  icon: string;
}

// ── Constants ───────────────────────────────────────────────────────────────

const POPULAR_WALLETS: PopularWallet[] = [
  {
    name: "Phantom",
    installUrl: "https://phantom.app/download",
    icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI4IiBmaWxsPSIjQUI5RkY2Ii8+PHRleHQgeD0iMTYiIHk9IjIxIiBmb250LWZhbWlseT0iQXJpYWwsc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxOCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IndoaXRlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5QPC90ZXh0Pjwvc3ZnPg==",
  },
  {
    name: "Solflare",
    installUrl: "https://solflare.com/download",
    icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI4IiBmaWxsPSIjRkM4OTFFIi8+PHRleHQgeD0iMTYiIHk9IjIxIiBmb250LWZhbWlseT0iQXJpYWwsc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxOCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IndoaXRlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5TPC90ZXh0Pjwvc3ZnPg==",
  },
  {
    name: "Backpack",
    installUrl: "https://backpack.app/download",
    icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI4IiBmaWxsPSIjRTMzRTNCIi8+PHRleHQgeD0iMTYiIHk9IjIxIiBmb250LWZhbWlseT0iQXJpYWwsc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxOCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IndoaXRlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5CPC90ZXh0Pjwvc3ZnPg==",
  },
  {
    name: "Glow",
    installUrl: "https://glow.app/download",
    icon: "data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI4IiBmaWxsPSIjN0VEOTVFIi8+PHRleHQgeD0iMTYiIHk9IjIxIiBmb250LWZhbWlseT0iQXJpYWwsc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxOCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IndoaXRlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5HPC90ZXh0Pjwvc3ZnPg==",
  },
];

const RECENT_WALLET_KEY = "solclone:recent-wallet";

// ── Styles ──────────────────────────────────────────────────────────────────

function getModalStyles(theme: "dark" | "light") {
  const isDark = theme === "dark";
  return {
    overlay: {
      position: "fixed" as const,
      inset: 0,
      background: "rgba(0, 0, 0, 0.6)",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      zIndex: 10000,
      backdropFilter: "blur(4px)",
    },
    modal: {
      background: isDark ? "#1E1E2E" : "#FFFFFF",
      borderRadius: "16px",
      width: "400px",
      maxWidth: "90vw",
      maxHeight: "80vh",
      overflow: "hidden",
      display: "flex",
      flexDirection: "column" as const,
      boxShadow: "0 16px 48px rgba(0, 0, 0, 0.3)",
    },
    header: {
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "20px 24px 12px",
      borderBottom: `1px solid ${isDark ? "#333" : "#EEE"}`,
    },
    title: {
      fontSize: "18px",
      fontWeight: 700 as const,
      color: isDark ? "#FFFFFF" : "#1A1A1A",
      fontFamily: "system-ui, -apple-system, sans-serif",
      margin: 0,
    },
    closeButton: {
      background: "none",
      border: "none",
      fontSize: "20px",
      cursor: "pointer",
      color: isDark ? "#888" : "#666",
      padding: "4px 8px",
      borderRadius: "6px",
    },
    searchContainer: {
      padding: "12px 24px",
    },
    searchInput: {
      width: "100%",
      padding: "10px 14px",
      borderRadius: "10px",
      border: `1px solid ${isDark ? "#444" : "#DDD"}`,
      background: isDark ? "#2A2A3E" : "#F5F5F5",
      color: isDark ? "#FFFFFF" : "#1A1A1A",
      fontSize: "14px",
      fontFamily: "system-ui, -apple-system, sans-serif",
      outline: "none",
      boxSizing: "border-box" as const,
    },
    list: {
      flex: 1,
      overflowY: "auto" as const,
      padding: "8px 16px 16px",
    },
    sectionLabel: {
      fontSize: "11px",
      fontWeight: 600 as const,
      textTransform: "uppercase" as const,
      color: isDark ? "#888" : "#999",
      padding: "8px 8px 4px",
      letterSpacing: "0.5px",
      fontFamily: "system-ui, -apple-system, sans-serif",
    },
    walletItem: {
      display: "flex",
      alignItems: "center",
      gap: "12px",
      width: "100%",
      padding: "12px",
      borderRadius: "10px",
      border: "none",
      cursor: "pointer",
      background: "transparent",
      transition: "background 0.1s",
      fontFamily: "system-ui, -apple-system, sans-serif",
    },
    walletItemHover: {
      background: isDark ? "#2A2A3E" : "#F0F0F0",
    },
    walletIcon: {
      width: "36px",
      height: "36px",
      borderRadius: "8px",
      objectFit: "cover" as const,
    },
    walletName: {
      fontSize: "14px",
      fontWeight: 600 as const,
      color: isDark ? "#FFFFFF" : "#1A1A1A",
    },
    walletDetail: {
      fontSize: "12px",
      color: isDark ? "#888" : "#999",
    },
    installBadge: {
      marginLeft: "auto",
      fontSize: "11px",
      padding: "3px 8px",
      borderRadius: "6px",
      background: isDark ? "#333" : "#EEE",
      color: isDark ? "#AAA" : "#666",
    },
    qrContainer: {
      display: "flex",
      flexDirection: "column" as const,
      alignItems: "center",
      padding: "20px",
      gap: "12px",
    },
    qrLabel: {
      fontSize: "13px",
      color: isDark ? "#888" : "#999",
      textAlign: "center" as const,
      fontFamily: "system-ui, -apple-system, sans-serif",
    },
  };
}

// ── Component ───────────────────────────────────────────────────────────────

export function WalletModal({
  open,
  onClose,
  theme = "dark",
  walletConnectProjectId,
}: WalletModalProps) {
  const { connect: connectWallet } = useSolCloneContext();
  const [search, setSearch] = useState("");
  const [wallets, setWallets] = useState<DetectedWallet[]>([]);
  const [recentWalletName, setRecentWalletName] = useState<string | null>(null);
  const [showQR, setShowQR] = useState(false);
  const [qrDataUri, setQrDataUri] = useState<string | null>(null);
  const [connectingName, setConnectingName] = useState<string | null>(null);
  const modalRef = useRef<HTMLDivElement>(null);

  const styles = getModalStyles(theme);

  // ── Load Wallets & Recent ───────────────────────────────────────────

  useEffect(() => {
    if (!open) return;

    const detected = detectWallets();
    setWallets(detected);

    try {
      const recent = localStorage.getItem(RECENT_WALLET_KEY);
      setRecentWalletName(recent);
    } catch {
      // localStorage may not be available.
    }
  }, [open]);

  // ── Close on Escape ─────────────────────────────────────────────────

  useEffect(() => {
    if (!open) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [open, onClose]);

  // ── Connect to Wallet ───────────────────────────────────────────────

  const handleSelectWallet = useCallback(
    async (wallet: DetectedWallet) => {
      setConnectingName(wallet.name);

      try {
        await connectWallet(wallet);

        // Save as recent.
        try {
          localStorage.setItem(RECENT_WALLET_KEY, wallet.name);
        } catch {
          // Ignore storage errors.
        }

        onClose();
      } catch (error) {
        console.error("Failed to connect wallet:", error);
      } finally {
        setConnectingName(null);
      }
    },
    [connectWallet, onClose],
  );

  // ── WalletConnect QR ────────────────────────────────────────────────

  const handleWalletConnect = useCallback(async () => {
    if (!walletConnectProjectId) return;

    setShowQR(true);

    try {
      const { SolCloneWalletConnectClient, WalletConnectQRModal } =
        await import("@solclone/wallet-connect");

      const client = new SolCloneWalletConnectClient({
        projectId: walletConnectProjectId,
      });

      await client.init();
      const { uri, approval } = await client.connect();

      const qrModal = new WalletConnectQRModal();
      const qr = qrModal.generateQR(uri);
      setQrDataUri(qr.dataUri);

      // Wait for approval in background.
      approval.then(() => {
        onClose();
      }).catch(() => {
        setShowQR(false);
        setQrDataUri(null);
      });
    } catch (error) {
      console.error("WalletConnect error:", error);
      setShowQR(false);
      setQrDataUri(null);
    }
  }, [walletConnectProjectId, onClose]);

  // ── Filter Wallets ──────────────────────────────────────────────────

  const filteredWallets = wallets.filter((w) =>
    w.name.toLowerCase().includes(search.toLowerCase()),
  );

  // Separate recent wallet.
  const recentWallet = recentWalletName
    ? filteredWallets.find((w) => w.name === recentWalletName)
    : null;
  const otherWallets = filteredWallets.filter(
    (w) => w.name !== recentWalletName,
  );

  // Find popular wallets that are not installed (for "Install" links).
  const installedNames = new Set(wallets.map((w) => w.name.toLowerCase()));
  const notInstalled = POPULAR_WALLETS.filter(
    (pw) => !installedNames.has(pw.name.toLowerCase()),
  );
  const filteredNotInstalled = notInstalled.filter((pw) =>
    pw.name.toLowerCase().includes(search.toLowerCase()),
  );

  if (!open) return null;

  // ── Render ──────────────────────────────────────────────────────────

  return (
    <div
      style={styles.overlay}
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
      role="dialog"
      aria-modal="true"
      aria-label="Connect a wallet"
    >
      <div ref={modalRef} style={styles.modal}>
        {/* Header */}
        <div style={styles.header}>
          <h2 style={styles.title}>
            {showQR ? "Scan QR Code" : "Connect Wallet"}
          </h2>
          <button
            type="button"
            onClick={() => {
              if (showQR) {
                setShowQR(false);
                setQrDataUri(null);
              } else {
                onClose();
              }
            }}
            style={styles.closeButton}
            aria-label={showQR ? "Back" : "Close"}
          >
            {showQR ? "\u2190" : "\u2715"}
          </button>
        </div>

        {/* QR Code View */}
        {showQR ? (
          <div style={styles.qrContainer}>
            {qrDataUri ? (
              <img
                src={qrDataUri}
                alt="WalletConnect QR Code"
                width={280}
                height={280}
                style={{ borderRadius: "12px" }}
              />
            ) : (
              <div
                style={{
                  width: "280px",
                  height: "280px",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "center",
                  color: theme === "dark" ? "#888" : "#999",
                }}
              >
                Generating QR code...
              </div>
            )}
            <p style={styles.qrLabel}>
              Scan with your mobile wallet to connect
            </p>
          </div>
        ) : (
          <>
            {/* Search */}
            <div style={styles.searchContainer}>
              <input
                type="text"
                placeholder="Search wallets..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                style={styles.searchInput}
                autoFocus
              />
            </div>

            {/* Wallet List */}
            <div style={styles.list}>
              {/* Recently Connected */}
              {recentWallet && (
                <>
                  <div style={styles.sectionLabel}>Recently Connected</div>
                  <WalletItem
                    name={recentWallet.name}
                    icon={
                      typeof recentWallet.icon === "string"
                        ? recentWallet.icon
                        : undefined
                    }
                    detail="Last used"
                    isConnecting={connectingName === recentWallet.name}
                    onClick={() => handleSelectWallet(recentWallet)}
                    styles={styles}
                    theme={theme}
                  />
                </>
              )}

              {/* Detected Wallets */}
              {otherWallets.length > 0 && (
                <>
                  <div style={styles.sectionLabel}>Available Wallets</div>
                  {otherWallets.map((w) => (
                    <WalletItem
                      key={w.name}
                      name={w.name}
                      icon={
                        typeof w.icon === "string" ? w.icon : undefined
                      }
                      detail={`${w.chains.length} chains`}
                      isConnecting={connectingName === w.name}
                      onClick={() => handleSelectWallet(w)}
                      styles={styles}
                      theme={theme}
                    />
                  ))}
                </>
              )}

              {/* WalletConnect Option */}
              {walletConnectProjectId && (
                <>
                  <div style={styles.sectionLabel}>Mobile Wallet</div>
                  <WalletItem
                    name="WalletConnect"
                    icon="data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHJ4PSI4IiBmaWxsPSIjMzM5NkZGIi8+PHRleHQgeD0iMTYiIHk9IjIxIiBmb250LWZhbWlseT0iQXJpYWwsc2Fucy1zZXJpZiIgZm9udC1zaXplPSIxNCIgZm9udC13ZWlnaHQ9ImJvbGQiIGZpbGw9IndoaXRlIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIj5XQzwvdGV4dD48L3N2Zz4="
                    detail="Scan with mobile wallet"
                    isConnecting={false}
                    onClick={handleWalletConnect}
                    styles={styles}
                    theme={theme}
                  />
                </>
              )}

              {/* Not Installed */}
              {filteredNotInstalled.length > 0 && (
                <>
                  <div style={styles.sectionLabel}>Install a Wallet</div>
                  {filteredNotInstalled.map((pw) => (
                    <a
                      key={pw.name}
                      href={pw.installUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      style={{ textDecoration: "none" }}
                    >
                      <WalletItem
                        name={pw.name}
                        icon={pw.icon}
                        detail="Not installed"
                        isConnecting={false}
                        onClick={() => {}}
                        styles={styles}
                        theme={theme}
                        badge="Install"
                      />
                    </a>
                  ))}
                </>
              )}

              {/* Empty State */}
              {filteredWallets.length === 0 &&
                filteredNotInstalled.length === 0 && (
                  <div
                    style={{
                      textAlign: "center",
                      padding: "24px",
                      color: theme === "dark" ? "#888" : "#999",
                      fontSize: "14px",
                    }}
                  >
                    No wallets found matching &quot;{search}&quot;
                  </div>
                )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

// ── Wallet Item Sub-Component ───────────────────────────────────────────────

function WalletItem({
  name,
  icon,
  detail,
  isConnecting,
  onClick,
  styles,
  theme,
  badge,
}: {
  name: string;
  icon?: string;
  detail: string;
  isConnecting: boolean;
  onClick: () => void;
  styles: ReturnType<typeof getModalStyles>;
  theme: "dark" | "light";
  badge?: string;
}) {
  const [hovered, setHovered] = useState(false);

  return (
    <button
      type="button"
      onClick={onClick}
      disabled={isConnecting}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        ...styles.walletItem,
        ...(hovered ? styles.walletItemHover : {}),
        opacity: isConnecting ? 0.6 : 1,
      }}
    >
      {icon ? (
        <img src={icon} alt={name} style={styles.walletIcon} />
      ) : (
        <div
          style={{
            ...styles.walletIcon,
            background: theme === "dark" ? "#333" : "#DDD",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            fontSize: "16px",
            fontWeight: 700,
            color: theme === "dark" ? "#AAA" : "#666",
          }}
        >
          {name.charAt(0)}
        </div>
      )}
      <div style={{ textAlign: "left" }}>
        <div style={styles.walletName}>
          {name}
          {isConnecting && " ..."}
        </div>
        <div style={styles.walletDetail}>{detail}</div>
      </div>
      {badge && <span style={styles.installBadge}>{badge}</span>}
    </button>
  );
}
