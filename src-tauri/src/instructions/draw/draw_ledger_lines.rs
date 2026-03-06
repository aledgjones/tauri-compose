use crate::instructions::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::instructions::parse::get_accidentals::Accidentals;
use crate::instructions::parse::get_shunts::{Shunt, Shunts};
use crate::instructions::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::instructions::parse::get_written_durations::Notation;
use crate::renderer::converter::Space;
use crate::renderer::RendererContext;
use crate::score::components::duration::Tick;
use crate::score::components::measurements::Point;
use crate::score::components::pitch::Accidental;
use rustc_hash::FxHashMap;
use skia_safe::Color;

struct ToneAffectingLedger {
    offset_amount: f32,
    required_slot: u8,
}

struct LedgerLineTweaks {
    tweaks: FxHashMap<i8, Vec<ToneAffectingLedger>>,
}

impl LedgerLineTweaks {
    fn new() -> Self {
        Self {
            tweaks: FxHashMap::default(),
        }
    }

    fn insert(&mut self, offset: i8, amount: f32, slot: u8) {
        self.tweaks
            .entry(offset)
            .or_default()
            .push(ToneAffectingLedger {
                offset_amount: amount,
                required_slot: slot,
            });
    }
}

fn draw_lines<T: Iterator<Item = i8>>(
    origin: &Point,
    range: T,
    tick: &Tick,
    glyph_width: Space,
    shunts: &Shunts,
    offset_tweaks: &FxHashMap<i8, Vec<ToneAffectingLedger>>,
    renderer: &mut RendererContext,
) {
    let notehead = glyph_width;

    let mut start_x = origin.x - 0.4;
    let mut stop_x = origin.x + 0.4;
    let mut has_pre_shunt_extended = false;

    for offset in range {
        // get the furthest start position
        if let Some(shunt) = shunts.by_offset.get(&(*tick, offset)) {
            match shunt {
                Shunt::Pre => {
                    let point = origin.x - notehead - 0.4;

                    if point < start_x {
                        start_x = point;
                    }

                    has_pre_shunt_extended = true;
                }
                Shunt::None => {
                    let point = origin.x + notehead + 0.4;

                    if point > stop_x {
                        stop_x = point;
                    }

                    // if the previous offset is pre shunt, extend the line 0.5 note heads
                    if let Some(Shunt::Pre) = shunts.by_offset.get(&(*tick, offset + 1)) {
                        let point = origin.x - (notehead * 0.5);

                        if point < start_x {
                            start_x = point;
                        }
                    }

                    // if the next offset is post shunt, extend the line 0.5 note heads
                    if let Some(Shunt::Post) = shunts.by_offset.get(&(*tick, offset - 1)) {
                        let point = origin.x + (notehead * 1.5);

                        if point > stop_x {
                            stop_x = point;
                        }
                    }
                }
                Shunt::Post => {
                    let point = origin.x + (notehead * 2.0) + 0.4;

                    if point > stop_x {
                        stop_x = point;
                    }
                }
            }
        };

        if offset % 2 == 0 {
            let y = origin.y + (offset as f32 / 2.0);

            // Apply tweaks from pre-computed map
            let mut accidental_offset: f32 = 0.0;
            let max_slot = if has_pre_shunt_extended { 2 } else { 1 };

            if let Some(affecting_tones) = offset_tweaks.get(&offset) {
                for tone_data in affecting_tones {
                    if tone_data.required_slot <= max_slot {
                        accidental_offset = accidental_offset.max(tone_data.offset_amount);
                    }
                }
            }

            let start = start_x + accidental_offset;
            let stop = stop_x;

            renderer.line(
                STAVE_LINE_WIDTH * 1.5,
                Color::BLACK,
                vec![Point { x: start, y }, Point { x: stop, y }],
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ledger_lines(
    origin: &Point,
    tick: &Tick,
    entry: &Notation,
    glyph_width: Space,
    shunts: &Shunts,
    accidentals: &Accidentals,
    tone_offsets: &ToneVerticalOffsets,
    renderer: &mut RendererContext,
) {
    let info = entry.info.as_ref().unwrap();
    let highest = info.highest;
    let lowest = info.lowest;

    let from = if highest > -5 { -5 } else { highest };
    let to = if lowest < 5 { 5 } else { lowest + 1 };

    // Pre-compute which tones affect which ledger line offsets
    let mut offset_tweaks = LedgerLineTweaks::new();

    for tone in entry.tones.iter() {
        if let Some(accidental) = accidentals.by_id.get(&(*tick, tone.id)) {
            // Only process accidentals that are drawn (slot > 0) and in slot 1 or 2
            if accidental.slot > 2 {
                continue;
            }

            let tone_offset = tone_offsets[&tone.id];

            // Insert ledger line tweaks based on accidental type and position
            match tone.pitch.accidental {
                Accidental::Sharp => {
                    if tone_offset % 2 == 0 {
                        // on line
                        offset_tweaks.insert(tone_offset, 0.3, accidental.slot);
                        offset_tweaks.insert(tone_offset + 2, 0.1, accidental.slot);
                    } else {
                        // in space
                        offset_tweaks.insert(tone_offset - 3, 0.05, accidental.slot);
                        offset_tweaks.insert(tone_offset - 1, 0.3, accidental.slot);
                        offset_tweaks.insert(tone_offset + 1, 0.3, accidental.slot);
                    }
                }
                Accidental::Flat => {
                    if tone_offset % 2 == 0 {
                        // on line
                        offset_tweaks.insert(tone_offset, 0.25, accidental.slot);
                    } else {
                        // in space
                        offset_tweaks.insert(tone_offset - 1, 0.25, accidental.slot);
                    }
                }
                Accidental::Natural => {
                    if tone_offset % 2 == 0 {
                        // on line
                        offset_tweaks.insert(tone_offset, 0.3, accidental.slot);
                        offset_tweaks.insert(tone_offset + 2, 0.3, accidental.slot);
                    } else {
                        // in space
                        offset_tweaks.insert(tone_offset - 1, 0.3, accidental.slot);
                        offset_tweaks.insert(tone_offset + 1, 0.3, accidental.slot);
                    }
                }
                _ => {}
            }
        }
    }

    // high ledger lines
    draw_lines(
        origin,
        from..-5,
        tick,
        glyph_width,
        shunts,
        &offset_tweaks.tweaks,
        renderer,
    );

    // low ledger lines
    draw_lines(
        origin,
        (5..to).rev(),
        tick,
        glyph_width,
        shunts,
        &offset_tweaks.tweaks,
        renderer,
    );
}

#[cfg(test)]
mod tests {
    use super::draw_ledger_lines;
    use crate::instructions::parse::get_accidentals::Accidentals;
    use crate::instructions::parse::get_shunts::Shunts;
    use crate::instructions::parse::get_tone_offset_info::ToneOffsetInfo;
    use crate::instructions::parse::get_written_durations::Notation;
    use crate::renderer::{DrawInstruction, test_helpers::make_test_renderer};
    use crate::score::components::measurements::Point;
    use rustc_hash::FxHashMap;

    fn empty_shunts() -> Shunts {
        Shunts { by_id: FxHashMap::default(), by_offset: FxHashMap::default() }
    }

    fn notation_with_info(highest: i8, lowest: i8) -> Notation {
        let mut n = Notation::new(0, 32);
        n.info = Some(ToneOffsetInfo { highest, lowest, furthest: 0 });
        n
    }

    #[test]
    fn no_ledger_lines_for_mid_staff_note() {
        let entry = notation_with_info(0, 0);
        let mut r = make_test_renderer();
        draw_ledger_lines(
            &Point { x: 0.0, y: 0.0 },
            &0,
            &entry,
            1.175,
            &empty_shunts(),
            &Accidentals::new(),
            &FxHashMap::default(),
            &mut r,
        );
        assert_eq!(r.instructions.len(), 0);
    }

    #[test]
    fn one_high_ledger_line_for_offset_minus_six() {
        // highest = -6: range (-6..-5), only -6 (even) → 1 line
        let entry = notation_with_info(-6, 0);
        let mut r = make_test_renderer();
        draw_ledger_lines(
            &Point { x: 0.0, y: 0.0 },
            &0,
            &entry,
            1.175,
            &empty_shunts(),
            &Accidentals::new(),
            &FxHashMap::default(),
            &mut r,
        );
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        assert_eq!(lines, 1);
    }

    #[test]
    fn one_low_ledger_line_for_offset_six() {
        // lowest = 6: range (5..7).rev() = 6,5; only 6 (even) → 1 line
        let entry = notation_with_info(0, 6);
        let mut r = make_test_renderer();
        draw_ledger_lines(
            &Point { x: 0.0, y: 0.0 },
            &0,
            &entry,
            1.175,
            &empty_shunts(),
            &Accidentals::new(),
            &FxHashMap::default(),
            &mut r,
        );
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        assert_eq!(lines, 1);
    }

    #[test]
    fn two_ledger_lines_for_high_and_low() {
        // highest = -6 (1 high), lowest = 6 (1 low) → 2 lines total
        let entry = notation_with_info(-6, 6);
        let mut r = make_test_renderer();
        draw_ledger_lines(
            &Point { x: 0.0, y: 0.0 },
            &0,
            &entry,
            1.175,
            &empty_shunts(),
            &Accidentals::new(),
            &FxHashMap::default(),
            &mut r,
        );
        let lines = r.instructions.iter()
            .filter(|i| matches!(i, DrawInstruction::Line { .. }))
            .count();
        assert_eq!(lines, 2);
    }
}
