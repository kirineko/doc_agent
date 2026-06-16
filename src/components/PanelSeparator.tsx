import { Separator } from "react-resizable-panels";

interface PanelSeparatorProps {
  orientation?: "horizontal" | "vertical";
  disabled?: boolean;
}

export function PanelSeparator({
  orientation = "horizontal",
  disabled = false,
}: PanelSeparatorProps) {
  const isHorizontal = orientation === "horizontal";

  return (
    <Separator
      disabled={disabled}
      disableDoubleClick
      className={
        isHorizontal
          ? "group mx-0.5 flex w-2.5 shrink-0 items-stretch bg-transparent outline-none"
          : "group my-0.5 flex h-2.5 shrink-0 flex-col bg-transparent outline-none"
      }
    >
      <div
        className={
          isHorizontal
            ? "mx-auto w-px flex-1 rounded-full bg-transparent opacity-0 transition-[opacity,background-color] group-hover:opacity-100 group-hover:bg-link/50 group-active:bg-link group-active:opacity-100 group-data-[separator=active]:bg-link group-data-[separator=active]:opacity-100"
            : "my-auto h-px flex-1 rounded-full bg-transparent opacity-0 transition-[opacity,background-color] group-hover:opacity-100 group-hover:bg-link/50 group-active:bg-link group-active:opacity-100 group-data-[separator=active]:bg-link group-data-[separator=active]:opacity-100"
        }
      />
    </Separator>
  );
}
