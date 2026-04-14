use super::super::{
    record_phase_failure, PluginEditorPocProbeProfile, PluginEditorTarget, PluginHostPhaseLogger,
};
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Vst3HostCoreLifecycleProfile {
    delay_component_runtime_setup: bool,
}

impl Vst3HostCoreLifecycleProfile {
    pub(crate) fn from_probe_profile(_profile: PluginEditorPocProbeProfile) -> Self {
        Self {
            // Richer instruments such as HALion can consult controller-side state that
            // depends on an already-prepared processor, so initialize the controller
            // against a runtime-ready component instead of deferring setup until later.
            delay_component_runtime_setup: false,
        }
    }

    #[allow(dead_code)] // Runtime setup deferral field is no longer consulted; init always defers to ensure_render_processing.
    pub(crate) fn delays_component_runtime_setup(self) -> bool {
        self.delay_component_runtime_setup
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

struct SteinbergPlugProviderSetup {
    host_application: PluginHostApplicationHandle,
    component_handler: ComPtr<IComponentHandler>,
    component: ComPtr<IComponent>,
    controller: ComPtr<IEditController>,
    controller_shared_with_component: bool,
    controller_initialized: bool,
    connection_points_connected: bool,
    component_connection_proxy: Option<ComPtr<IConnectionPoint>>,
    controller_connection_proxy: Option<ComPtr<IConnectionPoint>>,
}

fn initialize_steinberg_plug_provider(
    plugin_path: &Path,
    host_application: PluginHostApplicationHandle,
    factory: &ComPtr<IPluginFactory>,
    _lifecycle_profile: Vst3HostCoreLifecycleProfile,
    telemetry: &mut PluginHostPhaseLogger,
) -> Result<SteinbergPlugProviderSetup, PluginEditorHostAttemptError> {
    telemetry.record_starting(
        PluginEditorHostValidationPhase::HostContext,
        "About to configure the preallocated VST3 host application and factory host context.",
    );
    let host_context = host_application.clone_interface().upcast::<FUnknown>();
    let component_handler = create_component_handler().map_err(|error| {
        record_phase_failure(
            telemetry,
            PluginEditorHostAttemptError::host_contract_failure(
                PluginEditorHostValidationPhase::HostContext,
                error,
            ),
        )
    })?;
    let factory_host_context_result = configure_factory_host_context(factory, &host_context);
    let factory_host_context_message = match factory_host_context_result {
        Some(result) => format!(
            "Configured factory host context host_application_ptr={} host_context_ptr={} set_host_context_tresult={}",
            super::interface_pointer_label(host_application.as_ptr()),
            super::interface_pointer_label(host_context.as_ptr()),
            result,
        ),
        None => format!(
            "Configured factory host context host_application_ptr={} host_context_ptr={} set_host_context_result=factory3_unavailable",
            super::interface_pointer_label(host_application.as_ptr()),
            super::interface_pointer_label(host_context.as_ptr()),
        ),
    };
    telemetry.log_phase_event(
        PluginEditorHostValidationPhase::HostContext,
        "info",
        &factory_host_context_message,
    );
    telemetry.record_completed(
        PluginEditorHostValidationPhase::HostContext,
        "Created the host context and configured the VST3 factory.",
    );

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ComponentCreate,
        "About to create the VST3 component instance.",
    );
    let component = create_component(factory, plugin_path).map_err(|error| {
        record_phase_failure(
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

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ComponentInitialize,
        "About to initialize the VST3 component (MuseScore/PlugProvider order: component init → controller init → connect → setComponentHandler; NO runtime setup during init).",
    );
    let component_init = unsafe { component.initialize(host_context.as_ptr()) };
    if component_init != kResultOk {
        return Err(record_phase_failure(
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
    telemetry.record_completed(
        PluginEditorHostValidationPhase::ComponentInitialize,
        "Initialized the VST3 component.",
    );

    // MuseScore / Steinberg PlugProvider pattern: NO component runtime setup
    // (bus activation, setupProcessing, setActive) during initialization.
    // Audio setup is deferred to ensure_render_processing when playback starts.
    telemetry.record_skipped(
        PluginEditorHostValidationPhase::ComponentRuntimeSetup,
        "Skipped — runtime bus activation and processing setup is deferred to first render pass (MuseScore/PlugProvider pattern).",
    );

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ControllerCreate,
        "About to create the VST3 edit controller.",
    );
    let CreatedController {
        controller,
        shared_with_component: controller_shared_with_component,
    } = match create_controller(factory, &component) {
        Ok(controller) => controller,
        Err(error) => {
            unsafe {
                let _ = component.terminate();
            }
            return Err(record_phase_failure(
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

    telemetry.record_skipped(
        PluginEditorHostValidationPhase::ControllerSetComponentHandlerPreInit,
        "Skipped because the shared host core now follows Steinberg PlugProvider order and MuseScore's provider wiring: the controller initializes with the host context first, and only then receives the component handler.",
    );

    if controller_shared_with_component {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ControllerInitialize,
            "Skipped because the edit controller is shared with the component instance.",
        );
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ConnectionPointHandshake,
            "Skipped because the component and controller share one instance.",
        );
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ControllerSetComponentHandler,
            "Skipped because the edit controller is shared with the component instance.",
        );

        return Ok(SteinbergPlugProviderSetup {
            host_application,
            component_handler,
            component,
            controller,
            controller_shared_with_component,
            controller_initialized: false,
            connection_points_connected: false,
            component_connection_proxy: None,
            controller_connection_proxy: None,
        });
    }

    let controller_pointer_label = super::interface_pointer_label(controller.as_ptr());
    let host_context_label = super::interface_pointer_label(host_context.as_ptr());
    let host_application_label = super::interface_pointer_label(host_application.as_ptr());
    let component_handler_pointer_label =
        super::interface_pointer_label(component_handler.as_ptr());
    telemetry.record_starting(
        PluginEditorHostValidationPhase::ControllerInitialize,
        "About to initialize the VST3 edit controller.",
    );
    let controller_initialize_details = format!(
        "Controller initialize pointers controller_ptr={} host_context_ptr={} host_application_ptr={} component_handler_ptr={}",
        controller_pointer_label,
        host_context_label,
        host_application_label,
        component_handler_pointer_label,
    );
    telemetry.log_phase_event(
        PluginEditorHostValidationPhase::ControllerInitialize,
        "info",
        &controller_initialize_details,
    );
    let controller_init = unsafe { controller.initialize(host_context.as_ptr()) };
    let controller_initialize_return = format!(
        "controller.initialize returned tresult={} controller_ptr={}",
        controller_init, controller_pointer_label,
    );
    telemetry.log_phase_event(
        PluginEditorHostValidationPhase::ControllerInitialize,
        "info",
        &controller_initialize_return,
    );
    if controller_init != kResultOk {
        unsafe {
            let _ = component.terminate();
        }
        return Err(record_phase_failure(
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

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ConnectionPointHandshake,
        "About to connect the VST3 component and controller connection points via ConnectionProxy (MuseScore/PlugProvider pattern).",
    );
    let (connection_points_connected, component_connection_proxy, controller_connection_proxy) =
        match connect_component_and_controller(&component, &controller) {
            ConnectionPointHandshakeOutcome::Connected {
                message,
                component_proxy,
                controller_proxy,
            } => {
                telemetry.record_completed(
                    PluginEditorHostValidationPhase::ConnectionPointHandshake,
                    &message,
                );
                (true, Some(component_proxy), Some(controller_proxy))
            }
            ConnectionPointHandshakeOutcome::Skipped(message) => {
                telemetry.record_skipped(
                    PluginEditorHostValidationPhase::ConnectionPointHandshake,
                    &message,
                );
                (false, None, None)
            }
        };

    telemetry.record_starting(
        PluginEditorHostValidationPhase::ControllerSetComponentHandler,
        "About to set the VST3 component handler after controller initialization and connection-point setup.",
    );
    let set_component_handler_result =
        unsafe { controller.setComponentHandler(component_handler.as_ptr()) };
    let set_component_handler_message = format!(
        "controller.setComponentHandler (post-connection) returned tresult={} controller_ptr={} component_handler_ptr={}",
        set_component_handler_result,
        super::interface_pointer_label(controller.as_ptr()),
        super::interface_pointer_label(component_handler.as_ptr()),
    );
    telemetry.log_phase_event(
        PluginEditorHostValidationPhase::ControllerSetComponentHandler,
        if set_component_handler_result == kResultOk {
            "completed"
        } else {
            "failed"
        },
        &set_component_handler_message,
    );

    Ok(SteinbergPlugProviderSetup {
        host_application,
        component_handler,
        component,
        controller,
        controller_shared_with_component,
        controller_initialized: true,
        connection_points_connected,
        component_connection_proxy,
        controller_connection_proxy,
    })
}

pub(crate) struct PluginHostCoreSession {
    pub(super) target: PluginEditorTarget,
    pub(super) plugin_path: PathBuf,
    pub(super) restorable_plugin_state: Option<PlaybackSlotPluginState>,
    pub(super) _module: LoadedVst3Module,
    pub(super) component: ComPtr<IComponent>,
    pub(super) controller: ComPtr<IEditController>,
    pub(super) view: Option<ComPtr<IPlugView>>,
    pub(super) _host_application: PluginHostApplicationHandle,
    pub(super) _component_handler: ComPtr<IComponentHandler>,
    pub(super) _plug_frame: Option<ComPtr<IPlugFrame>>,
    pub(super) host_view_ptr: usize,
    pub(super) plugin_can_resize: bool,
    pub(super) view_attached: bool,
    pub(super) connection_points_connected: bool,
    pub(super) _component_connection_proxy: Option<ComPtr<IConnectionPoint>>,
    pub(super) _controller_connection_proxy: Option<ComPtr<IConnectionPoint>>,
    pub(super) component_initialized: bool,
    pub(super) controller_initialized: bool,
    pub(super) controller_shared_with_component: bool,
    pub(super) close_prepared: bool,
    pub(super) render_processing_active: bool,
    pub(super) render_component_active: bool,
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
        telemetry: &mut PluginHostPhaseLogger,
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
            record_phase_failure(
                telemetry,
                PluginEditorHostAttemptError::host_contract_failure(
                    PluginEditorHostValidationPhase::ViewCreate,
                    "Failed to allocate the VST3 plug-frame shim.",
                ),
            )
        })?;

        let view = unsafe { ComPtr::from_raw(self.controller.createView(ViewType::kEditor)) }
            .ok_or_else(|| {
                record_phase_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ViewCreate,
                        "Plugin editor POC did not receive a plugin editor view. The selected VST3 may not expose an editor.",
                    ),
                )
            })?;

        let platform_supported = unsafe { view.isPlatformTypeSupported(kPlatformTypeNSView) };
        if platform_supported != kResultOk {
            return Err(record_phase_failure(
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
            return Err(record_phase_failure(
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
                        super::disconnect_connection_proxies(
                            &self.component,
                            &self.controller,
                            &self._component_connection_proxy,
                            &self._controller_connection_proxy,
                        );
                    }
                    self._component_connection_proxy = None;
                    self._controller_connection_proxy = None;
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

pub(super) fn build_plugin_host_core_session_with_logger(
    target: &PluginEditorTarget,
    restorable_plugin_state: Option<&PlaybackSlotPluginState>,
    probe_profile: PluginEditorPocProbeProfile,
    telemetry: &mut PluginHostPhaseLogger,
) -> Result<PluginHostCoreSession, PluginEditorHostAttemptError> {
    let lifecycle_profile = Vst3HostCoreLifecycleProfile::from_probe_profile(probe_profile);
    let plugin_path = target.path();
    let host_application = create_plugin_host_application().map_err(|error| {
        PluginEditorHostAttemptError::host_contract_failure(
            PluginEditorHostValidationPhase::HostContext,
            error,
        )
    })?;
    telemetry.record_starting(
        PluginEditorHostValidationPhase::ModuleLoad,
        "About to load the VST3 module binary.",
    );
    let module = load_vst3_module(plugin_path).map_err(|error| {
        record_phase_failure(
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
        record_phase_failure(
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

    let SteinbergPlugProviderSetup {
        host_application,
        component_handler,
        component,
        controller,
        controller_shared_with_component,
        controller_initialized,
        connection_points_connected,
        component_connection_proxy,
        controller_connection_proxy,
    } = initialize_steinberg_plug_provider(
        plugin_path,
        host_application,
        &factory,
        lifecycle_profile,
        telemetry,
    )?;

    if let Some(plugin_state) = restorable_plugin_state {
        telemetry.record_starting(
            PluginEditorHostValidationPhase::ComponentStateSync,
            if controller_shared_with_component {
                "About to restore the portable component state after prior-art provider initialization."
            } else {
                "About to restore the portable component state after prior-art provider initialization and mirror it into the controller via setComponentState."
            },
        );
        let component_state_bytes =
            restore_component_state(&component, plugin_state).map_err(|error| {
                record_phase_failure(
                    telemetry,
                    PluginEditorHostAttemptError::plugin_failure(
                        PluginEditorHostValidationPhase::ComponentStateSync,
                        error,
                    ),
                )
            })?;
        if !controller_shared_with_component {
            sync_component_state_to_controller(&controller, &component_state_bytes).map_err(
                |error| {
                    record_phase_failure(
                        telemetry,
                        PluginEditorHostAttemptError::plugin_failure(
                            PluginEditorHostValidationPhase::ComponentStateSync,
                            error,
                        ),
                    )
                },
            )?;
        }
        telemetry.record_completed(
            PluginEditorHostValidationPhase::ComponentStateSync,
            if controller_shared_with_component {
                "Restored the portable component state after provider initialization."
            } else {
                "Restored the portable component state after provider initialization and mirrored it into the controller via setComponentState."
            },
        );
    } else {
        telemetry.record_skipped(
            PluginEditorHostValidationPhase::ComponentStateSync,
            "Skipped because no portable component state was provided.",
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
        _component_handler: component_handler,
        _plug_frame: None,
        host_view_ptr: 0,
        plugin_can_resize: false,
        view_attached: false,
        connection_points_connected,
        _component_connection_proxy: component_connection_proxy,
        _controller_connection_proxy: controller_connection_proxy,
        component_initialized: true,
        controller_initialized,
        controller_shared_with_component,
        close_prepared: false,
        render_processing_active: false,
        render_component_active: false,
        render_sample_rate: None,
        render_max_block_size: 0,
        render_buses_activated: false,
        render_output_buses: Vec::new(),
        render_scratch: RenderScratch::default(),
    })
}
