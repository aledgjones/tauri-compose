import { mdiAlertCircleOutline } from "@mdi/js";
import {
  Instrument,
  PlaybackRack,
  PluginInventoryEntry,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
  UniqueId,
} from "../../../../../services/types";

type PlaybackReadinessKind = "ok" | "needs-attention";

export type PlaybackReadinessPresentation = {
  kind: PlaybackReadinessKind;
  label: string;
  hover_title: string | null;
  icon: string;
  tone_class_name: string;
  icon_class_name: string;
};

export const resolvePluginName = (
  pluginId: string,
  pluginInventoryEntries: PluginInventoryEntry[],
) => {
  return (
    pluginInventoryEntries.find((entry) => entry.id === pluginId)?.name ??
    pluginId
  );
};

export const getPlaybackRackSlotLabel = (
  slotId: UniqueId,
  rack: PlaybackRack | null | undefined,
  pluginInventoryEntries: PluginInventoryEntry[] = [],
  fallbackPluginId: string | null = null,
) => {
  const slotIndex = rack?.order.indexOf(slotId) ?? -1;
  const slot = rack?.by_id[slotId] ?? null;
  const pluginId = fallbackPluginId ?? slot?.target?.plugin_id ?? null;
  const pluginLabel = pluginId
    ? resolvePluginName(pluginId, pluginInventoryEntries)
    : "--";

  if (slotIndex >= 0) {
    return `${slotIndex + 1}. ${pluginLabel}`;
  }

  return `${slotId}. ${pluginLabel}`;
};

export const getPlaybackReadinessPresentation = (
  route: ResolvedPlaybackRoute | null | undefined,
  _instrument: Instrument,
  _pluginInventoryEntries: PluginInventoryEntry[] = [],
  rack: PlaybackRack | null | undefined = null,
): PlaybackReadinessPresentation => {
  const hasSlotReference = route?.slot_id != null;
  const hasSlotInRack = hasSlotReference
    ? rack?.by_id[route.slot_id as UniqueId] !== undefined
    : false;

  if (route?.status === ResolvedPlaybackRouteStatus.Ready) {
    return {
      kind: "ok",
      label: "Ready",
      hover_title: null,
      icon: mdiAlertCircleOutline,
      tone_class_name: "text-amber-300",
      icon_class_name: "text-amber-300",
    };
  }

  return {
    kind: "needs-attention",
    label: "Needs attention",
    hover_title:
      hasSlotReference || hasSlotInRack
        ? "Repair this playback route to enable playback."
        : "Assign a playback route to enable playback.",
    icon: mdiAlertCircleOutline,
    tone_class_name: "text-amber-300",
    icon_class_name: "text-amber-300",
  };
};
