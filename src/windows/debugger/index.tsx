import {
  useAppState,
  useAudioState,
  usePluginInventoryState,
} from "../../services/state";
import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { AppBar } from "../../components/app-bar";
import { actions } from "../../services/actions";
import { chromeDark, ObjectInspector } from "react-inspector";
import { Switch } from "../../ui/switch";
import { PerformanceTimings } from "./components/performance-timings";
import { Input } from "../../ui/input";
import { Select, SelectOption } from "../../ui/select";
import {
  PluginEditorHostValidation,
  PluginEditorPocProbeLogCrashHint,
  PluginEditorPocProbeProfile,
} from "../../services/types";

const inspectorTheme = {
  ...chromeDark,
  BASE_BACKGROUND_COLOR: "transparent",
} as unknown as string;

const probeProfileOptions: SelectOption<PluginEditorPocProbeProfile>[] = [
  {
    value: "default",
    label: "Default",
  },
  {
    value: "controller_init_host_only_lifecycle_aligned",
    label: "Controller Init Host-Only Lifecycle Aligned",
  },
];

const probeProfileLabel = (profile: PluginEditorPocProbeProfile) => {
  if (profile === "controller_init_host_only_lifecycle_aligned") {
    return "Controller Init Host-Only Lifecycle Aligned";
  }

  return "Default";
};

const readPersistedProbeLog = async (
  setProbeLogPath: (value: string | null) => void,
  setSelectedProbeProfile: (value: PluginEditorPocProbeProfile) => void,
  setProbeCrashHint: (value: PluginEditorPocProbeLogCrashHint | null) => void,
  setProbeLog: (value: string | null) => void,
  setProbeLogError: (value: string | null) => void,
  setIsProbeLogLoading: (value: boolean) => void,
) => {
  setIsProbeLogLoading(true);

  try {
    const persistedLog = await actions.pluginInventory.readProbeLog();
    setProbeLogPath(persistedLog.path);
    if (persistedLog.profile) {
      setSelectedProbeProfile(persistedLog.profile);
    }
    setProbeCrashHint(persistedLog.crash_hint);
    setProbeLog(persistedLog.contents);
    setProbeLogError(null);
  } catch (error) {
    setProbeLogPath(null);
    setProbeCrashHint(null);
    setProbeLog(null);
    setProbeLogError(error instanceof Error ? error.message : String(error));
  } finally {
    setIsProbeLogLoading(false);
  }
};

