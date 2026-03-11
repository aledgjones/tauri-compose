use crate::score::Score;
use crate::utils::UniqueId;
use serde::Serialize;

const MAX_COMPOSED_VISIBLE_FLOWS: usize = 2;

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
    pub drawn_flow_id: UniqueId,
    pub prepared_flows: Vec<PreparedFlowRenderInput>,
    pub single_flow_draw_fallback: bool,
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
        if self.single_flow_draw_fallback {
            vec![self.drawn_flow_id]
        } else {
            self.prepared_flows
                .iter()
                .take(MAX_COMPOSED_VISIBLE_FLOWS)
                .map(|flow| flow.flow_id)
                .collect()
        }
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
    pub fallback_draw_flow_id: Option<UniqueId>,
    pub prepared_flow_ids: Vec<UniqueId>,
    pub prepared_flow_count: usize,
    pub composed_flow_ids: Vec<UniqueId>,
    pub composed_flow_count: usize,
    pub single_flow_draw_fallback: bool,
}

impl From<&FlowRenderPlan> for FlowRenderPreparationDiagnostics {
    fn from(plan: &FlowRenderPlan) -> Self {
        Self {
            layout_id: plan.layout_id,
            active_flow_id: plan.active_flow_id,
            active_flow_visible_in_layout: plan.active_flow_visible_in_layout,
            fallback_draw_flow_id: plan
                .single_flow_draw_fallback
                .then_some(plan.drawn_flow_id),
            prepared_flow_ids: plan.prepared_flow_ids(),
            prepared_flow_count: plan.prepared_flow_count(),
            composed_flow_ids: plan.composed_flow_ids(),
            composed_flow_count: plan.composed_flow_count(),
            single_flow_draw_fallback: plan.single_flow_draw_fallback,
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
        drawn_flow_id: active_flow_id,
        single_flow_draw_fallback: prepared_flows.len() < MAX_COMPOSED_VISIBLE_FLOWS,
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

        score.flows.order = vec![10, 20, 30];
        score.flows.by_id.insert(10, Flow::new(10, 100));
        score.flows.by_id.insert(20, Flow::new(20, 200));
        score.flows.by_id.insert(30, Flow::new(30, 300));
        score.tracks.by_id.insert(100, Track::new(100));
        score.tracks.by_id.insert(200, Track::new(200));
        score.tracks.by_id.insert(300, Track::new(300));

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
        assert_eq!(plan.drawn_flow_id, 30);
        assert!(!plan.single_flow_draw_fallback);
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
    fn prepare_flow_render_plan_keeps_active_draw_target_when_active_flow_is_hidden() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.remove(&30);

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();

        assert!(!plan.active_flow_visible_in_layout);
        assert_eq!(plan.drawn_flow_id, 30);
        assert_eq!(plan.prepared_flow_ids(), vec![10]);
        assert_eq!(plan.composed_flow_ids(), vec![30]);
        assert!(plan.single_flow_draw_fallback);
    }

    #[test]
    fn prepare_flow_render_plan_preserves_active_flow_render_when_layout_has_no_visible_flows() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.clear();

        let plan = prepare_flow_render_plan(&score, 0, 10).unwrap();

        assert!(!plan.active_flow_visible_in_layout);
        assert_eq!(plan.drawn_flow_id, 10);
        assert!(plan.prepared_flows.is_empty());
        assert_eq!(plan.composed_flow_ids(), vec![10]);
        assert!(plan.single_flow_draw_fallback);
    }

    #[test]
    fn diagnostics_report_prepared_and_composed_flow_ids() {
        let score = build_score();

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.fallback_draw_flow_id, None);
        assert_eq!(diagnostics.prepared_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.prepared_flow_count, 2);
        assert_eq!(diagnostics.composed_flow_ids, vec![10, 30]);
        assert_eq!(diagnostics.composed_flow_count, 2);
        assert!(!diagnostics.single_flow_draw_fallback);
    }

    #[test]
    fn diagnostics_report_fallback_draw_flow_only_in_single_flow_mode() {
        let mut score = build_score();
        let layout = score.layouts.by_id.get_mut(&0).unwrap();
        layout.flow_ids.remove(&30);

        let plan = prepare_flow_render_plan(&score, 0, 30).unwrap();
        let diagnostics = FlowRenderPreparationDiagnostics::from(&plan);

        assert_eq!(diagnostics.fallback_draw_flow_id, Some(30));
        assert!(diagnostics.single_flow_draw_fallback);
    }

    #[test]
    fn prepare_flow_render_plan_returns_none_when_active_flow_is_missing() {
        let score = build_score();

        let plan = prepare_flow_render_plan(&score, 0, 999);

        assert!(plan.is_none());
    }
}
