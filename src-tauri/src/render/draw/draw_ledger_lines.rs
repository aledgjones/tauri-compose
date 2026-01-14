use crate::render::draw::draw_staves::STAVE_LINE_WIDTH;
use crate::render::parse::get_accidentals::Accidentals;
use crate::render::parse::get_shunts::{Shunt, Shunts};
use crate::render::parse::get_tone_offsets::ToneVerticalOffsets;
use crate::render::parse::get_written_durations::Notation;
use crate::renderer::converter::Space;
use crate::renderer::Renderer;
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
    renderer: &mut Renderer,
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
    renderer: &mut Renderer,
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