export const Main = () => {
  const state = useAppState();
  const audio = useAudioState();
  const pluginInventory = usePluginInventoryState();
  const [selectedPluginId, setSelectedPluginId] = useState<string | null>(null);
  const [selectedProbeProfile, setSelectedProbeProfile] =
    useState<PluginEditorPocProbeProfile>("default");
  const [pluginPathOverride, setPluginPathOverride] = useState("");
  const [probeResult, setProbeResult] =
    useState<PluginEditorHostValidation | null>(null);
  const [probeError, setProbeError] = useState<string | null>(null);
  const [isProbeRunning, setIsProbeRunning] = useState(false);
  const [probeLogPath, setProbeLogPath] = useState<string | null>(null);
  const [probeCrashHint, setProbeCrashHint] =
    useState<PluginEditorPocProbeLogCrashHint | null>(null);
  const [probeLog, setProbeLog] = useState<string | null>(null);
  const [probeLogError, setProbeLogError] = useState<string | null>(null);
  const [isProbeLogLoading, setIsProbeLogLoading] = useState(false);

  const pluginOptions = useMemo<SelectOption<string>[]>(() => {
    return [...pluginInventory.entries]
      .sort((left, right) => {
        const nameComparison = left.name.localeCompare(right.name, undefined, {
          sensitivity: "base",
        });

        if (nameComparison !== 0) {
          return nameComparison;
        }

        return left.id.localeCompare(right.id);
      })
      .map((entry) => ({
        value: entry.id,
        label: entry.name,
      }));
  }, [pluginInventory.entries]);

  const selectedPlugin = useMemo(() => {
    return (
      pluginInventory.entries.find((entry) => entry.id === selectedPluginId) ??
      null
    );
  }, [pluginInventory.entries, selectedPluginId]);

  const trimmedPluginPathOverride = pluginPathOverride.trim();
  const probeTargetPath =
    trimmedPluginPathOverride ||
    selectedPlugin?.bundle_path ||
    selectedPlugin?.id;
  const canRunProbe =
    !isProbeRunning &&
    (trimmedPluginPathOverride.length > 0 || selectedPluginId !== null);
  const probeSummary =
    probeResult?.maintainer_load_init_probe?.summary ?? probeResult?.message;

  useEffect(() => {
    const show = async () => {
      const win = getCurrentWindow();
      await win.show();
    };

    show();
  }, []);

  useEffect(() => {
    void readPersistedProbeLog(
      setProbeLogPath,
      setSelectedProbeProfile,
      setProbeCrashHint,
      setProbeLog,
      setProbeLogError,
      setIsProbeLogLoading,
    );
  }, []);

  useEffect(() => {
    if (pluginOptions.length === 0) {
      if (selectedPluginId !== null) {
        setSelectedPluginId(null);
      }

      return;
    }

    const hasSelectedPlugin = pluginOptions.some(
      (option) => option.value === selectedPluginId,
    );

    if (!hasSelectedPlugin) {
      setSelectedPluginId(pluginOptions[0]?.value ?? null);
    }
  }, [pluginOptions, selectedPluginId]);

  const handleRunProbe = async () => {
    if (!canRunProbe) {
      return;
    }

    const probeRequest = {
      plugin_id: trimmedPluginPathOverride.length > 0 ? null : selectedPluginId,
      plugin_path:
        trimmedPluginPathOverride.length > 0 ? trimmedPluginPathOverride : null,
      profile: selectedProbeProfile,
    };

    setIsProbeRunning(true);
    setProbeError(null);

    try {
      const preparedLog =
        await actions.pluginInventory.prepareProbeLog(probeRequest);
      setProbeLogPath(preparedLog.path);
      if (preparedLog.profile) {
        setSelectedProbeProfile(preparedLog.profile);
      }
      setProbeCrashHint(preparedLog.crash_hint);
      setProbeLog(preparedLog.contents);
      setProbeLogError(null);

      const validation =
        await actions.pluginInventory.probeLoadInit(probeRequest);

      setProbeResult(validation);
    } catch (error) {
      setProbeResult(null);
      setProbeError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsProbeRunning(false);
      void readPersistedProbeLog(
        setProbeLogPath,
        setSelectedProbeProfile,
        setProbeCrashHint,
        setProbeLog,
        setProbeLogError,
        setIsProbeLogLoading,
      );
    }
  };

  return (
    <div className="h-dvh bg-neutral-950 grid grid-rows-[28px_auto_auto_auto_1fr_1fr] gap-2 overflow-hidden pb-2">
      <AppBar title="Debugger" />

      <PerformanceTimings />

      <div className="bg-neutral-900 p-2 mx-2 flex flex-col gap-2">
        <div className="flex flex-wrap items-end gap-2">
          <Select
            ariaLabel="Inventory plugin"
            className="min-w-64 flex-1"
            value={selectedPluginId}
            options={pluginOptions}
            onChange={setSelectedPluginId}
            variant="filled"
            label="Inventory Plugin"
            placeholder={
              pluginOptions.length > 0
                ? "Select a discovered plugin"
                : "No discovered plugins"
            }
            disabled={pluginOptions.length === 0}
          />

          <Select
            ariaLabel="Probe profile"
            className="min-w-56"
            value={selectedProbeProfile}
            options={probeProfileOptions}
            onChange={(value) => {
              if (value) {
                setSelectedProbeProfile(value);
              }
            }}
            variant="filled"
            label="Probe Profile"
          />

          <Input
            ariaLabel="Plugin path override"
            className="min-w-72 flex-2"
            type="text"
            label="Plugin Path Override"
            placeholder="Optional absolute .vst3 bundle path"
            value={pluginPathOverride}
            onChange={setPluginPathOverride}
            variant="dark"
          />

          <button
            type="button"
            onClick={handleRunProbe}
            disabled={!canRunProbe}
            className="h-9 shrink-0 px-3 text-sm text-white bg-sky-700 hover:bg-sky-600 disabled:bg-neutral-800 disabled:text-neutral-500 disabled:cursor-not-allowed"
          >
            {isProbeRunning ? "Running Probe..." : "Run Load/Init Probe"}
          </button>

          <button
            type="button"
            onClick={() =>
              void readPersistedProbeLog(
                setProbeLogPath,
                setSelectedProbeProfile,
                setProbeCrashHint,
                setProbeLog,
                setProbeLogError,
                setIsProbeLogLoading,
              )
            }
            disabled={isProbeLogLoading}
            className="h-9 shrink-0 px-3 text-sm text-white bg-neutral-800 hover:bg-neutral-700 disabled:bg-neutral-800 disabled:text-neutral-500 disabled:cursor-not-allowed"
          >
            {isProbeLogLoading ? "Refreshing Log..." : "Refresh Probe Log"}
          </button>
        </div>

        <div className="text-xs text-neutral-400">
          Uses the maintainer-only load/init probe and emits the returned editor
          host validation back through plugin inventory state.
        </div>

        <div className="text-xs text-neutral-500">
          The debugger prepares the persisted log first, then invokes the risky
          host probe.
        </div>

        <div className="text-xs text-neutral-500">
          Default keeps the current bounded contract. Controller Init Host-Only
          Lifecycle Aligned keeps the delayed runtime ordering, removes optional
          init-time host interfaces, and logs direct `queryInterface` requests
          from the controller-init context.
        </div>

        <div className="text-xs text-neutral-500">
          Probe Profile: {probeProfileLabel(selectedProbeProfile)}
        </div>

        {probeTargetPath ? (
          <div className="text-xs text-neutral-500">
            Target: {probeTargetPath}
          </div>
        ) : null}

        {probeLogPath ? (
          <div className="text-xs text-neutral-500 break-all">
            Log Path: {probeLogPath}
          </div>
        ) : null}

        {probeError ? (
          <div
            role="alert"
            className="rounded-sm border border-rose-800/80 bg-rose-950/40 px-3 py-2 text-sm text-rose-100"
          >
            {probeError}
          </div>
        ) : null}

        {probeCrashHint ? (
          <div className="rounded-sm border border-amber-800/80 bg-amber-950/30 px-3 py-2 text-sm text-amber-100">
            <div className="font-medium">{probeCrashHint.message}</div>
            <div className="mt-1 text-xs text-amber-200 break-all">
              Last recorded line: {probeCrashHint.last_line}
            </div>
          </div>
        ) : null}

        {probeResult ? (
          <div className="rounded-sm border border-neutral-800 bg-neutral-950/60 p-2">
            <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-400">
              Latest Probe Result
            </div>
            {probeSummary ? (
              <div className="mt-1 text-sm text-white">{probeSummary}</div>
            ) : null}
            <div className="mt-2 max-h-56 overflow-auto">
              <ObjectInspector
                data={probeResult}
                name="load_init_probe"
                theme={inspectorTheme}
                expandPaths={["$"]}
              />
            </div>
          </div>
        ) : null}

        <div className="rounded-sm border border-neutral-800 bg-neutral-950/60 p-2">
          <div className="text-[11px] uppercase tracking-[0.18em] text-neutral-400">
            Persisted Probe Log
          </div>
          <div className="mt-1 text-xs text-neutral-400">
            Backend-owned log entries are written before each risky probe phase
            so the last started step survives a hard abort and can be inspected
            after restart.
          </div>

          {probeLogError ? (
            <div
              role="alert"
              className="mt-2 rounded-sm border border-rose-800/80 bg-rose-950/40 px-3 py-2 text-sm text-rose-100"
            >
              {probeLogError}
            </div>
          ) : null}

          {probeLog ? (
            <pre
              aria-label="Persisted probe log"
              className="mt-2 max-h-56 overflow-auto whitespace-pre-wrap break-all bg-black/20 px-3 py-2 text-xs text-neutral-200"
            >
              {probeLog}
            </pre>
          ) : (
            <div className="mt-2 rounded-sm border border-dashed border-neutral-800 px-3 py-2 text-xs text-neutral-500">
              No persisted maintainer probe log has been written yet.
            </div>
          )}
        </div>
      </div>

      <div className="bg-neutral-900 p-2 mx-2 flex items-center">
        <Switch
          label="Render Spacing Debugging"
          value={state.ui.renderer.debug}
          onChange={() => actions.ui.renderer.toggleDebug()}
        />
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={{
            ui: state.ui,
            score: state.score,
            preferences: state.preferences,
            current_file_path: state.current_file_path,
          }}
          name="state"
          theme={inspectorTheme}
          expandPaths={["$"]}
        />
      </div>

      <div className="bg-neutral-900 p-2 overflow-auto mx-2">
        <ObjectInspector
          data={audio}
          name="audio"
          theme={inspectorTheme}
          expandPaths={["$"]}
        />
      </div>
    </div>
  );
};
