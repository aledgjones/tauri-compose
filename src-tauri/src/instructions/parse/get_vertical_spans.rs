use crate::score::engraves::{BracketingApproach, Engrave};
use crate::score::instruments::Instrument;
use crate::utils::UniqueId;

#[derive(PartialEq)]
enum BracketSpan {
    None,
    Start,
    Continue,
}

pub struct VerticalSpan {
    pub start: UniqueId,
    pub stop: UniqueId,
}

pub struct VerticalSpans {
    pub brackets: Vec<VerticalSpan>,
    pub sub_brackets: Vec<VerticalSpan>,
    pub braces: Vec<VerticalSpan>,
    pub barlines: Vec<VerticalSpan>,
}

impl VerticalSpans {
    fn new() -> Self {
        Self {
            brackets: Vec::new(),
            sub_brackets: Vec::new(),
            braces: Vec::new(),
            barlines: Vec::new(),
        }
    }
}

fn get_span_type(
    instrument: &Instrument,
    previous_instrument: Option<&Instrument>,
    engrave: &Engrave,
) -> BracketSpan {
    match engrave.bracketing_approach {
        BracketingApproach::None => BracketSpan::None,
        BracketingApproach::SmallEnsemble => {
            if instrument.stave_ids.len() > 1 {
                return BracketSpan::None;
            }
            match previous_instrument {
                None => BracketSpan::Start,
                Some(instrument) => {
                    if instrument.stave_ids.len() > 1 {
                        BracketSpan::Start
                    } else {
                        BracketSpan::Continue
                    }
                }
            }
        }
        BracketingApproach::Orchestral => {
            let family = instrument.family();
            let previous_family = match previous_instrument {
                None => "",
                Some(instrument) => instrument.family(),
            };

            match family {
                "strings" | "woodwinds" | "brass" => {
                    if family == previous_family {
                        BracketSpan::Continue
                    } else {
                        BracketSpan::Start
                    }
                }
                _ => BracketSpan::None,
            }
        }
    }
}

