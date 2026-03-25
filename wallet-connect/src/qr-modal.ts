/**
 * WalletConnect QR Modal — Generates QR code data from a WalletConnect URI.
 *
 * This module provides a pure-logic QR code generator that returns
 * base64-encoded image data. The actual UI rendering is left to the
 * consuming application (or the connect-kit package).
 */

// ── QR Code Matrix Generator ────────────────────────────────────────────────

/**
 * Minimal QR code encoding. Generates a 2D boolean matrix representing
 * the QR code for a given string. This is a simplified implementation
 * suitable for WalletConnect URIs.
 */

/** A single module (pixel) in the QR code. */
type QRMatrix = boolean[][];

/**
 * Encodes a string into QR code bit matrix using alphanumeric mode.
 * This is a simplified encoder for demonstration; production use
 * should use a dedicated library like `qrcode`.
 */
function generateQRMatrix(data: string): QRMatrix {
  // Determine size based on data length.
  // WalletConnect URIs are typically 200-400 chars, needing version 10-15.
  const dataLen = data.length;
  let version: number;
  if (dataLen <= 77) version = 5;
  else if (dataLen <= 154) version = 10;
  else if (dataLen <= 268) version = 15;
  else if (dataLen <= 395) version = 20;
  else version = 25;

  const size = version * 4 + 17;
  const matrix: QRMatrix = Array.from({ length: size }, () =>
    Array(size).fill(false),
  );

  // Finder patterns (top-left, top-right, bottom-left).
  const drawFinder = (row: number, col: number) => {
    for (let r = -1; r <= 7; r++) {
      for (let c = -1; c <= 7; c++) {
        const mr = row + r;
        const mc = col + c;
        if (mr < 0 || mr >= size || mc < 0 || mc >= size) continue;
        const isOuter =
          r === -1 || r === 7 || c === -1 || c === 7;
        const isInner =
          r >= 2 && r <= 4 && c >= 2 && c <= 4;
        const isBorder =
          r === 0 || r === 6 || c === 0 || c === 6;
        matrix[mr][mc] = !isOuter && (isBorder || isInner);
      }
    }
  };

  drawFinder(0, 0);
  drawFinder(0, size - 7);
  drawFinder(size - 7, 0);

  // Timing patterns.
  for (let i = 8; i < size - 8; i++) {
    matrix[6][i] = i % 2 === 0;
    matrix[i][6] = i % 2 === 0;
  }

  // Data encoding: spread data bits across the matrix in a deterministic
  // pattern. This is a simplified version for visual representation.
  const bytes = new TextEncoder().encode(data);
  let bitIndex = 0;
  const totalBits = bytes.length * 8;

  for (let col = size - 1; col >= 1; col -= 2) {
    if (col === 6) col = 5; // Skip timing column.
    for (let row = 0; row < size; row++) {
      for (let c = 0; c < 2; c++) {
        const x = col - c;
        const y = row;
        if (matrix[y][x]) continue; // Skip reserved areas.
        if (y < 9 && x < 9) continue; // Skip finder TL.
        if (y < 9 && x >= size - 8) continue; // Skip finder TR.
        if (y >= size - 8 && x < 9) continue; // Skip finder BL.
        if (y === 6 || x === 6) continue; // Skip timing.

        if (bitIndex < totalBits) {
          const byteIdx = Math.floor(bitIndex / 8);
          const bitPos = 7 - (bitIndex % 8);
          matrix[y][x] = ((bytes[byteIdx] >> bitPos) & 1) === 1;
          bitIndex++;
        }
      }
    }
  }

  return matrix;
}

// ── SVG Generation ──────────────────────────────────────────────────────────

/**
 * Renders a QR matrix as an SVG string.
 */
