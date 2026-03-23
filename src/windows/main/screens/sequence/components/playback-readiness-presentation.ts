import {
  mdiAlertCircleOutline,
  mdiCheckCircleOutline,
  mdiMinusCircleOutline,
} from "@mdi/js";
import {
  Instrument,
  PlaybackRack,
  PlaybackRouteAddress,
  PlaybackRouteAddressMode,
  PlaybackRouteTargetMode,
  PluginInventoryEntry,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
  UniqueId,
} from "../../../../../services/types";

type PlaybackReadinessKind = "ok" | "needs-attention";

export type PlaybackReadinessPresentation = {
  kind: PlaybackReadinessKind;
  label: string;
  summary: string;
  explanation: string;
  plugin_label: string | null;
  plugin_caption: string | null;
  action_label: string | null;
  icon: string;
  tone_class_name: string;
  icon_class_name: string;
  slot_summary: string;
  target_summary: string;
  address_summary: string | null;
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

const getPlaybackRouteTargetLabel = (
  route: ResolvedPlaybackRoute | null | undefined,
) => {
  switch (route?.target?.mode) {
    case PlaybackRouteTargetMode.Plugin:
      return "Target Plugin output";
    case PlaybackRouteTargetMode.PluginPart:
      return `Target Plugin part ${(route.target.index ?? 0) + 1}`;
    case PlaybackRouteTargetMode.MidiChannel:
      return `Target MIDI channel ${(route.target.index ?? 0) + 1}`;
    default:
      return "No route target selected";
  }
};

const getPlaybackRouteAddressLabel = (
  address: PlaybackRouteAddress | null | undefined,
) => {
  switch (address?.mode) {
    case PlaybackRouteAddressMode.Root:
      return "Address Root";
    case PlaybackRouteAddressMode.Part:
      return `Address Part ${(address.index ?? 0) + 1}`;
    case PlaybackRouteAddressMode.MidiChannel:
      return `Address MIDI channel ${(address.index ?? 0) + 1}`;
    default:
      return null;
  }
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
    : "Unassigned";

  if (slotIndex >= 0) {
    return `${slotIndex + 1}. ${pluginLabel}`;
  }

  return `${slotId}. ${pluginLabel}`;
};

export const getPlaybackReadinessPresentation = (
  route: ResolvedPlaybackRoute | null | undefined,
  _instrument: Instrument,
  pluginInventoryEntries: PluginInventoryEntry[] = [],
  rack: PlaybackRack | null | undefined = null,
): PlaybackReadinessPresentation => {
  const slotId = route?.slot_id ?? null;
  const hasSlotReference = slotId !== null;
  const hasSlotInRack = hasSlotReference
    ? rack?.by_id[slotId] !== undefined
    : false;
  const requestedPluginId =
    route?.effective_plugin_id ?? route?.slot_target?.plugin_id ?? null;
  const pluginLabel = requestedPluginId
    ? resolvePluginName(requestedPluginId, pluginInventoryEntries)
    : null;
  const slotLabel = hasSlotReference
    ? hasSlotInRack
      ? getPlaybackRackSlotLabel(
          slotId,
          rack,
          pluginInventoryEntries,
          requestedPluginId,
        )
      : `Missing slot ${slotId}`
    : null;
  const targetSummary = getPlaybackRouteTargetLabel(route);
  const addressSummary = getPlaybackRouteAddressLabel(route?.address);

  if (hasSlotReference && !hasSlotInRack) {
    return {
      kind: "needs-attention",
      label: "Needs attention",
      summary: "A previously configured rack slot is missing.",
      explanation:
        "This instrument still points at a score rack slot that no longer exists. Open the current rack to repair or replace the configuration.",
      plugin_label: pluginLabel,
      plugin_caption: pluginLabel ? "Previous slot target" : null,
      action_label: "Open rack",
      icon: mdiAlertCircleOutline,
      tone_class_name: "text-amber-300",
      icon_class_name: "text-amber-300",
      slot_summary: slotLabel ?? "Missing slot",
      target_summary: targetSummary,
      address_summary: addressSummary,
    };
  }

  switch (route?.status) {
    case ResolvedPlaybackRouteStatus.Ready:
      return {
        kind: "ok",
        label: "Okay",
        summary: "Playable from this score context.",
        explanation: pluginLabel
          ? `Compose can play this instrument through ${pluginLabel} on the current score rack.`
          : "Compose can use the current routed rack slot for this instrument.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current slot target" : null,
        action_label: null,
        icon: mdiCheckCircleOutline,
        tone_class_name: "text-emerald-300",
        icon_class_name: "text-emerald-300",
        slot_summary: slotLabel
          ? `Slot ${slotLabel}`
          : "Slot label unavailable",
        target_summary: targetSummary,
        address_summary: addressSummary,
      };
    case ResolvedPlaybackRouteStatus.NeedsAttention:
      return {
        kind: "needs-attention",
        label: "Needs attention",
        summary: "Not playable until the route is corrected.",
        explanation:
          route?.target?.mode === PlaybackRouteTargetMode.Plugin &&
          route?.slot_target?.plugin_id
            ? "The routed slot points at a plugin that is not currently available. Open the current rack to repair the slot or route."
            : route?.target?.mode === PlaybackRouteTargetMode.PluginPart
              ? "This route still needs a target selection before Compose can use it for playback. Open the current rack to inspect the routed slot."
              : route?.target?.mode === PlaybackRouteTargetMode.MidiChannel
                ? "This channel-addressed route is not ready for playback in the current workflow yet. Open the current rack to inspect the routed slot."
                : "Compose cannot use the current playback route for this instrument yet. Open the current rack to inspect the score-context slot state.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current slot target" : null,
        action_label: "Open rack",
        icon: mdiAlertCircleOutline,
        tone_class_name: "text-amber-300",
        icon_class_name: "text-amber-300",
        slot_summary: slotLabel
          ? `Slot ${slotLabel}`
          : "No routed slot available",
        target_summary: targetSummary,
        address_summary: addressSummary,
      };
    case ResolvedPlaybackRouteStatus.Unrouted:
    default:
      return {
        kind: "ok",
        label: "Okay",
        summary: "No playable route is available yet.",
        explanation:
          "This instrument is not routed to a playable score rack slot yet.",
        plugin_label: null,
        plugin_caption: null,
        action_label: null,
        icon: mdiMinusCircleOutline,
        tone_class_name: "text-neutral-300",
        icon_class_name: "text-neutral-300",
        slot_summary: "No routed slot yet",
        target_summary: targetSummary,
        address_summary: addressSummary,
      };
  }
};
