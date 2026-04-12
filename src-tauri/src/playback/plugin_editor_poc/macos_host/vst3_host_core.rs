use super::super::{
    probe_profile_code, record_probe_failure, PluginEditorPocProbeProfile, PluginEditorTarget,
    PluginHostBuildTelemetry,
};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentHandlerTiming {
    BeforeControllerInitialize,
    AfterControllerInitialize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComponentRuntimeSetupTiming {
    BeforeControllerInitialize,
    AfterControllerInitialize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Vst3HostCoreLifecycleProfile {
    component_handler_timing: ComponentHandlerTiming,
    component_runtime_setup_timing: ComponentRuntimeSetupTiming,
}

impl Vst3HostCoreLifecycleProfile {
    pub(crate) fn from_probe_profile(profile: PluginEditorPocProbeProfile) -> Self {
        match profile {
            PluginEditorPocProbeProfile::Default => Self {
                component_handler_timing: ComponentHandlerTiming::BeforeControllerInitialize,
                component_runtime_setup_timing:
                    ComponentRuntimeSetupTiming::BeforeControllerInitialize,
            },
            PluginEditorPocProbeProfile::ControllerInitHostOnlyLifecycleAligned => Self {
                component_handler_timing: ComponentHandlerTiming::AfterControllerInitialize,
                component_runtime_setup_timing:
                    ComponentRuntimeSetupTiming::AfterControllerInitialize,
            },
        }
    }

    pub(crate) fn uses_pre_init_component_handler(self) -> bool {
        matches!(
            self.component_handler_timing,
            ComponentHandlerTiming::BeforeControllerInitialize
        )
    }

    pub(crate) fn delays_component_runtime_setup(self) -> bool {
        matches!(
            self.component_runtime_setup_timing,
            ComponentRuntimeSetupTiming::AfterControllerInitialize
        )
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn uses_host_only_controller_initialize_application(self) -> bool {
        false
    }

    fn controller_initialize_host_application(
        self,
        host_application: &ComPtr<IHostApplication>,
    ) -> Result<ComPtr<IHostApplication>, String> {
        super::create_controller_init_host_application_with_delegate(host_application)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Vst3HostCoreTeardownState {
    pub component_initialized: bool,
    pub controller_initialized: bool,
    pub controller_shared_with_component: bool,
    pub connection_points_connected: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vst3HostCoreTeardownStep {
    PrepareWindowClose,
    ClearControllerComponentHandler,
    DisconnectConnectionPoints,
    TerminateController,
    TerminateComponent,
}

pub(crate) fn teardown_steps(state: Vst3HostCoreTeardownState) -> Vec<Vst3HostCoreTeardownStep> {
    let mut steps = vec![Vst3HostCoreTeardownStep::PrepareWindowClose];

    if state.controller_initialized && !state.controller_shared_with_component {
        steps.push(Vst3HostCoreTeardownStep::ClearControllerComponentHandler);
    }
    if state.connection_points_connected {
        steps.push(Vst3HostCoreTeardownStep::DisconnectConnectionPoints);
    }
    if state.controller_initialized && !state.controller_shared_with_component {
        steps.push(Vst3HostCoreTeardownStep::TerminateController);
    }
    if state.component_initialized {
        steps.push(Vst3HostCoreTeardownStep::TerminateComponent);
    }

    steps
}

pub(crate) struct PluginHostCoreSession {
    pub(super) target: PluginEditorTarget,
    pub(super) plugin_path: PathBuf,
    pub(super) restorable_plugin_state: Option<PlaybackSlotPluginState>,
    pub(super) _module: LoadedVst3Module,
    pub(super) component: ComPtr<IComponent>,
    pub(super) controller: ComPtr<IEditController>,
    pub(super) view: Option<ComPtr<IPlugView>>,
    pub(super) _host_application: ComPtr<IHostApplication>,
    pub(super) _controller_initialize_host_application: ComPtr<IHostApplication>,
    pub(super) _component_handler_host_application: ComPtr<IHostApplication>,
    pub(super) _plug_frame: Option<ComPtr<IPlugFrame>>,
    pub(super) host_view_ptr: usize,
    pub(super) plugin_can_resize: bool,
    pub(super) view_attached: bool,
    pub(super) connection_points_connected: bool,
    pub(super) component_initialized: bool,
    pub(super) controller_initialized: bool,
    pub(super) controller_shared_with_component: bool,
    pub(super) close_prepared: bool,
    pub(super) render_processing_active: bool,
    pub(super) render_sample_rate: Option<u32>,
    pub(super) render_max_block_size: i32,
    pub(super) render_buses_activated: bool,
    pub(super) render_output_buses: Vec<OutputBusLayout>,
    pub(super) render_scratch: RenderScratch,
}

impl PluginHostCoreSession {
    pub(crate) fn target(&self) -> &PluginEditorTarget {
        &self.target
    }

    fn teardown_state(&self) -> Vst3HostCoreTeardownState {
        Vst3HostCoreTeardownState {
            component_initialized: self.component_initialized,
            controller_initialized: self.controller_initialized,
            controller_shared_with_component: self.controller_shared_with_component,
            connection_points_connected: self.connection_points_connected,
        }
    }

    pub(super) fn attach_view(
        &mut self,
        ns_window: *mut NSWindow,
        parent_view: &NSView,
        telemetry: &mut PluginHostBuildTelemetry,
    ) -> Result<(), PluginEditorHostAttemptError> {
        eprintln!(
            "[DIAG] attach_view: start (close_prepared={}, view_attached={}, view_exists={})",
            self.close_prepared,
            self.view_attached,
            self.view.is_some()
        );

        if self.component_initialized && self.close_prepared {
            let _ = unsafe { self.component.setActive(1) };
        }

        telemetry.record_starting(
            PluginEditorHostValidationPhase::ViewCreate,
            "About to create and prepare the VST3 editor view.",
        );
        let plug_frame = ComWrapper::new(PluginHostFrame {
            ns_window_ptr: ns_window,
        })
        .to_com_ptr::<IPlugFrame>()
        .ok_or_else(|| {
            record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::host_contract_failure(
                    PluginEditorHostValidationPhase::ViewCreate,
                    "Failed to allocate the VST3 plug-frame shim.",
                ),
            )
        })?;

        let view = unsafe { ComPtr::from_raw(self.controller.createView(ViewType::kEditor)) }
            .ok_or_else(|| {
                record_probe_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ViewCreate,
                        "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                    ),
                )
            })?;

        let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
        if platform_supported != kResultOk {
            return Err(record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ViewCreate,
                    "Plugin editor POC editor view does not report NSView support.",
                ),
            ));
        }

        let mut plug_view_size = ViewRect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { view.getSize(&mut plug_view_size) } == kResultOk {
            let width = (plug_view_size.right - plug_view_size.left).max(1) as f64;
            let height = (plug_view_size.bottom - plug_view_size.top).max(1) as f64;
            eprintln!(
                "[DIAG] attach_view: getSize returned {width}x{height}, calling setContentSize"
            );
            if let Some(ns_win) = unsafe { Retained::retain(ns_window) } {
                ns_win.setContentSize(NSSize::new(width, height));
            }
        }

        let plugin_can_resize = unsafe { view.canResize() } == kResultOk;
        if let Some(ns_win) = unsafe { Retained::retain(ns_window) } {
            let mut style = ns_win.styleMask();
            if plugin_can_resize {
                style |= NSWindowStyleMask::Resizable;
            } else {
                style -= NSWindowStyleMask::Resizable;
            }
            ns_win.setStyleMask(style);
        }

        telemetry.record_completed(
            PluginEditorHostValidationPhase::ViewCreate,
            "Created the VST3 editor view and prepared the host window.",
        );

        let _ = unsafe { view.setFrame(plug_frame.as_ptr()) };

        telemetry.record_starting(
            PluginEditorHostValidationPhase::ViewAttach,
            "About to attach the VST3 editor view to the bounded host surface.",
        );
        let parent_ptr = (parent_view as *const NSView).cast_mut().cast::<c_void>();
        let attached = unsafe { view.attached(parent_ptr, kPlatformTypeNSView) };
        if attached != kResultOk {
            return Err(record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ViewAttach,
                    format!(
                        "Plugin editor POC failed to attach the VST3 editor view to NSView (tresult = {}).",
                        attached
                    ),
                ),
            ));
        }

        telemetry.record_completed(
            PluginEditorHostValidationPhase::ViewAttach,
            "Attached the plugin editor view to the bounded host surface.",
        );

        self.view = Some(view);
        self._plug_frame = Some(plug_frame);
        self.host_view_ptr = parent_ptr as usize;
        self.plugin_can_resize = plugin_can_resize;
        self.view_attached = true;
        self.close_prepared = false;

        Ok(())
    }

    pub(super) fn prepare_for_window_close(&mut self) {
        unsafe {
            if self.close_prepared {
                return;
            }

            if self.view_attached {
                let Some(view) = self.view.as_ref() else {
                    self.view_attached = false;
                    self.close_prepared = true;
                    return;
                };
                let _ = view.setFrame(std::ptr::null_mut());
                let _ = view.removed();
                self.view_attached = false;
                self.view = None;
                self._plug_frame = None;
            }
            self.close_prepared = true;
        }
    }

    pub(super) fn shutdown(mut self) {
        let teardown_state = self.teardown_state();

        for step in teardown_steps(teardown_state) {
            match step {
                Vst3HostCoreTeardownStep::PrepareWindowClose => {
                    self.prepare_for_window_close();
                }
                Vst3HostCoreTeardownStep::ClearControllerComponentHandler => unsafe {
                    let _ = self.controller.setComponentHandler(std::ptr::null_mut());
                },
                Vst3HostCoreTeardownStep::DisconnectConnectionPoints => {
                    unsafe {
                        super::disconnect_component_and_controller(
                            &self.component,
                            &self.controller,
                        );
                    }
                    self.connection_points_connected = false;
                }
                Vst3HostCoreTeardownStep::TerminateController => unsafe {
                    let _ = self.controller.terminate();
                    self.controller_initialized = false;
                },
                Vst3HostCoreTeardownStep::TerminateComponent => unsafe {
                    let _ = self.component.setActive(0);
                    let _ = self.component.terminate();
                    self.component_initialized = false;
                },
            }
        }
    }

    pub(super) fn view_state(&self) -> Option<(ComPtr<IPlugView>, usize, bool)> {
        self.view
            .as_ref()
            .map(|view| (view.clone(), self.host_view_ptr, self.plugin_can_resize))
    }
}