pub fn get_vertical_spans(instruments: &Vec<&Instrument>, engrave: &Engrave) -> VerticalSpans {
    let mut output = VerticalSpans::new();

    let mut previous_instrument: Option<&Instrument> = None;

    for instrument in instruments {
        let span_type = get_span_type(instrument, previous_instrument, engrave);

        // BRACKETS
        match span_type {
            BracketSpan::Start => output.brackets.push(VerticalSpan {
                start: instrument.id,
                stop: instrument.id,
            }),
            BracketSpan::Continue => {
                let last = output.brackets.last_mut().unwrap();
                last.stop = instrument.id;
            }
            _ => {}
        };

        // SUB-BRACKETS
        if span_type == BracketSpan::Continue {
            if let Some(previous_instrument) = previous_instrument {
                if instrument.definition_id == previous_instrument.definition_id {
                    match output.sub_brackets.last_mut() {
                        None => output.sub_brackets.push(VerticalSpan {
                            start: previous_instrument.id,
                            stop: instrument.id,
                        }),
                        Some(entry) => {
                            if entry.stop == previous_instrument.id {
                                // extend the span another instrument
                                entry.stop = instrument.id;
                            } else {
                                // start a span from the previous instrument
                                output.sub_brackets.push(VerticalSpan {
                                    start: previous_instrument.id,
                                    stop: instrument.id,
                                });
                            }
                        }
                    }
                }
            };
        };

        // BRACES
        if instrument.stave_ids.len() > 1 {
            output.braces.push(VerticalSpan {
                start: *instrument.stave_ids.first().unwrap(),
                stop: *instrument.stave_ids.last().unwrap(),
            });
        }

        // BARLINES
        match span_type {
            BracketSpan::Start => output.barlines.push(VerticalSpan {
                start: instrument.id,
                stop: instrument.id,
            }),
            BracketSpan::Continue => {
                let last = output.barlines.last_mut().unwrap();
                last.stop = instrument.id;
            }
            BracketSpan::None => output.barlines.push(VerticalSpan {
                start: instrument.id,
                stop: instrument.id,
            }),
        };

        previous_instrument = Some(instrument);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::score::engraves::Engraves;
    use crate::score::instruments::{defs::InstrumentType, Instrument};
    use crate::score::layouts::LayoutType;

    fn make_instrument(id: u64, definition_id: &str, stave_ids: Vec<u64>) -> Instrument {
        Instrument {
            id,
            definition_id: definition_id.to_string(),
            instrument_type: InstrumentType::Melodic,
            long_name: String::new(),
            short_name: String::new(),
            stave_ids,
            count: None,
            assigned_plugin_id: None,
            volume: 100,
            pan: 64,
            mute: false,
            solo: false,
        }
    }

    #[test]
    fn empty_instruments_produces_empty_spans() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let vs = get_vertical_spans(&vec![], engrave);
        assert!(vs.brackets.is_empty());
        assert!(vs.barlines.is_empty());
    }

    #[test]
    fn single_instrument_produces_one_barline_span() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let inst = make_instrument(1, "woodwinds.flute", vec![10]);
        let vs = get_vertical_spans(&vec![&inst], engrave);
        // one instrument → always one barline span
        assert_eq!(vs.barlines.len(), 1);
        assert_eq!(vs.barlines[0].start, 1);
        assert_eq!(vs.barlines[0].stop, 1);
    }

    #[test]
    fn none_bracketing_approach_produces_no_brackets() {
        // Full score uses Orchestral by default; let's test no-bracket using clone
        use crate::score::components::{
            measurements::PaddingMm,
            text::{Align, Font, FontFamily, Justify},
        };
        use crate::score::engraves::{BracketStyle, BracketingApproach, Engrave};
        let none_engrave = Engrave {
            space: 1.5,
            page_padding: PaddingMm::new(30.0, 20.0, 30.0, 20.0),
            frame_padding: PaddingMm::new(20.0, 0.0, 20.0, 0.0),
            instrument_spacing: 8.0,
            stave_spacing: 6.0,
            system_start_padding: 0.75,
            instrument_name_font: Font {
                size: 1.75,
                family: FontFamily::LibreBaskerville,
                justify: Justify::End,
                align: Align::Middle,
            },
            instrument_name_spacing: 2.0,
            systemic_barline_single_instrument_system: false,
            bracketing_approach: BracketingApproach::None,
            bracket_style: BracketStyle::None,
            bracket_single_staves: false,
            base_note_space: 4.0,
            minimum_note_space: 2.0,
            minimum_tie_space: 2.0,
            note_space_ratio: 0.5,
            maximum_beam_slant: 0.5,
        };
        let inst1 = make_instrument(1, "woodwinds.flute", vec![10]);
        let inst2 = make_instrument(2, "woodwinds.oboe", vec![11]);
        let vs = get_vertical_spans(&vec![&inst1, &inst2], &none_engrave);
        assert!(vs.brackets.is_empty());
    }

    #[test]
    fn orchestral_same_family_merges_bracket() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        // Full score uses Orchestral bracketing
        let inst1 = make_instrument(1, "woodwinds.flute", vec![10]);
        let inst2 = make_instrument(2, "woodwinds.oboe", vec![11]);
        let vs = get_vertical_spans(&vec![&inst1, &inst2], engrave);
        // Both woodwinds → one bracket spanning both
        assert_eq!(vs.brackets.len(), 1);
        assert_eq!(vs.brackets[0].start, 1);
        assert_eq!(vs.brackets[0].stop, 2);
    }

    #[test]
    fn orchestral_different_families_get_separate_brackets() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let inst1 = make_instrument(1, "woodwinds.flute", vec![10]);
        let inst2 = make_instrument(2, "brass.horn.f", vec![11]);
        let vs = get_vertical_spans(&vec![&inst1, &inst2], engrave);
        // Different families → two separate bracket spans
        assert_eq!(vs.brackets.len(), 2);
    }

    #[test]
    fn instrument_with_multiple_staves_gets_brace() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let inst = make_instrument(1, "keyboard.piano", vec![10, 11]);
        let vs = get_vertical_spans(&vec![&inst], engrave);
        assert_eq!(vs.braces.len(), 1);
        assert_eq!(vs.braces[0].start, 10);
        assert_eq!(vs.braces[0].stop, 11);
    }

    #[test]
    fn instrument_with_single_stave_gets_no_brace() {
        let engraves = Engraves::new();
        let engrave = &engraves.by_layout_type[&LayoutType::FullScore];
        let inst = make_instrument(1, "woodwinds.flute", vec![10]);
        let vs = get_vertical_spans(&vec![&inst], engrave);
        assert!(vs.braces.is_empty());
    }
}
