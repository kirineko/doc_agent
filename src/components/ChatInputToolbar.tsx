import { useRef, type ReactNode } from "react";
import { IMAGE_FILE_ACCEPT, isAllowedImageFile, resolveImageMime } from "../lib/attachments";
import { PARALLEL_LIMIT_MESSAGE } from "../lib/sessionRunState";
import {
  ImageIcon,
  panelIconButtonClassName,
  PlusIcon,
  SlashIcon,
  TOOLBAR_ICON_CLASS,
} from "./PanelIcons";

interface ChatInputToolbarProps {
  /** clarify / busy / initializing */
  disabled: boolean;
  projectSelected: boolean;
  supportsVision: boolean;
  slashMenuOpen: boolean;
  canSend: boolean;
  busy: boolean;
  showStop?: boolean;
  stopping?: boolean;
  sendBlockedByParallel?: boolean;
  onSend: () => void;
  onStop?: () => void;
  onImportFiles: (files: File[]) => void | Promise<void>;
  onPickImage: (file: File, mime: string) => void | Promise<void>;
  onToggleSlashMenu: () => void;
  onInvalidImage?: () => void;
}

function ToolbarIconButton({
  label,
  title,
  disabled,
  active,
  onClick,
  onMouseDown,
  children,
}: {
  label: string;
  title: string;
  disabled: boolean;
  active?: boolean;
  onClick?: () => void;
  onMouseDown?: (event: React.MouseEvent<HTMLButtonElement>) => void;
  children: ReactNode;
}) {
  return (
    <button
      type="button"
      className={panelIconButtonClassName({ active, size: "toolbar" })}
      aria-label={label}
      title={title}
      disabled={disabled}
      aria-pressed={active}
      onClick={onClick}
      onMouseDown={onMouseDown}
    >
      {children}
    </button>
  );
}

export function ChatInputToolbar({
  disabled,
  projectSelected,
  supportsVision,
  slashMenuOpen,
  canSend,
  busy,
  showStop = false,
  stopping = false,
  sendBlockedByParallel = false,
  onSend,
  onStop,
  onImportFiles,
  onPickImage,
  onToggleSlashMenu,
  onInvalidImage,
}: ChatInputToolbarProps) {
  const importInputRef = useRef<HTMLInputElement>(null);
  const imageInputRef = useRef<HTMLInputElement>(null);
  const actionsDisabled = disabled || !projectSelected;
  const imageDisabled = actionsDisabled || !supportsVision;
  const imageTitle = !projectSelected
    ? "请先选择项目"
    : !supportsVision
      ? "当前模型不支持图片输入"
      : "选择图片附件";

  function pickImportFiles() {
    importInputRef.current?.click();
  }

  function pickImageFile() {
    imageInputRef.current?.click();
  }

  return (
    <div className="flex items-center gap-2 border-t border-border px-2 py-1.5">
      <input
        ref={importInputRef}
        type="file"
        multiple
        className="hidden"
        disabled={actionsDisabled}
        onChange={(event) => {
          const files = [...(event.target.files ?? [])];
          event.target.value = "";
          if (files.length > 0) void onImportFiles(files);
        }}
      />
      <input
        ref={imageInputRef}
        type="file"
        accept={IMAGE_FILE_ACCEPT}
        className="hidden"
        disabled={imageDisabled}
        onChange={(event) => {
          const file = event.target.files?.[0];
          event.target.value = "";
          if (!file) return;
          if (!isAllowedImageFile(file)) {
            onInvalidImage?.();
            return;
          }
          const mime = resolveImageMime(file);
          if (!mime) {
            onInvalidImage?.();
            return;
          }
          void onPickImage(file, mime);
        }}
      />
      <div className="flex items-center gap-0.5">
        <ToolbarIconButton
          label="上传文件到项目根目录"
          title={projectSelected ? "上传文件到项目根目录" : "请先选择项目"}
          disabled={actionsDisabled}
          onClick={pickImportFiles}
        >
          <PlusIcon className={TOOLBAR_ICON_CLASS} />
        </ToolbarIconButton>
        <ToolbarIconButton
          label="选择图片附件"
          title={imageTitle}
          disabled={imageDisabled}
          onClick={pickImageFile}
        >
          <ImageIcon className={TOOLBAR_ICON_CLASS} />
        </ToolbarIconButton>
        <ToolbarIconButton
          label="斜杠命令"
          title={projectSelected ? "斜杠命令（/init 等可执行命令）" : "请先选择项目"}
          disabled={actionsDisabled}
          active={slashMenuOpen}
          onMouseDown={(event) => {
            event.stopPropagation();
            onToggleSlashMenu();
          }}
        >
          <SlashIcon className={TOOLBAR_ICON_CLASS} />
        </ToolbarIconButton>
      </div>
      {showStop ? (
        <button
          type="button"
          className="ml-auto shrink-0 rounded-md border border-rose-500/40 px-3 py-1.5 text-sm font-medium text-rose-400 hover:bg-rose-500/10 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={stopping}
          onClick={onStop}
        >
          {stopping ? "停止中…" : "停止"}
        </button>
      ) : (
        <button
          type="button"
          className="btn-primary ml-auto shrink-0 rounded-md px-3 py-1.5 text-sm font-medium"
          disabled={disabled || !canSend || sendBlockedByParallel}
          title={sendBlockedByParallel ? PARALLEL_LIMIT_MESSAGE : undefined}
          onClick={onSend}
        >
          {busy ? "发送中" : "发送"}
        </button>
      )}
    </div>
  );
}