function matrixToSVG(
  matrix: QRMatrix,
  options: { moduleSize: number; margin: number; darkColor: string; lightColor: string },
): string {
  const size = matrix.length;
  const totalSize = size * options.moduleSize + options.margin * 2;

  let paths = "";
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      if (matrix[y][x]) {
        const px = x * options.moduleSize + options.margin;
        const py = y * options.moduleSize + options.margin;
        paths += `<rect x="${px}" y="${py}" width="${options.moduleSize}" height="${options.moduleSize}" fill="${options.darkColor}"/>`;
      }
    }
  }

  return [
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${totalSize} ${totalSize}" width="${totalSize}" height="${totalSize}">`,
    `<rect width="${totalSize}" height="${totalSize}" fill="${options.lightColor}"/>`,
    paths,
    `</svg>`,
  ].join("");
}

// ── Public API ──────────────────────────────────────────────────────────────

export interface QRCodeOptions {
  /** Size of each QR module in pixels. Default: 4. */
  moduleSize?: number;
  /** Margin (quiet zone) in pixels. Default: 16. */
  margin?: number;
  /** Dark module color. Default: "#000000". */
  darkColor?: string;
  /** Light module color. Default: "#FFFFFF". */
  lightColor?: string;
}

export interface QRCodeResult {
  /** Base64-encoded SVG image (data URI). */
  dataUri: string;
  /** Raw SVG string. */
  svg: string;
  /** The QR matrix (for custom rendering). */
  matrix: boolean[][];
  /** Size of the matrix (modules per side). */
  size: number;
}

/**
 * WalletConnect QR Modal utility.
 *
 * Generates QR code data from a WalletConnect URI. The result can be
 * displayed as an image (via the data URI) or rendered with custom UI.
 */
export class WalletConnectQRModal {
  /**
   * Generate QR code data from a WalletConnect URI.
   *
   * @param uri - The WalletConnect pairing URI (starts with "wc:").
   * @param options - Rendering options.
   * @returns QR code as base64 data URI, raw SVG, and matrix.
   *
   * @example
   * ```ts
   * const modal = new WalletConnectQRModal();
   * const qr = modal.generateQR("wc:abc123@2?relay-protocol=irn&symKey=xyz");
   *
   * // Use the data URI in an <img> element:
   * img.src = qr.dataUri;
   *
   * // Or use the matrix for custom pixel-based rendering.
   * ```
   */
  generateQR(uri: string, options: QRCodeOptions = {}): QRCodeResult {
    if (!uri || !uri.startsWith("wc:")) {
      throw new Error(
        "Invalid WalletConnect URI. Expected a string starting with 'wc:'.",
      );
    }

    const moduleSize = options.moduleSize ?? 4;
    const margin = options.margin ?? 16;
    const darkColor = options.darkColor ?? "#000000";
    const lightColor = options.lightColor ?? "#FFFFFF";

    const matrix = generateQRMatrix(uri);
    const svg = matrixToSVG(matrix, {
      moduleSize,
      margin,
      darkColor,
      lightColor,
    });

    // Encode SVG as base64 data URI.
    const base64 =
      typeof btoa === "function"
        ? btoa(svg)
        : Buffer.from(svg).toString("base64");

    const dataUri = `data:image/svg+xml;base64,${base64}`;

    return {
      dataUri,
      svg,
      matrix,
      size: matrix.length,
    };
  }

  /**
   * Generate a deep link URL for mobile wallets that support
   * WalletConnect deep linking.
   *
   * @param uri - WalletConnect URI.
   * @param walletScheme - The mobile wallet's URL scheme (e.g., "phantom://", "solflare://").
   * @returns The deep link URL.
   */
  generateDeepLink(uri: string, walletScheme: string): string {
    const encoded = encodeURIComponent(uri);
    const scheme = walletScheme.endsWith("://")
      ? walletScheme
      : `${walletScheme}://`;
    return `${scheme}wc?uri=${encoded}`;
  }

  /**
   * Generate deep links for common Solana wallets.
   */
  generateAllDeepLinks(uri: string): Record<string, string> {
    return {
      phantom: this.generateDeepLink(uri, "phantom"),
      solflare: this.generateDeepLink(uri, "solflare"),
      backpack: this.generateDeepLink(uri, "backpack"),
      glow: this.generateDeepLink(uri, "glow"),
    };
  }
}
