interface KeyEntryProps {
  value: string;
  placeholder: string;
  onChange: (value: string) => void;
  onSave: () => void;
}

export function KeyEntry({ value, placeholder, onChange, onSave }: KeyEntryProps) {
  return (
    <>
      <input
        type="password"
        className="input-field w-full rounded-md px-2 py-1 text-xs"
        placeholder={placeholder}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") onSave();
        }}
      />
      <button
        type="button"
        className="w-full rounded-md border border-indigo-700 bg-accent-muted px-2 py-0.5 text-[11px] hover:border-indigo-500"
        onClick={onSave}
      >
        保存
      </button>
    </>
  );
}
