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
