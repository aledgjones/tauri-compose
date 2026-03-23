import {
  mdiAlertCircleOutline,
  mdiCheckCircleOutline,
  mdiMinusCircleOutline,
} from "@mdi/js";
import {
  Instrument,
  PlaybackAssignmentMode,
  PlaybackAssignmentStatus,
  PluginInfo,
  PluginInventoryEntry,
  ResolvedPlaybackAssignment,
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
  assignment: ResolvedPlaybackAssignment | null | undefined,
  instrument: Instrument,
  pluginInventoryEntries: PluginInventoryEntry[] = [],
  samplerPlugins: PluginInfo[] = [],
): PlaybackReadinessPresentation => {
  const requestedPluginId =
    assignment?.effective_plugin_id ?? instrument.assignment_intent.plugin_id;
  const pluginLabel = requestedPluginId
    ? resolvePluginName(
        requestedPluginId,
        pluginInventoryEntries,
        samplerPlugins,
      )
    : null;

  switch (assignment?.status) {
    case PlaybackAssignmentStatus.Ready:
      return {
        kind: "ready",
        label: "Ready",
        summary: "Playable from this score context.",
        explanation: pluginLabel
          ? `Compose can use ${pluginLabel} for this instrument.`
          : "Compose can use the current plugin assignment for this instrument.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current plugin" : null,
        action_label: null,
        icon: mdiCheckCircleOutline,
        tone_class_name:
          "border-emerald-700/70 bg-emerald-500/10 text-emerald-100",
        icon_class_name: "text-emerald-200",
      };
    case PlaybackAssignmentStatus.NeedsAttention:
      return {
        kind: "needs-attention",
        label: "Needs attention",
        summary: "Not playable until the assignment is corrected.",
        explanation:
          instrument.assignment_intent.mode === PlaybackAssignmentMode.Direct &&
          instrument.assignment_intent.plugin_id
            ? "The assigned plugin is not currently available. Choose a different plugin in the Sequence row to recover playback readiness."
            : instrument.assignment_intent.mode ===
                PlaybackAssignmentMode.Targeted
              ? "This assignment still needs a target selection before Compose can use it for playback."
              : instrument.assignment_intent.mode ===
                  PlaybackAssignmentMode.SharedInstance
                ? "This shared assignment is not ready for playback in the current workflow yet."
                : "Compose cannot use the current playback assignment for this instrument yet.",
        plugin_label: pluginLabel,
        plugin_caption: pluginLabel ? "Current assignment" : null,
        action_label: "Focus row assignment",
        icon: mdiAlertCircleOutline,
        tone_class_name: "border-amber-700/70 bg-amber-500/10 text-amber-100",
        icon_class_name: "text-amber-200",
      };
    case PlaybackAssignmentStatus.Unassigned:
    default:
      return {
        kind: "unavailable",
        label: "Unavailable",
        summary: "No playable assignment is available yet.",
        explanation:
          "This instrument does not have a plugin assigned yet. Use the Sequence row assignment control to choose one playable plugin.",
        plugin_label: null,
        plugin_caption: null,
        action_label: "Focus row assignment",
        icon: mdiMinusCircleOutline,
        tone_class_name:
          "border-neutral-600 bg-neutral-700/70 text-neutral-100",
        icon_class_name: "text-neutral-200",
      };
  }
};
