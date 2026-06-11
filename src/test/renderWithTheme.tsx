import type { ReactNode } from "react";
import { ThemeProvider } from "../hooks/useTheme";

export function ThemeTestWrapper({ children }: { children: ReactNode }) {
  return <ThemeProvider>{children}</ThemeProvider>;
}
