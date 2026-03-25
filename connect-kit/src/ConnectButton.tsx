/**
 * ConnectButton — Drop-in "Connect Wallet" button for SolClone DApps.
 *
 * States:
 *   - Disconnected: gradient purple button "Connect Wallet"
 *   - Connecting: spinner with "Connecting..."
 *   - Connected: truncated address + balance, dropdown menu
 *
 * Supports dark/light theme via the `theme` prop.
 */

import React, { useState, useRef, useEffect } from "react";
import { useSolCloneContext } from "./SolCloneProvider";

// ── Types ───────────────────────────────────────────────────────────────────

export interface ConnectButtonProps {
  /** Color theme. Default: "dark". */
  theme?: "dark" | "light";
  /** Custom class name for the outer container. */
  className?: string;
  /** Callback when "Connect Wallet" is clicked (to open the WalletModal). */
  onConnect?: () => void;
  /** Callback after disconnect completes. */
  onDisconnect?: () => void;
}

// ── Helpers ─────────────────────────────────────────────────────────────────

function truncateAddress(address: string): string {
  if (address.length <= 10) return address;
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
}

function formatBalance(lamports: number | null): string {
  if (lamports === null) return "";
  const sol = lamports / 1_000_000_000;
  return `${sol.toFixed(4)} SCLONE`;
}

// ── Styles ──────────────────────────────────────────────────────────────────

const baseButton: React.CSSProperties = {
  display: "inline-flex",
  alignItems: "center",
  gap: "8px",
  padding: "10px 20px",
  borderRadius: "12px",
  border: "none",
  cursor: "pointer",
  fontSize: "14px",
  fontWeight: 600,
  fontFamily: "system-ui, -apple-system, sans-serif",
  transition: "opacity 0.15s, transform 0.1s",
  position: "relative",
};

function getThemeStyles(theme: "dark" | "light") {
  if (theme === "dark") {
    return {
      button: {
        ...baseButton,
        background: "linear-gradient(135deg, #9B37FF, #6A0DAD)",
        color: "#FFFFFF",
      },
      connectedButton: {
        ...baseButton,
        background: "#1E1E2E",
        color: "#FFFFFF",
        border: "1px solid #333",
      },
      dropdown: {
        background: "#1E1E2E",
        border: "1px solid #333",
        color: "#FFFFFF",
      },
      dropdownItem: {
        color: "#FFFFFF",
        hoverBg: "#2A2A3E",
      },
    };
  }
  return {
    button: {
      ...baseButton,
      background: "linear-gradient(135deg, #9B37FF, #6A0DAD)",
      color: "#FFFFFF",
    },
    connectedButton: {
      ...baseButton,
      background: "#F5F5F5",
      color: "#1A1A1A",
      border: "1px solid #DDD",
    },
    dropdown: {
      background: "#FFFFFF",
      border: "1px solid #DDD",
      color: "#1A1A1A",
    },
    dropdownItem: {
      color: "#1A1A1A",
      hoverBg: "#F0F0F0",
    },
  };
}

const dropdownStyles: React.CSSProperties = {
  position: "absolute",
  top: "calc(100% + 8px)",
  right: 0,
  minWidth: "200px",
  borderRadius: "12px",
  padding: "4px",
  zIndex: 1000,
  boxShadow: "0 8px 24px rgba(0, 0, 0, 0.2)",
};

const dropdownItemStyles: React.CSSProperties = {
  display: "block",
  width: "100%",
  padding: "10px 14px",
  borderRadius: "8px",
  border: "none",
  cursor: "pointer",
  fontSize: "14px",
  fontFamily: "system-ui, -apple-system, sans-serif",
  textAlign: "left",
  background: "transparent",
  transition: "background 0.1s",
};

const spinnerKeyframes = `
@keyframes solclone-spin {
  to { transform: rotate(360deg); }
}
`;

// ── Component ───────────────────────────────────────────────────────────────

