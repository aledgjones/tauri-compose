use crate::score::Score;
use crate::utils::UniqueId;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlowRenderRecalculationRequest {
    FullVisibleFlows,
    Flows(Vec<UniqueId>),
}

impl FlowRenderRecalculationRequest {
    pub fn full_visible_flows() -> Self {
        Self::FullVisibleFlows
    }

    pub fn for_flow_ids(flow_ids: &[UniqueId]) -> Option<Self> {
        (!flow_ids.is_empty()).then(|| Self::Flows(flow_ids.to_vec()))
    }

    pub fn for_local_update(
        previous_flow_id: Option<UniqueId>,
        next_flow_id: Option<UniqueId>,
    ) -> Option<Self> {
        let mut flow_ids = Vec::new();

        if let Some(flow_id) = previous_flow_id {
            flow_ids.push(flow_id);
        }

        if let Some(flow_id) = next_flow_id {
            if !flow_ids.contains(&flow_id) {
                flow_ids.push(flow_id);
            }
        }

        Self::for_flow_ids(&flow_ids)
    }

    fn resolve_flow_ids(self, prepared_flows: &[PreparedFlowRenderInput]) -> Vec<UniqueId> {
        let requested_flow_ids = match self {
            Self::FullVisibleFlows => {
                return prepared_flows.iter().map(|flow| flow.flow_id).collect()
            }
            Self::Flows(flow_ids) => flow_ids,
        };

        prepared_flows
            .iter()
            .filter(|flow| requested_flow_ids.contains(&flow.flow_id))
            .map(|flow| flow.flow_id)
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowRenderMode {
    SharedSurface,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlowRenderScope {
    NoVisibleFlows,
    SingleFlow,
    MultipleFlows,
}

impl FlowRenderScope {
    fn from_flow_count(flow_count: usize) -> Self {
        match flow_count {
            0 => Self::NoVisibleFlows,
            1 => Self::SingleFlow,
            _ => Self::MultipleFlows,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedFlowRenderInput {
    pub flow_id: UniqueId,
    pub order_index: usize,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowRenderPlan {
    pub layout_id: UniqueId,
    pub active_flow_id: UniqueId,
    pub active_flow_visible_in_layout: bool,
    pub prepared_flows: Vec<PreparedFlowRenderInput>,
    pub recalculated_flow_ids: Vec<UniqueId>,
}

impl FlowRenderPlan {
    pub fn render_mode(&self) -> FlowRenderMode {
        FlowRenderMode::SharedSurface
    }

    pub fn prepared_flow_ids(&self) -> Vec<UniqueId> {
        self.prepared_flows
            .iter()
            .map(|flow| flow.flow_id)
            .collect()
    }

    pub fn prepared_flow_count(&self) -> usize {
        self.prepared_flows.len()
    }

    pub fn prepared_flow_scope(&self) -> FlowRenderScope {
        FlowRenderScope::from_flow_count(self.prepared_flow_count())
    }

    pub fn composed_flow_ids(&self) -> Vec<UniqueId> {
        self.prepared_flow_ids()
    }

    pub fn composed_flow_count(&self) -> usize {
        self.composed_flow_ids().len()
    }

    pub fn composed_flow_scope(&self) -> FlowRenderScope {
        FlowRenderScope::from_flow_count(self.composed_flow_count())
    }

    pub fn recalculated_flow_ids(&self) -> Vec<UniqueId> {
        self.recalculated_flow_ids.clone()
    }

    pub fn recalculated_flow_count(&self) -> usize {
        self.recalculated_flow_ids().len()
    }

    pub fn recalculation_scope(&self) -> FlowRenderScope {
        FlowRenderScope::from_flow_count(self.recalculated_flow_count())
    }

    pub fn should_recalculate_flow(&self, flow_id: UniqueId) -> bool {
        self.recalculated_flow_ids.contains(&flow_id)
    }

    pub fn with_recalculated_flow_ids(&self, recalculated_flow_ids: Vec<UniqueId>) -> Self {
        let mut updated = self.clone();
        updated.recalculated_flow_ids = recalculated_flow_ids;
        updated
    }

    pub fn with_all_prepared_flows_recalculated(&self) -> Self {
        self.with_recalculated_flow_ids(self.prepared_flow_ids())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FlowRenderPreparationDiagnostics {
    pub render_mode: FlowRenderMode,
    pub layout_id: UniqueId,
    pub active_flow_id: UniqueId,
    pub active_flow_visible_in_layout: bool,
    pub prepared_flow_ids: Vec<UniqueId>,
    pub prepared_flow_count: usize,
    pub prepared_flow_scope: FlowRenderScope,
    pub composed_flow_ids: Vec<UniqueId>,
    pub composed_flow_count: usize,
    pub composed_flow_scope: FlowRenderScope,
    pub recalculated_flow_ids: Vec<UniqueId>,
    pub recalculated_flow_count: usize,
    pub recalculation_scope: FlowRenderScope,
}

impl From<&FlowRenderPlan> for FlowRenderPreparationDiagnostics {
    fn from(plan: &FlowRenderPlan) -> Self {
        Self {
            render_mode: plan.render_mode(),
            layout_id: plan.layout_id,
            active_flow_id: plan.active_flow_id,
            active_flow_visible_in_layout: plan.active_flow_visible_in_layout,
            prepared_flow_ids: plan.prepared_flow_ids(),
            prepared_flow_count: plan.prepared_flow_count(),
            prepared_flow_scope: plan.prepared_flow_scope(),
            composed_flow_ids: plan.composed_flow_ids(),
            composed_flow_count: plan.composed_flow_count(),
            composed_flow_scope: plan.composed_flow_scope(),
            recalculated_flow_ids: plan.recalculated_flow_ids(),
            recalculated_flow_count: plan.recalculated_flow_count(),
            recalculation_scope: plan.recalculation_scope(),
        }
    }
}

pub fn prepare_flow_render_plan(
    score: &Score,
    layout_id: UniqueId,
    active_flow_id: UniqueId,
    recalculation_request: FlowRenderRecalculationRequest,
) -> Option<FlowRenderPlan> {
    let layout = score.layouts.by_id.get(&layout_id)?;
    score.flows.by_id.get(&active_flow_id)?;
    let active_flow_visible_in_layout = layout.flow_ids.contains(&active_flow_id);

    let prepared_flows = score
        .flows
        .order
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(order_index, flow_id)| {
            if !layout.flow_ids.contains(&flow_id) || !score.flows.by_id.contains_key(&flow_id) {
                return None;
            }

            Some(PreparedFlowRenderInput {
                flow_id,
                order_index,
                is_active: flow_id == active_flow_id,
            })
        })
        .collect::<Vec<_>>();

    let recalculated_flow_ids = recalculation_request.resolve_flow_ids(&prepared_flows);

    Some(FlowRenderPlan {
        layout_id,
        active_flow_id,
        active_flow_visible_in_layout,
        prepared_flows,
        recalculated_flow_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::flows::Flow;
    use crate::score::layouts::LayoutType;
    use crate::score::tracks::Track;

    #[test]
    fn local_update_request_keeps_same_flow_requests_narrow() {
        assert_eq!(
            FlowRenderRecalculationRequest::for_local_update(Some(11), Some(11)),
            Some(FlowRenderRecalculationRequest::Flows(vec![11]))
        );
    }

    #[test]
    fn local_update_request_recalculates_previous_and_next_flows_in_order() {
        assert_eq!(
            FlowRenderRecalculationRequest::for_local_update(Some(11), Some(22)),
            Some(FlowRenderRecalculationRequest::Flows(vec![11, 22]))
        );
    }

    #[test]
    fn local_update_request_keeps_clear_requests_to_the_previous_flow() {
        assert_eq!(
            FlowRenderRecalculationRequest::for_local_update(Some(11), None),
            Some(FlowRenderRecalculationRequest::Flows(vec![11]))
        );
    }

    #[test]
    fn local_update_request_returns_none_when_no_flows_change() {
        assert_eq!(
            FlowRenderRecalculationRequest::for_local_update(None, None),
            None
        );
    }

    fn prepare_plan(
        score: &Score,
        layout_id: UniqueId,
        active_flow_id: UniqueId,
    ) -> FlowRenderPlan {
        prepare_flow_render_plan(
            score,
            layout_id,
            active_flow_id,
            FlowRenderRecalculationRequest::full_visible_flows(),
        )
        .unwrap()
    }

    fn build_score() -> Score {
        let mut score = Score::new();

        score.flows.order = vec![10, 20, 30, 40];
        score.flows.by_id.insert(10, Flow::new(10, 100));
        score.flows.by_id.insert(20, Flow::new(20, 200));
        score.flows.by_id.insert(30, Flow::new(30, 300));
        score.flows.by_id.insert(40, Flow::new(40, 400));
        score.tracks.by_id.insert(100, Track::new(100));
        score.tracks.by_id.insert(200, Track::new(200));
        score.tracks.by_id.insert(300, Track::new(300));
        score.tracks.by_id.insert(400, Track::new(400));

        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        *layout = crate::score::layouts::Layout::new(0, LayoutType::FullScore);
        layout.flow_ids.insert(30);
        layout.flow_ids.insert(10);

        score
    }

    #[test]
    fn prepare_flow_render_plan_preserves_score_flow_order_for_visible_flows() {
        let score = build_score();

        let plan = prepare_plan(&score, 0, 30);

        assert_eq!(plan.prepared_flow_ids(), vec![10, 30]);
        assert_eq!(plan.composed_flow_ids(), vec![10, 30]);
    }

    #[test]
    fn prepare_flow_render_plan_marks_active_flow_without_reordering_it_to_front() {
        let score = build_score();

        let plan = prepare_plan(&score, 0, 30);

        assert!(plan.active_flow_visible_in_layout);
        assert_eq!(plan.prepared_flows[0].flow_id, 10);
        assert!(!plan.prepared_flows[0].is_active);
        assert_eq!(plan.prepared_flows[1].flow_id, 30);
        assert!(plan.prepared_flows[1].is_active);
    }

    #[test]
    fn prepare_flow_render_plan_keeps_visible_flow_order_when_active_flow_is_hidden() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.remove(&30);

        let plan = prepare_plan(&score, 0, 30);

        assert!(!plan.active_flow_visible_in_layout);
        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert_eq!(plan.composed_flow_ids(), vec![10]);
    }

    #[test]
    fn prepare_flow_render_plan_returns_empty_composition_when_layout_has_no_visible_flows() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();

        let plan = prepare_plan(&score, 0, 10);

        assert!(!plan.active_flow_visible_in_layout);
        assert!(plan.prepared_flows.is_empty());
        assert!(plan.composed_flow_ids().is_empty());
    }

    #[test]
    fn diagnostics_report_prepared_and_composed_flow_ids() {
        let score = build_score();

        let plan = prepare_plan(&score, 0, 30);
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.render_mode, FlowRenderMode::SharedSurface);
        assert_eq!(diagnostics.layout_id, 0);
        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.prepared_flow_count, 2);
        assert_eq!(
            diagnostics.prepared_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.composed_flow_count, 2);
        assert_eq!(
            diagnostics.composed_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.recalculated_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.recalculated_flow_count, 2);
        assert_eq!(
            diagnostics.recalculation_scope,
            FlowRenderScope::MultipleFlows
        );
    }

    #[test]
    fn prepare_flow_render_plan_composes_all_visible_flows_in_score_order() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.insert(20);
        layout.flow_ids.insert(40);

        let plan = prepare_plan(&score, 0, 30);

        assert_eq!(plan.prepared_flow_ids(), vec![10, 20, 30, 40]);
        assert_eq!(plan.composed_flow_ids(), vec![10, 20, 30, 40]);
        assert_eq!(plan.prepared_flow_count(), 4);
        assert_eq!(plan.composed_flow_count(), 4);
    }

    #[test]
    fn prepare_flow_render_plan_composes_single_visible_flow_without_special_fallback() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();
        layout.flow_ids.insert(10);

        let plan = prepare_plan(&score, 0, 10);

        assert!(plan.active_flow_visible_in_layout);
        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert_eq!(plan.composed_flow_ids(), vec![10]);
    }

    #[test]
    fn diagnostics_report_single_flow_scope_for_narrow_passes() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();
        layout.flow_ids.insert(10);

        let plan = prepare_plan(&score, 0, 10);
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.prepared_flow_scope, FlowRenderScope::SingleFlow);
        assert_eq!(diagnostics.composed_flow_scope, FlowRenderScope::SingleFlow);
        assert_eq!(diagnostics.recalculated_flow_ids, vec![10]);
        assert_eq!(diagnostics.recalculated_flow_count, 1);
        assert_eq!(diagnostics.recalculation_scope, FlowRenderScope::SingleFlow);
    }

    #[test]
    fn diagnostics_report_no_visible_flow_scope_for_empty_passes() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();

        let plan = prepare_plan(&score, 0, 10);
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.prepared_flow_ids, Vec::<UniqueId>::new());
        assert_eq!(
            diagnostics.prepared_flow_scope,
            FlowRenderScope::NoVisibleFlows
        );
        assert_eq!(
            diagnostics.composed_flow_scope,
            FlowRenderScope::NoVisibleFlows
        );
        assert_eq!(diagnostics.recalculated_flow_ids, Vec::<UniqueId>::new());
        assert_eq!(
            diagnostics.recalculation_scope,
            FlowRenderScope::NoVisibleFlows
        );
    }

