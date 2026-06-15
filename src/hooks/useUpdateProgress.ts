import { useSyncExternalStore } from "react";
import {
  getUpdateProgressSnapshot,
  subscribeUpdateProgress,
  type UpdateProgressState,
} from "../lib/updateProgress";

export function useUpdateProgress(): UpdateProgressState {
  return useSyncExternalStore(subscribeUpdateProgress, getUpdateProgressSnapshot);
}
