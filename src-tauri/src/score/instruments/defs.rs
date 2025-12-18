use crate::score::{
    components::pitch::{Accidental, Pitch},
    entries::clef::{Clef, ClefDrawType},
    players::PlayerType,
};
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde::Serialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Copy, Clone, Hash, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Expression {
    Natural = 0,
    Pizzicato,
    Spiccato,
    Staccato,
    Tremolo,
    Mute, // may have to convert to MuteStaccato etc. later for mixed expressions
}

#[derive(Copy, Clone, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum InstrumentType {
    Melodic = 0,
    Percussive,
}

pub struct StaveDefinition {
    pub lines: Vec<u8>,
    pub clef: Clef,
}

pub struct InstrumentDefinition {
    pub id: &'static str,
    pub instrument_type: InstrumentType,
    pub path: Vec<&'static str>,
    pub long_name: &'static str,
    pub short_name: &'static str,
    pub staves: Vec<StaveDefinition>,
    pub solo_patches: FxHashMap<Expression, &'static str>,
    pub section_patches: FxHashMap<Expression, &'static str>,
}

lazy_static! {
    pub static ref INSTRUMENT_DEFINITIONS: Vec<InstrumentDefinition> = {
        vec![
            InstrumentDefinition {
                id: "brass.bass-trombone",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Bass Trombone"],
                long_name: "Bass Trombone",
                short_name: "B. Tbn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "bass-trombone/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "bass-trombone/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "brass.horn.f",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Horn", "F"],
                long_name: "Horn in F",
                short_name: "F Hn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "horn/natural.json"),
                    (Expression::Staccato, "horn/staccato.json"),
                    (Expression::Mute, "horn/mute.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "horn/natural.json"),
                    (Expression::Staccato, "horn/staccato.json"),
                    (Expression::Mute, "horn/mute.json"),
                ]),
            },
            InstrumentDefinition {
                id: "brass.trombone",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Trombone"],
                long_name: "Trombone",
                short_name: "Tbn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trombone/natural.json"),
                    (Expression::Staccato, "trombone/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trombone/natural.json"),
                    (Expression::Staccato, "trombone/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "brass.trumpet.b-flat",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Trumpet", "B Flat"],
                long_name: "Trumpet in B${flat}",
                short_name: "B${flat} Tpt.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trumpet/natural.json"),
                    (Expression::Staccato, "trumpet/staccato.json"),
                    (Expression::Mute, "trumpet/mute.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trumpet/natural.json"),
                    (Expression::Staccato, "trumpet/staccato.json"),
                    (Expression::Mute, "trumpet/mute.json"),
                ]),
            },
            InstrumentDefinition {
                id: "brass.trumpet.c",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Trumpet", "C"],
                long_name: "Trumpet in C",
                short_name: "C Tpt.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trumpet/natural.json"),
                    (Expression::Staccato, "trumpet/staccato.json"),
                    (Expression::Mute, "trumpet/mute.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "trumpet/natural.json"),
                    (Expression::Staccato, "trumpet/staccato.json"),
                    (Expression::Mute, "trumpet/mute.json"),
                ]),
            },
            InstrumentDefinition {
                id: "brass.tuba",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Brass", "Tuba"],
                long_name: "Tuba",
                short_name: "Tba.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "tuba/natural.json"),
                    (Expression::Staccato, "tuba/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "tuba/natural.json"),
                    (Expression::Staccato, "tuba/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "guitar.acoustic",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Guitar", "Acoustic Guitar"],
                long_name: "Acoustic Guitar",
                short_name: "A. Gtr.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "acoustic-guitar/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "acoustic-guitar/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "guitar.bass",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Guitar", "Bass Guitar"],
                long_name: "Bass Guitar",
                short_name: "B. Gtr.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "bass-guitar/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "bass-guitar/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "guitar.distortion",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Guitar", "Distortion Guitar"],
                long_name: "Distortion Guitar",
                short_name: "Gtr.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "distortion-guitar/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "distortion-guitar/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "unpitched-percussion.crash-cymbal",
                instrument_type: InstrumentType::Percussive,
                path: vec!["Unpitched Percussion", "Crash Cymbal"],
                long_name: "Crash Cymbal",
                short_name: "Cym.",
                staves: vec![StaveDefinition {
                    lines: vec![1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(60, Accidental::Natural),
                        0,
                        ClefDrawType::Percussion,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-crash/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-crash/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "unpitched-percussion.hi-hat",
                instrument_type: InstrumentType::Percussive,
                path: vec!["Unpitched Percussion", "Hi-Hat"],
                long_name: "Hi-Hat",
                short_name: "HH.",
                staves: vec![StaveDefinition {
                    lines: vec![1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(60, Accidental::Natural),
                        0,
                        ClefDrawType::Percussion,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-hihat/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-hihat/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "unpitched-percussion.kick",
                instrument_type: InstrumentType::Percussive,
                path: vec!["Unpitched Percussion", "Kick Drum"],
                long_name: "Kick Drum",
                short_name: "K Drm.",
                staves: vec![StaveDefinition {
                    lines: vec![1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(60, Accidental::Natural),
                        0,
                        ClefDrawType::Percussion,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-kicks/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "kit-kicks/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "unpitched-percussion.snare",
                instrument_type: InstrumentType::Percussive,
                path: vec!["Unpitched Percussion", "Snare"],
                long_name: "Snare",
                short_name: "Sn.",
                staves: vec![StaveDefinition {
                    lines: vec![1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(60, Accidental::Natural),
                        0,
                        ClefDrawType::Percussion,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(Expression::Natural, "snare/natural.json")]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "snare/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.glockenspiel",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Glockenspiel"],
                long_name: "Glokenspiel",
                short_name: "Glock.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "glockenspiel/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "glockenspiel/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.harp",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Harp"],
                long_name: "Harp",
                short_name: "Hrp.",
                staves: vec![
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(67, Accidental::Natural),
                            2,
                            ClefDrawType::G,
                        ),
                    },
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(53, Accidental::Natural),
                            -2,
                            ClefDrawType::F,
                        ),
                    },
                ],
                solo_patches: FxHashMap::from_iter([(Expression::Natural, "harp/natural.json")]),
                section_patches: FxHashMap::from_iter([(Expression::Natural, "harp/natural.json")]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.marimba",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Marimba"],
                long_name: "Marimba",
                short_name: "Mrm.",
                staves: vec![
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(67, Accidental::Natural),
                            2,
                            ClefDrawType::G,
                        ),
                    },
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(53, Accidental::Natural),
                            -2,
                            ClefDrawType::F,
                        ),
                    },
                ],
                solo_patches: FxHashMap::from_iter([(Expression::Natural, "marimba/natural.json")]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "marimba/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.timpani",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Timpani"],
                long_name: "Timpani",
                short_name: "Timp.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "timpani/natural.json"),
                    (Expression::Tremolo, "timpani/roll.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "timpani/natural.json"),
                    (Expression::Tremolo, "timpani/roll.json"),
                ]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.vibraphone",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Vibraphone"],
                long_name: "Vibraphone",
                short_name: "Vib.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "vibraphone/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "vibraphone/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "pitched-percussion.xylophone",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Pitched Percussion", "Xylophone"],
                long_name: "Xylophone",
                short_name: "Xyl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(79, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "xylophone/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "xylophone/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "keyboard.celesta",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Keyboards", "Celesta"],
                long_name: "Celesta",
                short_name: "Cel.",
                staves: vec![
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(79, Accidental::Natural),
                            2,
                            ClefDrawType::G,
                        ),
                    },
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(65, Accidental::Natural),
                            -2,
                            ClefDrawType::F,
                        ),
                    },
                ],
                solo_patches: FxHashMap::from_iter([(Expression::Natural, "celesta/natural.json")]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "celesta/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "keyboard.piano",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Keyboards", "Piano"],
                long_name: "Piano",
                short_name: "Pno.",
                staves: vec![
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(67, Accidental::Natural),
                            2,
                            ClefDrawType::G,
                        ),
                    },
                    StaveDefinition {
                        lines: vec![1, 1, 1, 1, 1],
                        clef: Clef::new(
                            0,
                            0,
                            Pitch::new(53, Accidental::Natural),
                            -2,
                            ClefDrawType::F,
                        ),
                    },
                ],
                solo_patches: FxHashMap::from_iter([(Expression::Natural, "piano/natural.json")]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "piano/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "strings.contrabass",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Strings", "Contrabass"],
                long_name: "Contrabass",
                short_name: "Cb.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(41, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "contrabass/natural.json"),
                    (Expression::Pizzicato, "contrabass/pizzicato.json"),
                    (Expression::Staccato, "contrabass/spiccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "contrabass-section/natural.json"),
                    (Expression::Pizzicato, "contrabass-section/pizzicato.json"),
                    (Expression::Staccato, "contrabass-section/spiccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "strings.viola",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Strings", "Viola"],
                long_name: "Viola",
                short_name: "Vla.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(60, Accidental::Natural),
                        0,
                        ClefDrawType::C,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "viola/natural.json"),
                    (Expression::Pizzicato, "viola/pizzicato.json"),
                    (Expression::Staccato, "viola/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "viola-section/natural.json"),
                    (Expression::Pizzicato, "viola-section/pizzicato.json"),
                    (Expression::Staccato, "viola-section/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "strings.violin",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Strings", "Violin"],
                long_name: "Violin",
                short_name: "Vln.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "violin/natural.json"),
                    (Expression::Pizzicato, "violin/pizzicato.json"),
                    (Expression::Staccato, "violin/spiccato.json"),
                    (Expression::Tremolo, "violin/tremolo.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "violin-section/natural.json"),
                    (Expression::Pizzicato, "violin-section/pizzicato.json"),
                    (Expression::Staccato, "violin-section/spiccato.json"),
                    (Expression::Tremolo, "violin-section/tremolo.json"),
                ]),
            },
            InstrumentDefinition {
                id: "strings.violoncello",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Strings", "Violoncello"],
                long_name: "Violoncello",
                short_name: "Vc.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "violoncello/natural.json"),
                    (Expression::Pizzicato, "violoncello/pizzicato.json"),
                    (Expression::Staccato, "violoncello/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "violoncello-section/natural.json"),
                    (Expression::Pizzicato, "violoncello-section/pizzicato.json"),
                    (Expression::Staccato, "violoncello-section/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.alto-flute",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Alto Flute"],
                long_name: "Alto Flute",
                short_name: "A. Fl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "alto-flute/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "alto-flute/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "woodwinds.alto-sxophone",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Alto Saxophone"],
                long_name: "Alto Saxophone",
                short_name: "A. Sax.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "alto-saxophone/natural.json"),
                    (Expression::Staccato, "alto-saxophone/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "alto-saxophone/natural.json"),
                    (Expression::Staccato, "alto-saxophone/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.bassoon",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Bassoon"],
                long_name: "Bassoon",
                short_name: "Bsn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "bassoon/natural.json"),
                    (Expression::Staccato, "bassoon/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "bassoon/natural.json"),
                    (Expression::Staccato, "bassoon/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.bass-clarinet",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Bass Clarinet"],
                long_name: "Bass Clarinet",
                short_name: "B. Cl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "bass-clarinet/natural.json"),
                    (Expression::Staccato, "bass-clarinet/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "bass-clarinet/natural.json"),
                    (Expression::Staccato, "bass-clarinet/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.clarinet.a",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Clarinet", "A"],
                long_name: "Clarinet in A",
                short_name: "A Cl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "clarinet/natural.json"),
                    (Expression::Staccato, "clarinet/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "clarinet/natural.json"),
                    (Expression::Staccato, "clarinet/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.clarinet.b-flat",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Clarinet", "B Flat"],
                long_name: "Clarinet in B${flat}",
                short_name: "B${flat} Cl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "clarinet/natural.json"),
                    (Expression::Staccato, "clarinet/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "clarinet/natural.json"),
                    (Expression::Staccato, "clarinet/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.contrabassoon",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Contrabassoon"],
                long_name: "Contrabasson",
                short_name: "Cbsn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(53, Accidental::Natural),
                        -2,
                        ClefDrawType::F,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "contrabassoon/natural.json",
                )]),
                section_patches: FxHashMap::from_iter([(
                    Expression::Natural,
                    "contrabassoon/natural.json",
                )]),
            },
            InstrumentDefinition {
                id: "woodwinds.english-horn",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "English Horn"],
                long_name: "English Horn",
                short_name: "E Hn.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "cor-anglais/natural.json"),
                    (Expression::Staccato, "cor-anglais/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "cor-anglais/natural.json"),
                    (Expression::Staccato, "cor-anglais/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.flute",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Flute"],
                long_name: "Flute",
                short_name: "Fl.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "flute/natural.json"),
                    (Expression::Staccato, "flute/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "flute/natural.json"),
                    (Expression::Staccato, "flute/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.oboe",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Oboe"],
                long_name: "Oboe",
                short_name: "Ob.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(67, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "oboe/natural.json"),
                    (Expression::Staccato, "oboe/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "oboe/natural.json"),
                    (Expression::Staccato, "oboe/staccato.json"),
                ]),
            },
            InstrumentDefinition {
                id: "woodwinds.piccolo",
                instrument_type: InstrumentType::Melodic,
                path: vec!["Woodwinds", "Piccolo"],
                long_name: "Piccolo",
                short_name: "Pc.",
                staves: vec![StaveDefinition {
                    lines: vec![1, 1, 1, 1, 1],
                    clef: Clef::new(
                        0,
                        0,
                        Pitch::new(79, Accidental::Natural),
                        2,
                        ClefDrawType::G,
                    ),
                }],
                solo_patches: FxHashMap::from_iter([
                    (Expression::Natural, "piccolo/natural.json"),
                    (Expression::Staccato, "piccolo/staccato.json"),
                ]),
                section_patches: FxHashMap::from_iter([
                    (Expression::Natural, "piccolo/natural.json"),
                    (Expression::Staccato, "piccolo/staccato.json"),
                ]),
            },
        ]
    };
}

#[derive(Serialize)]
pub struct InstrumentIdPath {
    pub id: &'static str,
    pub path: Vec<&'static str>,
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_get_all_definitions() -> Vec<InstrumentIdPath> {
    INSTRUMENT_DEFINITIONS
        .iter()
        .map(|instrument| InstrumentIdPath {
            id: instrument.id,
            path: instrument.path.clone(),
        })
        .collect()
}

#[tauri::command(rename_all = "snake_case")]
pub fn score_instruments_get_patches(id: String, player_type: PlayerType) -> FxHashMap<u8, String> {
    if let Some(def) = get_instrument_definition(&id) {
        let patches = match player_type {
            PlayerType::Solo => &def.solo_patches,
            PlayerType::Section => &def.section_patches,
        };
        patches
            .iter()
            .map(|(expr, path)| (*expr as u8, (*path).to_string()))
            .collect()
    } else {
        FxHashMap::default()
    }
}

pub fn get_instrument_definition(id: &String) -> Option<&InstrumentDefinition> {
    INSTRUMENT_DEFINITIONS.iter().find(|&def| def.id == id)
}
