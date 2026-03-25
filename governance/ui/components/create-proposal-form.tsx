"use client";

import { useState } from "react";

type ActionType = "treasury_transfer" | "parameter_change" | "program_upgrade";

interface CreateProposalFormProps {
  onSubmit?: (proposal: {
    title: string;
    description: string;
    actionType: ActionType;
    parameters: Record<string, string>;
  }) => void;
  onCancel?: () => void;
}

const ACTION_TYPES: { value: ActionType; label: string; description: string }[] = [
  {
    value: "treasury_transfer",
    label: "Treasury Transfer",
    description: "Transfer tokens from the DAO treasury to a recipient",
  },
  {
    value: "parameter_change",
    label: "Parameter Change",
    description: "Modify a DAO governance parameter",
  },
  {
    value: "program_upgrade",
    label: "Program Upgrade",
    description: "Upgrade a deployed program to a new version",
  },
];

const PARAMETER_FIELDS: Record<ActionType, { key: string; label: string; placeholder: string }[]> = {
  treasury_transfer: [
    { key: "recipient", label: "Recipient Address", placeholder: "Enter wallet address..." },
    { key: "amount", label: "Amount (SCLONE)", placeholder: "e.g. 50000" },
    { key: "memo", label: "Memo", placeholder: "Purpose of transfer..." },
  ],
  parameter_change: [
    { key: "parameter", label: "Parameter Name", placeholder: "e.g. voting_period, quorum_bps" },
    { key: "current_value", label: "Current Value", placeholder: "Current setting" },
    { key: "new_value", label: "New Value", placeholder: "Proposed new value" },
  ],
  program_upgrade: [
    { key: "program_id", label: "Program ID", placeholder: "Program address to upgrade..." },
    { key: "buffer_address", label: "Buffer Address", placeholder: "Upgrade buffer address..." },
    { key: "authority", label: "Upgrade Authority", placeholder: "Authority address..." },
  ],
};

export default function CreateProposalForm({
  onSubmit,
  onCancel,
}: CreateProposalFormProps) {
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [actionType, setActionType] = useState<ActionType>("treasury_transfer");
  const [parameters, setParameters] = useState<Record<string, string>>({});

  const fields = PARAMETER_FIELDS[actionType];

  function handleParameterChange(key: string, value: string) {
    setParameters((prev) => ({ ...prev, [key]: value }));
  }

  function handleActionTypeChange(newType: ActionType) {
    setActionType(newType);
    setParameters({});
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    onSubmit?.({ title, description, actionType, parameters });
  }

  const isValid = title.trim().length > 0 && description.trim().length > 0;

  return (
    <form onSubmit={handleSubmit} className="mx-auto max-w-2xl">
      <div className="mb-6">
        <h2 className="text-xl font-bold text-foreground">Create Proposal</h2>
        <p className="mt-1 text-sm text-muted">
          Requires at least 100,000 SCLONE tokens to submit.
        </p>
      </div>

      <div className="space-y-5">
        {/* Title */}
        <div>
          <label
            htmlFor="proposal-title"
            className="mb-1.5 block text-sm font-medium text-foreground"
          >
            Title
          </label>
          <input
            id="proposal-title"
            type="text"
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="A clear, concise proposal title..."
            maxLength={128}
            className="w-full rounded-lg border border-card-border bg-background px-4 py-2.5 text-sm text-foreground placeholder:text-muted focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple"
          />
          <p className="mt-1 text-right text-xs text-muted">
            {title.length}/128
          </p>
        </div>

        {/* Description */}
        <div>
          <label
            htmlFor="proposal-description"
            className="mb-1.5 block text-sm font-medium text-foreground"
          >
            Description
          </label>
          <textarea
            id="proposal-description"
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Describe the proposal in detail. Markdown is supported..."
            rows={8}
            maxLength={1024}
            className="w-full resize-y rounded-lg border border-card-border bg-background px-4 py-2.5 text-sm text-foreground placeholder:text-muted focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple"
          />
          <p className="mt-1 text-right text-xs text-muted">
            {description.length}/1024
          </p>
        </div>

        {/* Action Type */}
        <div>
          <label
            htmlFor="action-type"
            className="mb-1.5 block text-sm font-medium text-foreground"
          >
            Action Type
          </label>
          <select
            id="action-type"
            value={actionType}
            onChange={(e) =>
              handleActionTypeChange(e.target.value as ActionType)
            }
            className="w-full rounded-lg border border-card-border bg-background px-4 py-2.5 text-sm text-foreground focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple"
          >
            {ACTION_TYPES.map((type) => (
              <option key={type.value} value={type.value}>
                {type.label}
              </option>
            ))}
          </select>
          <p className="mt-1 text-xs text-muted">
            {ACTION_TYPES.find((t) => t.value === actionType)?.description}
          </p>
        </div>

        {/* Dynamic parameters */}
        <div className="rounded-lg border border-card-border bg-card-bg p-4">
          <h3 className="mb-3 text-sm font-semibold text-foreground">
            Parameters
          </h3>
          <div className="space-y-3">
            {fields.map((field) => (
              <div key={field.key}>
                <label
                  htmlFor={`param-${field.key}`}
                  className="mb-1 block text-xs font-medium text-muted"
                >
                  {field.label}
                </label>
                <input
                  id={`param-${field.key}`}
                  type="text"
                  value={parameters[field.key] ?? ""}
                  onChange={(e) =>
                    handleParameterChange(field.key, e.target.value)
                  }
                  placeholder={field.placeholder}
                  className="w-full rounded-lg border border-card-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted focus:border-accent-purple focus:outline-none focus:ring-1 focus:ring-accent-purple"
                />
              </div>
            ))}
          </div>
        </div>

        {/* Buttons */}
        <div className="flex gap-3 pt-2">
          <button
            type="submit"
            disabled={!isValid}
            className="flex-1 rounded-lg bg-accent-purple py-3 text-sm font-semibold text-white transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Submit Proposal
          </button>
          {onCancel && (
            <button
              type="button"
              onClick={onCancel}
              className="rounded-lg border border-card-border px-6 py-3 text-sm font-medium text-muted transition-colors hover:border-foreground hover:text-foreground"
            >
              Cancel
            </button>
          )}
        </div>
      </div>
    </form>
  );
}
