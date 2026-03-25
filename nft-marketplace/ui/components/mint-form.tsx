"use client";

import { useCallback, useState } from "react";

interface Attribute {
  key: string;
  value: string;
}

const COLLECTIONS = [
  { id: "none", name: "No Collection" },
  { id: "sol-apes", name: "SolApes" },
  { id: "pixel-punks", name: "Pixel Punks" },
  { id: "cosmic-cats", name: "Cosmic Cats" },
  { id: "neon-worlds", name: "Neon Worlds" },
];

export default function MintForm() {
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [attributes, setAttributes] = useState<Attribute[]>([
    { key: "", value: "" },
  ]);
  const [royaltyPercent, setRoyaltyPercent] = useState("5");
  const [collection, setCollection] = useState("none");
  const [dragActive, setDragActive] = useState(false);
  const [imagePreview, setImagePreview] = useState<string | null>(null);
  const [fileName, setFileName] = useState<string | null>(null);

  const handleDrag = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      if (e.type === "dragenter" || e.type === "dragover") {
        setDragActive(true);
      } else if (e.type === "dragleave") {
        setDragActive(false);
      }
    },
    [],
  );

  const processFile = useCallback((file: File) => {
    setFileName(file.name);
    const reader = new FileReader();
    reader.onload = (e) => {
      setImagePreview(e.target?.result as string);
    };
    reader.readAsDataURL(file);
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragActive(false);
      if (e.dataTransfer.files?.[0]) {
        processFile(e.dataTransfer.files[0]);
      }
    },
    [processFile],
  );

  const handleFileChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (e.target.files?.[0]) {
        processFile(e.target.files[0]);
      }
    },
    [processFile],
  );

  const addAttribute = () => {
    setAttributes((prev) => [...prev, { key: "", value: "" }]);
  };

  const removeAttribute = (index: number) => {
    setAttributes((prev) => prev.filter((_, i) => i !== index));
  };

  const updateAttribute = (
    index: number,
    field: "key" | "value",
    val: string,
  ) => {
    setAttributes((prev) =>
      prev.map((attr, i) => (i === index ? { ...attr, [field]: val } : attr)),
    );
  };

  const handleMint = (e: React.FormEvent) => {
    e.preventDefault();
    // In production this would call the on-chain program
    alert(
      `Minting "${name}" with ${attributes.filter((a) => a.key).length} attributes, ${royaltyPercent}% royalty`,
    );
  };

  const inputClasses =
    "w-full rounded-xl border border-input-border bg-input-bg px-4 py-3 text-sm text-foreground placeholder:text-muted/60 transition-colors focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple/50";

  return (
    <form onSubmit={handleMint} className="space-y-8 animate-fade-in">
      {/* ── Upload zone ── */}
      <div>
        <label className="mb-2 block text-sm font-semibold text-foreground">
          Image
        </label>
        <div
          onDragEnter={handleDrag}
          onDragLeave={handleDrag}
          onDragOver={handleDrag}
          onDrop={handleDrop}
          className={`relative flex min-h-[240px] cursor-pointer flex-col items-center justify-center rounded-2xl border-2 border-dashed transition-colors ${
            dragActive
              ? "border-accent-purple bg-accent-purple/10"
              : "border-card-border bg-surface/50 hover:border-accent-purple/50"
          }`}
        >
          <input
            type="file"
            accept="image/*"
            onChange={handleFileChange}
            className="absolute inset-0 cursor-pointer opacity-0"
          />

          {imagePreview ? (
            <div className="relative flex flex-col items-center gap-3 p-4">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img
                src={imagePreview}
                alt="Preview"
                className="h-40 w-40 rounded-xl object-cover"
              />
              <p className="text-xs text-muted">{fileName}</p>
              <p className="text-xs text-accent-purple">
                Click or drag to replace
              </p>
            </div>
          ) : (
            <div className="flex flex-col items-center gap-3 p-8">
              <div className="flex h-14 w-14 items-center justify-center rounded-xl bg-accent-purple/10">
                <svg
                  width="28"
                  height="28"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                  className="text-accent-purple"
                >
                  <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                  <polyline points="17 8 12 3 7 8" />
                  <line x1="12" y1="3" x2="12" y2="15" />
                </svg>
              </div>
              <div className="text-center">
                <p className="text-sm font-medium text-foreground">
                  Drag and drop or{" "}
                  <span className="text-accent-purple">browse</span>
                </p>
                <p className="mt-1 text-xs text-muted">
                  PNG, JPG, GIF, WEBP. Max 10MB.
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* ── Name ── */}
      <div>
        <label
          htmlFor="nft-name"
          className="mb-2 block text-sm font-semibold text-foreground"
        >
          Name
        </label>
        <input
          id="nft-name"
          type="text"
          placeholder="e.g. Cosmic Voyager #42"
          value={name}
          onChange={(e) => setName(e.target.value)}
          className={inputClasses}
          required
        />
      </div>

      {/* ── Description ── */}
      <div>
        <label
          htmlFor="nft-desc"
          className="mb-2 block text-sm font-semibold text-foreground"
        >
          Description
        </label>
        <textarea
          id="nft-desc"
          placeholder="Tell the story of your NFT..."
          rows={4}
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          className={inputClasses + " resize-none"}
        />
      </div>

      {/* ── Attributes ── */}
      <div>
        <div className="mb-3 flex items-center justify-between">
          <label className="text-sm font-semibold text-foreground">
            Attributes
          </label>
          <button
            type="button"
            onClick={addAttribute}
            className="text-xs font-medium text-accent-purple transition-colors hover:text-accent-purple/80"
          >
            + Add Attribute
          </button>
        </div>
        <div className="space-y-3">
          {attributes.map((attr, i) => (
            <div key={i} className="flex items-center gap-3">
              <input
                type="text"
                placeholder="Trait (e.g. Background)"
                value={attr.key}
                onChange={(e) => updateAttribute(i, "key", e.target.value)}
                className={inputClasses}
              />
              <input
                type="text"
                placeholder="Value (e.g. Cosmic Blue)"
                value={attr.value}
                onChange={(e) => updateAttribute(i, "value", e.target.value)}
                className={inputClasses}
              />
              {attributes.length > 1 && (
                <button
                  type="button"
                  onClick={() => removeAttribute(i)}
                  className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg border border-card-border text-muted transition-colors hover:border-red-500/50 hover:text-red-400"
                >
                  <svg
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                  >
                    <line x1="18" y1="6" x2="6" y2="18" />
                    <line x1="6" y1="6" x2="18" y2="18" />
                  </svg>
                </button>
              )}
            </div>
          ))}
        </div>
      </div>

      {/* ── Royalty + Collection (row) ── */}
      <div className="grid gap-6 sm:grid-cols-2">
        <div>
          <label
            htmlFor="royalty"
            className="mb-2 block text-sm font-semibold text-foreground"
          >
            Royalty %
          </label>
          <input
            id="royalty"
            type="number"
            min="0"
            max="50"
            step="0.5"
            value={royaltyPercent}
            onChange={(e) => setRoyaltyPercent(e.target.value)}
            className={inputClasses}
          />
          <p className="mt-1 text-xs text-muted">
            Earned on every secondary sale (max 50%)
          </p>
        </div>

        <div>
          <label
            htmlFor="collection"
            className="mb-2 block text-sm font-semibold text-foreground"
          >
            Collection
          </label>
          <select
            id="collection"
            value={collection}
            onChange={(e) => setCollection(e.target.value)}
            className={inputClasses + " appearance-none"}
          >
            {COLLECTIONS.map((c) => (
              <option key={c.id} value={c.id}>
                {c.name}
              </option>
            ))}
          </select>
        </div>
      </div>

      {/* ── Submit ── */}
      <button
        type="submit"
        className="w-full rounded-xl bg-gradient-to-r from-accent-purple to-accent-green py-4 text-base font-semibold text-white transition-opacity hover:opacity-90 disabled:opacity-50"
        disabled={!name}
      >
        Mint NFT
      </button>
    </form>
  );
}
