import {
  mdiAlertCircleOutline,
  mdiCheckCircleOutline,
  mdiMinusCircleOutline,
} from "@mdi/js";
import {
  Instrument,
  PlaybackRouteTargetMode,
  PluginInfo,
  PluginInventoryEntry,
  ResolvedPlaybackRoute,
  ResolvedPlaybackRouteStatus,
} from "../../../../../services/types";

type PlaybackReadinessKind = "ready" | "needs-attention" | "unavailable";

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
};

const resolvePluginName = (
  pluginId: string,
  pluginInventoryEntries: PluginInventoryEntry[],
  samplerPlugins: PluginInfo[],
) => {
  return (
    pluginInventoryEntries.find((entry) => entry.id === pluginId)?.name ??
    samplerPlugins.find((plugin) => plugin.id === pluginId)?.name ??
    pluginId
  );
};

export const getPlaybackReadinessPresentation = (
  route: ResolvedPlaybackRoute | null | undefined,
  _instrument: Instrument,
  pluginInventoryEntries: PluginInventoryEntry[] = [],
  samplerPlugins: PluginInfo[] = [],
): PlaybackReadinessPresentation => {
  const requestedPluginId =
    route?.effective_plugin_id ?? route?.slot_target?.plugin_id ?? null;
  const pluginLabel = requestedPluginId
    ? resolvePluginName(
        requestedPluginId,
        pluginInventoryEntries,
        samplerPlugins,
      )
    : null;

  switch (route?.status) {
    case ResolvedPlaybackRouteStatus.Ready:
      return {
        kind: "ready",
        label: "Ready",
        summary: "Playable from this score context.",
        explanation: pluginLabel
          ? `Compose can use ${pluginLabel} for this instrument.`
          : "Compose can use the current routed plugin for this instrument.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current plugin" : null,
        action_label: null,
        icon: mdiCheckCircleOutline,
        tone_class_name:
          "border-emerald-700/70 bg-emerald-500/10 text-emerald-100",
        icon_class_name: "text-emerald-200",
      };
    case ResolvedPlaybackRouteStatus.NeedsAttention:
      return {
        kind: "needs-attention",
        label: "Needs attention",
        summary: "Not playable until the route is corrected.",
        explanation:
          route?.target?.mode === PlaybackRouteTargetMode.Plugin &&
          route?.slot_target?.plugin_id
            ? "The routed slot points at a plugin that is not currently available. Use the current rack workflow to repair the slot or route."
            : route?.target?.mode === PlaybackRouteTargetMode.PluginPart
              ? "This route still needs a target selection before Compose can use it for playback."
              : route?.target?.mode === PlaybackRouteTargetMode.MidiChannel
                ? "This channel-addressed route is not ready for playback in the current workflow yet."
                : "Compose cannot use the current playback route for this instrument yet.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current slot target" : null,
        action_label: "Focus route summary",
        icon: mdiAlertCircleOutline,
        tone_class_name: "border-amber-700/70 bg-amber-500/10 text-amber-100",
        icon_class_name: "text-amber-200",
      };
    case ResolvedPlaybackRouteStatus.Unrouted:
    default:
      return {
        kind: "unavailable",
        label: "Unavailable",
        summary: "No playable route is available yet.",
        explanation:
          "This instrument is not routed to a playable slot yet. Use the rack and route workflow in Sequence to prepare a slot and connect the instrument.",
        plugin_label: null,
        plugin_caption: null,
        action_label: "Focus route summary",
        icon: mdiMinusCircleOutline,
        tone_class_name:
          "border-neutral-600 bg-neutral-700/70 text-neutral-100",
        icon_class_name: "text-neutral-200",
      };
  }
};