pub(super) fn build_plugin_host_core_session_with_telemetry(
    target: &PluginEditorTarget,
    restorable_plugin_state: Option<&PlaybackSlotPluginState>,
    probe_profile: PluginEditorPocProbeProfile,
    telemetry: &mut PluginHostBuildTelemetry,
) -> Result<PluginHostCoreSession, PluginEditorHostAttemptError> {
    let lifecycle_profile = Vst3HostCoreLifecycleProfile::from_probe_profile(probe_profile);
    let plugin_path = target.path();
    telemetry.record_starting(
        PluginEditorHostValidationPhase::ModuleLoad,
        "About to load the VST3 module binary.",
    );
    let module = load_vst3_module(plugin_path).map_err(|error| {
        record_probe_failure(
            telemetry,
            PluginEditorHostAttemptError::host_contract_failure(
                PluginEditorHostValidationPhase::ModuleLoad,
                error,
            ),
        )
    })?;
    telemetry.record_completed(
        PluginEditorHostValidationPhase::ModuleLoad,
        "Loaded the VST3 module binary.",
    );
    telemetry.record_starting(
        PluginEditorHostValidationPhase::FactoryCreate,
        "About to resolve the VST3 plugin factory.",
    );
    let factory = create_plugin_factory(&module).map_err(|error| {
        record_probe_failure(
            telemetry,
            PluginEditorHostAttemptError::plugin_failure(
                PluginEditorHostValidationPhase::FactoryCreate,
                error,
            ),
        )
    })?;
    telemetry.record_completed(
        PluginEditorHostValidationPhase::FactoryCreate,
        "Resolved the VST3 plugin factory.",
    );

    telemetry.record_starting(
        PluginEditorHostValidationPhase::HostContext,
        "About to allocate the VST3 host application and configure the factory host context.",
    );
    let host_application = create_component_handler_host_application().map_err(|error| {
        record_probe_failure(
            telemetry,
            PluginEditorHostAttemptError::host_contract_failure(
                PluginEditorHostValidationPhase::HostContext,
                error,
            ),
        )
    })?;
    let controller_initialize_host_application = lifecycle_profile
        .controller_initialize_host_application(&host_application)
        .map_err(|error| {
            record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::host_contract_failure(
                    PluginEditorHostValidationPhase::HostContext,
                    error,
                ),
            )
        })?;
    let component_handler_host_application =
        create_component_handler_host_application().map_err(|error| {
            record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::host_contract_failure(
                    PluginEditorHostValidationPhase::HostContext,
                    error,
                ),
            )
        })?;
    let host_context = host_application.clone().upcast::<FUnknown>();
    configure_factory_host_context(&factory, &host_context);
    telemetry.record_completed(
        PluginEditorHostValidationPhase::HostContext,
        "Created the host context and configured the VST3 factory.",
    );

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ComponentCreate,
        "About to create the VST3 component instance.",
    );
    let component = create_component(&factory, plugin_path).map_err(|error| {
        record_probe_failure(
            telemetry,
            PluginEditorHostAttemptError::plugin_failure(
                PluginEditorHostValidationPhase::ComponentCreate,
                error,
            ),
        )
    })?;
    telemetry.record_completed(
        PluginEditorHostValidationPhase::ComponentCreate,
        "Created the VST3 component instance.",
    );
    let delay_component_runtime_setup = lifecycle_profile.delays_component_runtime_setup();
    telemetry.record_starting(
        PluginEditorHostValidationPhase::ComponentInitialize,
        if delay_component_runtime_setup {
            "About to initialize the VST3 component. Runtime bus activation and processing setup will be deferred until after controller initialization."
        } else {
            "About to initialize the VST3 component before controller initialization."
        },
    );
    let component_init = unsafe { component.initialize(host_context.as_ptr()) };
    if component_init != kResultOk {
        return Err(record_probe_failure(
            telemetry,
            PluginEditorHostAttemptError::plugin_failure(
                PluginEditorHostValidationPhase::ComponentInitialize,
                format!(
                    "Plugin editor POC failed to initialize the VST3 component (tresult = {}).",
                    component_init
                ),
            ),
        ));
    }

    if let Some(plugin_state) = restorable_plugin_state {
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "About to restore the portable component state into the VST3 component.",
        );
        restore_component_state(&component, plugin_state).map_err(|error| {
            record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ComponentStateSync,
                    error,
                ),
            )
        })?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "Restored the portable component state into the VST3 component.",
        );
    } else {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "Skipped because no portable component state was provided.",
        );
    }

    telemetry.record_completed(
        PluginEditorHostValidationPhase::ComponentInitialize,
        if delay_component_runtime_setup {
            "Initialized the VST3 component and deferred runtime bus activation, processing setup, and component activation until after controller initialization."
        } else {
            "Initialized the VST3 component."
        },
    );

    if !delay_component_runtime_setup {
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentRuntimeSetup,
            "About to activate buses, configure processing, and activate the VST3 component before controller initialization.",
        );
        apply_component_runtime_setup(&component, telemetry)?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentRuntimeSetup,
            "Activated buses, configured processing, and activated the VST3 component before controller initialization.",
        );
    }

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ControllerCreate,
        "About to create the VST3 edit controller.",
    );
    let CreatedController {
        controller,
        shared_with_component: controller_shared_with_component,
    } = match create_controller(&factory, &component) {
        Ok(controller) => controller,
        Err(error) => {
            unsafe {
                let _ = component.terminate();
            }
            return Err(record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ControllerCreate,
                    error,
                ),
            ));
        }
    };
    telemetry.record_completed(
        PluginEditorHostValidationPhase::ControllerCreate,
        "Created the VST3 edit controller.",
    );

    let component_handler_ptr = component_handler_host_application
        .clone()
        .upcast::<FUnknown>()
        .cast::<IComponentHandler>()
        .expect("PluginHostApplication must expose IComponentHandler");
    let set_component_handler_pre_init = lifecycle_profile.uses_pre_init_component_handler();

    if !controller_shared_with_component {
        if set_component_handler_pre_init {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                "About to set the VST3 component handler before controller initialization.",
            );
            let _ = unsafe { controller.setComponentHandler(component_handler_ptr.as_ptr()) };
            telemetry.log_phase_event(
                PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                "completed",
                "Set the VST3 component handler before controller initialization.",
            );
        } else {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
                format!(
                    "Skipped because the {} probe profile defers component-handler wiring until after controller initialization.",
                    probe_profile_code(probe_profile)
                ),
            );
        }
    } else {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
            "Skipped because the edit controller is shared with the component instance.",
        );
    }

    let controller_initialized = if controller_shared_with_component {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ControllerInitialize,
            "Skipped because the edit controller is shared with the component instance.",
        );
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ControllerSetComponentHandler,
            "Skipped because the edit controller is shared with the component instance.",
        );
        false
    } else {
        let controller_initialize_host_context = controller_initialize_host_application
            .clone()
            .upcast::<FUnknown>();
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ControllerInitialize,
            "About to initialize the VST3 edit controller.",
        );
        let controller_init =
            unsafe { controller.initialize(controller_initialize_host_context.as_ptr()) };
        if controller_init != kResultOk {
            unsafe {
                let _ = component.terminate();
            }
            return Err(record_probe_failure(
                telemetry,
                PluginEditorHostAttemptError::plugin_failure(
                    PluginEditorHostValidationPhase::ControllerInitialize,
                    format!(
                        "Plugin editor POC failed to initialize the VST3 edit controller (tresult = {}).",
                        controller_init
                    ),
                ),
            ));
        }
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ControllerInitialize,
            "Initialized the VST3 edit controller.",
        );

        if set_component_handler_pre_init {
            telemetry.record_skipped(
                PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                "Skipped because the default probe profile already wired the component handler before controller initialization.",
            );
        } else {
            telemetry.record_starting(
                PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                "About to set the VST3 component handler after controller initialization.",
            );
            let _ = unsafe { controller.setComponentHandler(component_handler_ptr.as_ptr()) };
            telemetry.record_completed(
                PluginEditorHostValidationPhase::ControllerSetComponentHandler,
                "Set the VST3 component handler after controller initialization.",
            );
        }
        true
    };

    if !controller_shared_with_component {
        telemetry.log_phase_event(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "starting",
            "About to synchronize the live component state into the controller before connection-point setup.",
        );
        sync_component_state_to_controller(&component, &controller);
        telemetry.log_phase_event(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "completed",
            "Synchronized the live component state into the controller before connection-point setup.",
        );
    }

    let connection_points_connected = if controller_shared_with_component {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ConnectionPointHandshake,
            "Skipped because the component and controller share one instance.",
        );
        false
    } else {
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ConnectionPointHandshake,
            "About to connect the VST3 component and controller connection points.",
        );
        match connect_component_and_controller(&component, &controller) {
            ConnectionPointHandshakeOutcome::Connected(message) => {
                telemetry.record_completed(
                    PluginEditorHostValidationPhase::ConnectionPointHandshake,
                    message,
                );
                true
            }
            ConnectionPointHandshakeOutcome::Skipped(message) => {
                telemetry.record_skipped(
                    PluginEditorHostValidationPhase::ConnectionPointHandshake,
                    message,
                );
                false
            }
        }
    };

    if delay_component_runtime_setup {
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentRuntimeSetup,
            "About to activate buses, configure processing, and activate the VST3 component after controller initialization.",
        );
        apply_component_runtime_setup(&component, telemetry)?;
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentRuntimeSetup,
            "Activated buses, configured processing, and activated the VST3 component after controller initialization.",
        );
    }

    Ok(PluginHostCoreSession {
        target: target.clone(),
        plugin_path: plugin_path.to_path_buf(),
        restorable_plugin_state: restorable_plugin_state.cloned(),
        _module: module,
        component,
        controller,
        view: None,
        _host_application: host_application,
        _controller_initialize_host_application: controller_initialize_host_application,
        _component_handler_host_application: component_handler_host_application,
        _plug_frame: None,
        host_view_ptr: 0,
        plugin_can_resize: false,
        view_attached: false,
        connection_points_connected,
        component_initialized: true,
        controller_initialized,
        controller_shared_with_component,
        close_prepared: false,
        render_processing_active: false,
        render_sample_rate: None,
        render_max_block_size: 0,
        render_buses_activated: false,
        render_output_buses: Vec::new(),
        render_scratch: RenderScratch::default(),
    })
}
