use crate::score::components::text::{Align, Font, FontFamily, Justify};
use skia_safe::utils::text_utils::Align as SkiaAlign;
use skia_safe::{
    surfaces, Color, EncodedImageFormat, Font as SkiaFont, FontMgr, Paint, Path, Point, Surface,
    Typeface,
};

pub struct Fonts {
    pub bravura: Typeface,
    pub libre_baskerville: Typeface,
}

impl Fonts {
    pub fn new() -> Self {
        let font_mgr = FontMgr::new();

        // Load the fixed set of fonts
        let bravura_font_data = include_bytes!("../../../src/assets/fonts/bravura.woff2");
        let libre_baskerville_font_data =
            include_bytes!("../../../src/assets/fonts/libre-baskerville.woff2");

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
    surface: Surface,
    fonts: Fonts,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Renderer {
        let mut surface = surfaces::raster_n32_premul((width, height)).expect("surface");
        surface.canvas().clear(Color::WHITE);

        let fonts = Fonts::new();

        Renderer { surface, fonts }
    }

    pub fn line(&mut self, points: Vec<Point>) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke(true);

        let mut path = Path::new();

        path.move_to(points[0]);
        for point in &points[1..] {
            path.line_to(*point);
        }
        self.surface.canvas().draw_path(&path, &paint);
    }

    pub fn text(&mut self, text: &str, font: &Font, point: Point) {
        let typeface = match font.family {
            FontFamily::Bravura => &self.fonts.bravura,
            FontFamily::LibreBaskerville => &self.fonts.libre_baskerville,
        };

        let skia_font = SkiaFont::new(typeface.clone(), font.size);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);

        let (_, metrics) = skia_font.metrics();

        let justify = match font.justify {
            Justify::Start => SkiaAlign::Left,
            Justify::Middle => SkiaAlign::Center,
            Justify::End => SkiaAlign::Right,
        };

        let y = match font.align {
            Align::Top => point.y + font.padding.top + metrics.x_height,
            Align::Middle => point.y + font.padding.top + (metrics.x_height) / 2.0,
            Align::Bottom => point.y - font.padding.bottom,
        };

        self.surface.canvas().draw_text_align(
            text,
            Point { x: point.x, y },
            &skia_font,
            &paint,
            justify,
        );
    }

    pub fn data(&mut self) -> Vec<u8> {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        match image.encode(context.as_mut(), EncodedImageFormat::WEBP, None) {
            Some(data) => data.as_bytes().to_vec(),
            None => {
                println!("None");
                vec![]
            }
        }
    }
}
