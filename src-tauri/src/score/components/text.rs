use crate::renderer::converter::Space;
use crate::renderer::text_measurer::TextMeasurer;
use crate::score::components::measurements::BoundingBox;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

const INLINE_SYMBOL_SCALE: Space = 1.4;
const INLINE_SYMBOL_Y_OFFSET: Space = -0.3;
const INLINE_SYMBOL_LEFT_GAP: Space = -0.2;
const INLINE_SYMBOL_WIDTH_SCALE: Space = 0.5;

#[derive(Copy, Clone, PartialEq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Justify {
    Start,
    Middle,
    End,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Align {
    Top,
    Middle,
    Bottom,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum FontFamily {
    Bravura,
    LibreBaskerville,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Font {
    pub size: Space,
    pub family: FontFamily,
    pub justify: Justify,
    pub align: Align,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpan {
    pub text: String,
    pub font: Font,
    pub x_offset: Space,
    pub y_offset: Space,
    pub width_scale: Space,
}

fn glyph_for_placeholder(token: &str) -> Option<&'static str> {
    match token {
        "${copy}" => Some("\u{00A9}"),
        "${sharp}" => Some("\u{E262}"),
        "${natural}" => Some("\u{E261}"),
        "${flat}" => Some("\u{E260}"),
        "${double-sharp}" => Some("\u{E263}"),
        "${double-flat}" => Some("\u{E264}"),
        _ => None,
    }
}

fn symbol_font(base_font: &Font) -> Font {
    Font {
        size: base_font.size * INLINE_SYMBOL_SCALE,
        family: FontFamily::Bravura,
        justify: base_font.justify,
        align: Align::Middle,
    }
}

fn push_text_span(spans: &mut Vec<TextSpan>, text: &str, font: Font) {
    push_positioned_text_span(spans, text, font, 0.0, 0.0, 1.0);
}

fn push_positioned_text_span(
    spans: &mut Vec<TextSpan>,
    text: &str,
    font: Font,
    x_offset: Space,
    y_offset: Space,
    width_scale: Space,
) {
    if text.is_empty() {
        return;
    }

    if let Some(last) = spans.last_mut() {
        if last.font == font
            && last.x_offset == x_offset
            && last.y_offset == y_offset
            && last.width_scale == width_scale
        {
            last.text.push_str(text);
            return;
        }
    }

    spans.push(TextSpan {
        text: String::from(text),
        font,
        x_offset,
        y_offset,
        width_scale,
    });
}

pub fn text_span_width(span: &TextSpan, text_measurer: &TextMeasurer<'_>) -> Space {
    text_measurer.measure(&span.text, &span.font).width * span.width_scale
}

pub fn tokenize_text_with_symbols(text: &str, font: &Font) -> Vec<TextSpan> {
    let mut spans = Vec::new();
    let mut index = 0;

    while let Some(start_offset) = text[index..].find("${") {
        let start = index + start_offset;

        push_text_span(&mut spans, &text[index..start], font.clone());

        let Some(end_offset) = text[start..].find('}') else {
            push_text_span(&mut spans, &text[start..], font.clone());
            return spans;
        };

        let end = start + end_offset + 1;
        let token = &text[start..end];

        if let Some(glyph) = glyph_for_placeholder(token) {
            push_positioned_text_span(
                &mut spans,
                glyph,
                symbol_font(font),
                INLINE_SYMBOL_LEFT_GAP,
                INLINE_SYMBOL_Y_OFFSET,
                INLINE_SYMBOL_WIDTH_SCALE,
            );
        } else {
            push_text_span(&mut spans, token, font.clone());
        }

        index = end;
    }

    push_text_span(&mut spans, &text[index..], font.clone());

    spans
}

pub fn measure_tokenized_text(
    text: &str,
    font: &Font,
    text_measurer: &TextMeasurer<'_>,
) -> BoundingBox {
    let mut width: Space = 0.0;
    let mut height: Space = 0.0;

    for span in tokenize_text_with_symbols(text, font) {
        let bounds = text_measurer.measure(&span.text, &span.font);
        width += span.x_offset + text_span_width(&span, text_measurer);
        height = height.max(bounds.height);
    }

    BoundingBox {
        width,
        height,
        right_padding: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::test_helpers::make_test_renderer;

    fn make_font() -> Font {
        Font {
            size: 1.75,
            family: FontFamily::LibreBaskerville,
            justify: Justify::End,
            align: Align::Middle,
        }
    }

    #[test]
    fn tokenize_text_with_symbols_replaces_known_placeholder_with_bravura_span() {
        let spans = tokenize_text_with_symbols("Trumpet in B${flat}", &make_font());

        assert_eq!(spans.len(), 2);
        assert_eq!(spans[0].text, "Trumpet in B");
        assert_eq!(spans[0].font.family, FontFamily::LibreBaskerville);
        assert_eq!(spans[0].x_offset, 0.0);
        assert_eq!(spans[0].y_offset, 0.0);
        assert_eq!(spans[0].width_scale, 1.0);
        assert_eq!(spans[1].text, "\u{E260}");
        assert_eq!(spans[1].font.family, FontFamily::Bravura);
        assert!(spans[1].font.size > spans[0].font.size);
        assert_eq!(spans[1].x_offset, INLINE_SYMBOL_LEFT_GAP);
        assert_eq!(spans[1].y_offset, INLINE_SYMBOL_Y_OFFSET);
        assert_eq!(spans[1].width_scale, INLINE_SYMBOL_WIDTH_SCALE);
    }

    #[test]
    fn tokenize_text_with_symbols_leaves_unknown_placeholder_as_text() {
        let spans = tokenize_text_with_symbols("Horn in ${unknown}", &make_font());

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].text, "Horn in ${unknown}");
        assert_eq!(spans[0].font.family, FontFamily::LibreBaskerville);
    }

    #[test]
    fn measure_tokenized_text_sums_segment_widths_and_max_height() {
        let renderer = make_test_renderer();
        let text_measurer = TextMeasurer::new(&renderer.converter);
        let font = make_font();

        let bounds = measure_tokenized_text("B${flat}", &font, &text_measurer);
        let expected_text = text_measurer.measure("B", &font);
        let expected_symbol = text_measurer.measure("\u{E260}", &symbol_font(&font));

        assert!(
            (bounds.width
                - (expected_text.width
                    + INLINE_SYMBOL_LEFT_GAP
                    + (expected_symbol.width * INLINE_SYMBOL_WIDTH_SCALE)))
                .abs()
                < 1e-6
        );
        assert!((bounds.height - expected_text.height.max(expected_symbol.height)).abs() < 1e-6);
    }
}