export function ConnectButton({
  theme = "dark",
  className,
  onConnect,
  onDisconnect,
}: ConnectButtonProps) {
  const { connected, connecting, address, balance, wallet, disconnect } =
    useSolCloneContext();
  const [dropdownOpen, setDropdownOpen] = useState(false);
  const [copied, setCopied] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const styles = getThemeStyles(theme);

  // Close dropdown when clicking outside.
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setDropdownOpen(false);
      }
    }

    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  // ── Disconnected State ──────────────────────────────────────────────

  if (!connected && !connecting) {
    return (
      <div className={className} style={{ display: "inline-block" }}>
        <button
          type="button"
          onClick={onConnect}
          style={styles.button}
          onMouseDown={(e) =>
            ((e.currentTarget as HTMLElement).style.transform = "scale(0.97)")
          }
          onMouseUp={(e) =>
            ((e.currentTarget as HTMLElement).style.transform = "scale(1)")
          }
        >
          Connect Wallet
        </button>
      </div>
    );
  }

  // ── Connecting State ────────────────────────────────────────────────

  if (connecting) {
    return (
      <div className={className} style={{ display: "inline-block" }}>
        <style>{spinnerKeyframes}</style>
        <button type="button" disabled style={{ ...styles.button, opacity: 0.8 }}>
          <span
            style={{
              display: "inline-block",
              width: "16px",
              height: "16px",
              border: "2px solid rgba(255,255,255,0.3)",
              borderTopColor: "#FFFFFF",
              borderRadius: "50%",
              animation: "solclone-spin 0.6s linear infinite",
            }}
          />
          Connecting...
        </button>
      </div>
    );
  }

  // ── Connected State ─────────────────────────────────────────────────

  const handleCopyAddress = async () => {
    if (address) {
      await navigator.clipboard.writeText(address);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  const handleDisconnect = async () => {
    setDropdownOpen(false);
    await disconnect();
    onDisconnect?.();
  };

  const handleChangeWallet = () => {
    setDropdownOpen(false);
    onConnect?.();
  };

  return (
    <div
      ref={containerRef}
      className={className}
      style={{ display: "inline-block", position: "relative" }}
    >
      <button
        type="button"
        onClick={() => setDropdownOpen(!dropdownOpen)}
        style={styles.connectedButton}
      >
        {wallet?.icon && (
          <img
            src={wallet.icon}
            alt={wallet.name}
            width={20}
            height={20}
            style={{ borderRadius: "4px" }}
          />
        )}
        <span>
          {address ? truncateAddress(address) : "Connected"}
          {balance !== null && (
            <span style={{ opacity: 0.7, marginLeft: "6px", fontSize: "12px" }}>
              {formatBalance(balance)}
            </span>
          )}
        </span>
        <span
          style={{
            fontSize: "10px",
            opacity: 0.5,
            transform: dropdownOpen ? "rotate(180deg)" : "rotate(0deg)",
            transition: "transform 0.15s",
          }}
        >
          &#9660;
        </span>
      </button>

      {dropdownOpen && (
        <div style={{ ...dropdownStyles, ...styles.dropdown }}>
          <button
            type="button"
            onClick={handleCopyAddress}
            style={{
              ...dropdownItemStyles,
              color: styles.dropdownItem.color,
            }}
            onMouseEnter={(e) =>
              ((e.currentTarget as HTMLElement).style.background =
                styles.dropdownItem.hoverBg)
            }
            onMouseLeave={(e) =>
              ((e.currentTarget as HTMLElement).style.background = "transparent")
            }
          >
            {copied ? "Copied!" : "Copy Address"}
          </button>
          <button
            type="button"
            onClick={handleChangeWallet}
            style={{
              ...dropdownItemStyles,
              color: styles.dropdownItem.color,
            }}
            onMouseEnter={(e) =>
              ((e.currentTarget as HTMLElement).style.background =
                styles.dropdownItem.hoverBg)
            }
            onMouseLeave={(e) =>
              ((e.currentTarget as HTMLElement).style.background = "transparent")
            }
          >
            Change Wallet
          </button>
          <div
            style={{
              height: "1px",
              background: theme === "dark" ? "#333" : "#DDD",
              margin: "4px 8px",
            }}
          />
          <button
            type="button"
            onClick={handleDisconnect}
            style={{
              ...dropdownItemStyles,
              color: "#E53E3E",
            }}
            onMouseEnter={(e) =>
              ((e.currentTarget as HTMLElement).style.background =
                styles.dropdownItem.hoverBg)
            }
            onMouseLeave={(e) =>
              ((e.currentTarget as HTMLElement).style.background = "transparent")
            }
          >
            Disconnect
          </button>
        </div>
      )}
    </div>
  );
}
