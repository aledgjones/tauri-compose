use super::get_vertical_spacing::VerticalSpacing;
use super::get_vertical_spans::VerticalSpans;
use crate::renderer::converter::Space;
use crate::score::engraves::Engrave;

pub fn get_max_brackets_width(
    vertical_spacing: &VerticalSpacing,
    spans: &VerticalSpans,
    engrave: &Engrave,
) -> Space {
    let mut max: Space = 0.0;

    // BRACKET of fixed width 1.0
    for bracket in &spans.brackets {
        // we need at least on instrument to bracket single staves
        if engrave.bracket_single_staves || bracket.stop != bracket.start {
            max = 1.0;
            break;
        }
    }

    // SUB-BRACKETS of fixed width .5
    if !spans.sub_brackets.is_empty() {
        max += 0.5;
    }

    // BRACES of variable widths
    for brace in &spans.braces {
        let top = &vertical_spacing.staves[&brace.start];
        let bottom = &vertical_spacing.staves[&brace.stop];
        let height: Space = (bottom.y + bottom.height) - top.y;
        let width = height * 0.1;
        if width > max {
            max = width;
        }
    }

    max
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::parse::get_vertical_spacing::{VerticalPosition, VerticalSpacing};
    use crate::instructions::parse::get_vertical_spans::{VerticalSpan, VerticalSpans};
    use crate::score::engraves::Engraves;
    use crate::score::layouts::LayoutType;
    use rustc_hash::FxHashMap;

    fn empty_spans() -> VerticalSpans {
        VerticalSpans {
            brackets: Vec::new(),
            sub_brackets: Vec::new(),
            braces: Vec::new(),
            barlines: Vec::new(),
        }
    }

    fn empty_spacing() -> VerticalSpacing {
        VerticalSpacing {
            height: 10.0,
            instruments: FxHashMap::default(),
            staves: FxHashMap::default(),
        }
    }

    #[test]
    fn no_spans_returns_zero() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let result = get_max_brackets_width(&empty_spacing(), &empty_spans(), engrave);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn single_bracket_spanning_multiple_instruments_returns_one() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let mut spans = empty_spans();
        spans.brackets.push(VerticalSpan { start: 1, stop: 2 });
        let result = get_max_brackets_width(&empty_spacing(), &spans, engrave);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn single_bracket_same_start_stop_without_bracket_singles_returns_zero() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        // FullScore uses bracket_single_staves = false
        let mut spans = empty_spans();
        spans.brackets.push(VerticalSpan { start: 1, stop: 1 }); // same = single stave
                                                                 // bracket_single_staves is false by default → max stays 0
        let result = get_max_brackets_width(&empty_spacing(), &spans, engrave);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn sub_bracket_adds_half_to_max() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let mut spans = empty_spans();
        spans.sub_brackets.push(VerticalSpan { start: 1, stop: 2 });
        let result = get_max_brackets_width(&empty_spacing(), &spans, engrave);
        assert_eq!(result, 0.5);
    }

    #[test]
    fn brace_contributes_proportional_width() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let mut spans = empty_spans();
        spans.braces.push(VerticalSpan {
            start: 10,
            stop: 11,
        });

        let mut spacing = empty_spacing();
        spacing.staves.insert(
            10,
            VerticalPosition {
                y: 0.0,
                height: 5.0,
            },
        );
        spacing.staves.insert(
            11,
            VerticalPosition {
                y: 15.0,
                height: 5.0,
            },
        );

        // height = (15.0 + 5.0) - 0.0 = 20.0, width = 20.0 * 0.1 = 2.0
        let result = get_max_brackets_width(&spacing, &spans, engrave);
        assert!((result - 2.0).abs() < 1e-6);
    }

    #[test]
    fn bracket_and_sub_bracket_combined() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let mut spans = empty_spans();
        spans.brackets.push(VerticalSpan { start: 1, stop: 2 }); // adds 1.0
        spans.sub_brackets.push(VerticalSpan { start: 1, stop: 2 }); // adds 0.5
        let result = get_max_brackets_width(&empty_spacing(), &spans, engrave);
        assert_eq!(result, 1.5);
    }
}