    #[test]
    fn diagnostics_report_all_composed_flow_ids_without_a_two_flow_cap() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.insert(20);
        layout.flow_ids.insert(40);

        let plan = prepare_plan(&score, 0, 30);
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.prepared_flow_count, 4);
        assert_eq!(
            diagnostics.prepared_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.composed_flow_count, 4);
        assert_eq!(
            diagnostics.composed_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.recalculated_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.recalculated_flow_count, 4);
        assert_eq!(
            diagnostics.recalculation_scope,
            FlowRenderScope::MultipleFlows
        );
    }

    #[test]
    fn prepare_flow_render_plan_returns_none_when_active_flow_is_missing() {
        let score = build_score();

        let plan = prepare_flow_render_plan(
            &score,
            0,
            999,
            FlowRenderRecalculationRequest::full_visible_flows(),
        );

        assert!(plan.is_none());
    }

    #[test]
    fn prepare_flow_render_plan_reports_only_targeted_visible_flow_as_recalculated() {
        let score = build_score();

        let plan = prepare_flow_render_plan(
            &score,
            0,
            30,
            FlowRenderRecalculationRequest::Flows(vec![30]),
        )
        .unwrap();

        assert_eq!(plan.prepared_flow_ids(), vec![10, 30]);
        assert_eq!(plan.composed_flow_ids(), vec![10, 30]);
        assert_eq!(plan.recalculated_flow_ids(), vec![30]);
        assert_eq!(plan.recalculation_scope(), FlowRenderScope::SingleFlow);
        assert!(plan.should_recalculate_flow(30));
        assert!(!plan.should_recalculate_flow(10));
    }

    #[test]
    fn prepare_flow_render_plan_reports_no_recalculated_visible_flows_for_hidden_target() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.remove(&30);

        let plan = prepare_flow_render_plan(
            &score,
            0,
            30,
            FlowRenderRecalculationRequest::Flows(vec![30]),
        )
        .unwrap();

        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert!(plan.recalculated_flow_ids().is_empty());
        assert_eq!(plan.recalculation_scope(), FlowRenderScope::NoVisibleFlows);
    }

    #[test]
    fn prepare_flow_render_plan_reports_only_requested_visible_flows_for_multi_flow_targets() {
        let score = build_score();

        let plan = prepare_flow_render_plan(
            &score,
            0,
            30,
            FlowRenderRecalculationRequest::Flows(vec![30, 10, 999]),
        )
        .unwrap();

        assert_eq!(plan.prepared_flow_ids(), vec![10, 30]);
        assert_eq!(plan.recalculated_flow_ids(), vec![10, 30]);
        assert_eq!(plan.recalculation_scope(), FlowRenderScope::MultipleFlows);
        assert!(plan.should_recalculate_flow(10));
        assert!(plan.should_recalculate_flow(30));
    }

    #[test]
    fn diagnostics_distinguish_single_flow_local_updates_inside_multi_flow_layouts() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.insert(20);
        layout.flow_ids.insert(40);

        let plan = prepare_flow_render_plan(
            &score,
            0,
            30,
            FlowRenderRecalculationRequest::Flows(vec![20]),
        )
        .unwrap();
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.prepared_flow_count, 4);
        assert_eq!(
            diagnostics.prepared_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.composed_flow_count, 4);
        assert_eq!(
            diagnostics.composed_flow_scope,
            FlowRenderScope::MultipleFlows
        );
        assert_eq!(diagnostics.recalculated_flow_ids, vec![20]);
        assert_eq!(diagnostics.recalculated_flow_count, 1);
        assert_eq!(diagnostics.recalculation_scope, FlowRenderScope::SingleFlow);
    }

    #[test]
    fn for_flow_ids_returns_none_for_empty_requests() {
        assert_eq!(FlowRenderRecalculationRequest::for_flow_ids(&[]), None);
    }
}
