interface ChoiceChipOption {
  value: string;
  label: string;
  hint?: string;
}

interface ChoiceChipGroupProps {
  label: string;
  value: string;
  options: ChoiceChipOption[];
  layout: "grid" | "row";
  onChange: (value: string) => void;
}

export function ChoiceChipGroup({ label, value, options, layout, onChange }: ChoiceChipGroupProps) {
  return (
    <div
      role="radiogroup"
      aria-label={label}
      className={layout === "grid" ? "grid grid-cols-2 gap-1" : "flex gap-1"}
    >
      {options.map((option) => {
        const selected = option.value === value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={option.hint ? `${option.label} · ${option.hint}` : option.label}
            title={option.hint}
            className={`chip-surface min-w-0 rounded-md px-2 py-1.5 text-[11px] ${
              layout === "row" ? "flex-1" : ""
            } ${selected ? "chip-surface-selected font-medium" : ""}`}
            onClick={() => onChange(option.value)}
          >
            <span className="flex items-center justify-center gap-1.5">
              <span className="truncate">{option.label}</span>
              {option.hint ? (
                <span
                  aria-label={option.hint}
                  className="h-1.5 w-1.5 shrink-0 rounded-full bg-amber-500"
                />
              ) : null}
            </span>
          </button>
        );
      })}
    </div>
  );
}
