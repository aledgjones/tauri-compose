use crate::score::Score;
use crate::utils::UniqueId;
use serde::Serialize;

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
}

impl FlowRenderPlan {
    pub fn prepared_flow_ids(&self) -> Vec<UniqueId> {
        self.prepared_flows
            .iter()
            .map(|flow| flow.flow_id)
            .collect()
    }

    pub fn prepared_flow_count(&self) -> usize {
        self.prepared_flows.len()
    }

    pub fn composed_flow_ids(&self) -> Vec<UniqueId> {
        self.prepared_flow_ids()
    }

    pub fn composed_flow_count(&self) -> usize {
        self.composed_flow_ids().len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FlowRenderPreparationDiagnostics {
    pub layout_id: UniqueId,
    pub active_flow_id: UniqueId,
    pub active_flow_visible_in_layout: bool,
    pub prepared_flow_ids: Vec<UniqueId>,
    pub prepared_flow_count: usize,
    pub composed_flow_ids: Vec<UniqueId>,
    pub composed_flow_count: usize,
}

impl From<&FlowRenderPlan> for FlowRenderPreparationDiagnostics {
    fn from(plan: &FlowRenderPlan) -> Self {
        Self {
            layout_id: plan.layout_id,
            active_flow_id: plan.active_flow_id,
            active_flow_visible_in_layout: plan.active_flow_visible_in_layout,
            prepared_flow_ids: plan.prepared_flow_ids(),
            prepared_flow_count: plan.prepared_flow_count(),
            composed_flow_ids: plan.composed_flow_ids(),
            composed_flow_count: plan.composed_flow_count(),
        }
    }
}

pub fn prepare_flow_render_plan(
    score: &Score,
    layout_id: UniqueId,
    active_flow_id: UniqueId,
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

    Some(FlowRenderPlan {
        layout_id,
        active_flow_id,
        active_flow_visible_in_layout,
        prepared_flows,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::flows::Flow;
    use crate::score::layouts::LayoutType;
    use crate::score::tracks::Track;

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

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();

        assert_eq!(plan.prepared_flow_ids(), vec![10, 30]);
        assert_eq!(plan.composed_flow_ids(), vec![10, 30]);
    }

    #[test]
    fn prepare_flow_render_plan_marks_active_flow_without_reordering_it_to_front() {
        let score = build_score();

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();

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

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();

        assert!(!plan.active_flow_visible_in_layout);
        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert_eq!(plan.composed_flow_ids(), vec![10]);
    }

    #[test]
    fn prepare_flow_render_plan_returns_empty_composition_when_layout_has_no_visible_flows() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();

        let plan = prepare_flow_render_plan(&score, 0, 10).unwrap();

        assert!(!plan.active_flow_visible_in_layout);
        assert!(plan.prepared_flows.is_empty());
        assert!(plan.composed_flow_ids().is_empty());
    }

    #[test]
    fn diagnostics_report_prepared_and_composed_flow_ids() {
        let score = build_score();

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.layout_id, 0);
        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.prepared_flow_count, 2);
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.composed_flow_count, 2);
    }

    #[test]
    fn prepare_flow_render_plan_composes_all_visible_flows_in_score_order() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.insert(20);
        layout.flow_ids.insert(40);

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();

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

        let plan = prepare_flow_render_plan(&score, 0, 10).unwrap();

        assert!(plan.active_flow_visible_in_layout);
        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert_eq!(plan.composed_flow_ids(), vec![10]);
    }

    #[test]
    fn diagnostics_report_all_composed_flow_ids_without_a_two_flow_cap() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.insert(20);
        layout.flow_ids.insert(40);

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.prepared_flow_count, 4);
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 20, 30, 40]);
        assert_eq!(diagnostics.composed_flow_count, 4);
    }

    #[test]
    fn prepare_flow_render_plan_returns_none_when_active_flow_is_missing() {
        let score = build_score();

        let plan = prepare_flow_render_plan(&score, 0, 999);

        assert!(plan.is_none());
    }
}
