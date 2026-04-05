import { useCallback, useEffect, useRef, useState } from "react";
import type {
  ConnectionProfile,
  ConnectionStatus,
  ProfileStatus,
} from "../types";
import { api, onStatusChange } from "./useTauri";

export function useConnections() {
  const [profiles, setProfiles] = useState<ConnectionProfile[]>([]);
  const [statuses, setStatuses] = useState<Map<string, ProfileStatus>>(
    new Map(),
  );
  const [loading, setLoading] = useState(true);
  const unlistenRef = useRef<(() => void) | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [profs, stats] = await Promise.all([
        api.getProfiles(),
        api.getAllStatus(),
      ]);
      setProfiles(profs);
      const map = new Map<string, ProfileStatus>();
      for (const s of stats) {
        map.set(s.profileId, s);
      }
      setStatuses(map);
    } catch (e) {
      console.error("Failed to refresh:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    onStatusChange((event) => {
      setStatuses((prev) => {
        const next = new Map(prev);
        const existing = next.get(event.profileId);
        next.set(event.profileId, {
          profileId: event.profileId,
          profileName: existing?.profileName ?? "",
          status: event.status,
          tunnelStatuses: event.tunnelStatuses,
        });
        return next;
      });
    }).then((unlisten) => {
      unlistenRef.current = unlisten;
    }).catch((e) => {
      console.error("Failed to subscribe to status events:", e);
    });
    return () => {
      unlistenRef.current?.();
    };
  }, [refresh]);

  const getStatus = useCallback(
    (profileId: string): ConnectionStatus => {
      return statuses.get(profileId)?.status ?? "disconnected";
    },
    [statuses],
  );

  return { profiles, statuses, loading, refresh, getStatus };
}
