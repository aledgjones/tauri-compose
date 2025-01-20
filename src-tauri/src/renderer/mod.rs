mod parse;

use crate::score::components::measurements::PaddingSpaces;
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::score::components::units::Converter;
use crate::score::layouts::LayoutType;
use crate::utils::UniqueId;
use crate::AppState;
use skia_safe::svg::canvas::Canvas;
use skia_safe::{Color, Font as SkiaFont, FontMgr, Paint, Path, Point, Rect, TextBlob, Typeface};
use tauri::{ipc::Response, State};

pub struct Fonts {
    pub bravura: Typeface,
    pub libre_baskerville: Typeface,
}

impl Fonts {
    pub fn new() -> Self {
        let font_mgr = FontMgr::new();

        // Load the fixed set of fonts
        let bravura_font_data = include_bytes!("../../../src/fonts/bravura.woff2");
        let libre_baskerville_font_data =
            include_bytes!("../../../src/fonts/libre-baskerville.woff2");

        // Load the typefaces
        let bravura = font_mgr
            .new_from_data(bravura_font_data, None)
            .expect("Failed to load Bravura font");

        let libre_baskerville = font_mgr
            .new_from_data(libre_baskerville_font_data, None)
            .expect("Failed to load Libre Baskerville font");

        Fonts {
            bravura,
            libre_baskerville,
        }
    }
}

pub struct Renderer {
    canvas: Option<Canvas>,
    fonts: Fonts,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Renderer {
        let canvas = Canvas::new(Rect::from_wh(width as f32, height as f32), None);

        canvas.clear(Color::WHITE);

        let fonts = Fonts::new();

        Renderer {
            canvas: Some(canvas),
            fonts,
        }
    }

    pub fn line(&mut self, points: Vec<Point>) {
        if let Some(canvas) = &mut self.canvas {
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_stroke(true);

            let mut path = Path::new();

            path.move_to(points[0]);
            for point in &points[1..] {
                path.line_to(*point);
            }
            canvas.draw_path(&path, &paint);
        }
    }

    pub fn text(&mut self, text: &str, font: &Font, point: Point) {
        if let Some(canvas) = &mut self.canvas {
            let typeface = match font.family {
                FontFamily::Bravura => &self.fonts.bravura,
                FontFamily::LibreBaskerville => &self.fonts.libre_baskerville,
            };

            let skia_font = SkiaFont::new(typeface.clone(), font.size);

            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            paint.set_color(Color::BLACK);

            let (_, metrics) = skia_font.metrics();
            let (_, bounds) = skia_font.measure_str(text, Some(&paint));

            let x = match font.justify {
                Justify::Start => point.x + font.padding.left,
                Justify::Middle => point.x + font.padding.left - bounds.width() / 2.0,
                Justify::End => point.x - bounds.width() - font.padding.right,
            };

            let y = match font.align {
                Align::Top => point.y + font.padding.top + metrics.x_height,
                Align::Middle => point.y + font.padding.top + (metrics.x_height) / 2.0,
                Align::Bottom => point.y - font.padding.bottom,
            };

            let blob = TextBlob::new(text, &skia_font).unwrap();

            canvas.draw_text_blob(&blob, Point { x, y }, &paint);
        }
    }

    pub fn data(&mut self) -> Vec<u8> {
        if let Some(canvas) = self.canvas.take() {
            let data = canvas.end();
            data.as_bytes().to_vec()
        } else {
            vec![]
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn render(state: State<'_, AppState>, flow_id: UniqueId) -> Result<Response, ()> {
    // ** TODO: pass these into the function
    let mm: f32 = 3.77;
    let layout_type = LayoutType::FullScore;
    // **

    let state = state.lock().unwrap();

    let engrave = &state.score.engraves.by_layout_type[&layout_type];
    let converter = Converter::new(mm, engrave.space);

    let vertical_spacing = state.measure_vertical_spacing(flow_id, engrave);

    let frame_padding = engrave.frame_padding.to_spaces(&converter);

    let height =
        converter.spaces_to_px(frame_padding.top + vertical_spacing.height + frame_padding.bottom);
    let width = converter.spaces_to_px(frame_padding.left + 50.0 + frame_padding.right);

    let mut renderer = Renderer::new(width as i32, height as i32);

    renderer.text(
        "Texty",
        &Font {
            size: 12.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::Start,
            align: Align::Top,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 120.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 110.0 },
        Point { x: 100.0, y: 130.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 120.0 },
        Point { x: 110.0, y: 120.0 },
    ]);

    renderer.text(
        "Texty",
        &Font {
            size: 12.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::Start,
            align: Align::Middle,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 180.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 170.0 },
        Point { x: 100.0, y: 190.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 180.0 },
        Point { x: 110.0, y: 180.0 },
    ]);

    renderer.text(
        "Texty",
        &Font {
            size: 12.0,
            family: FontFamily::LibreBaskerville,
            justify: Justify::Start,
            align: Align::Bottom,
            padding: PaddingSpaces {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        },
        Point { x: 100.0, y: 240.0 },
    );

    renderer.line(vec![
        Point { x: 100.0, y: 230.0 },
        Point { x: 100.0, y: 250.0 },
    ]);

    renderer.line(vec![
        Point { x: 90.0, y: 240.0 },
        Point { x: 110.0, y: 240.0 },
    ]);

    Ok(Response::new(renderer.data()))
}
