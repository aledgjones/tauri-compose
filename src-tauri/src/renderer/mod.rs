pub mod converter;
pub mod fonts;
pub mod text_measurer;

use crate::score::components::measurements::{CurvePoint, Point};
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use converter::{Converter, Space};
use fonts::FONTS;
use skia_safe::utils::text_utils::Align as SkiaAlign;
use skia_safe::{
    surfaces, Canvas, Color, EncodedImageFormat, Font as SkiaFont, Paint, PaintStyle, PathBuilder,
    PathEffect, Point as SkiaPoint, Surface,
};

#[derive(Clone)]
pub enum DrawInstruction {
    Line {
        width: f32,
        color: Color,
        points: Vec<(f32, f32)>,
    },
    DashedLine {
        width: f32,
        color: Color,
        points: Vec<(f32, f32)>,
        interval: f32,
    },
    Text {
        text: String,
        font_size: f32,
        font_family: FontFamily,
        justify: Justify,
        align: Align,
        point: (f32, f32),
    },
    Circle {
        radius: f32,
        color: Color,
        origin: (f32, f32),
    },
    Curve {
        color: Color,
        points: [(f32, f32, f32); 3], // (x, y, thickness)
    },
    Shape {
        color: Color,
        points: Vec<(f32, f32)>,
    },
}

impl DrawInstruction {
    fn render(&self, renderer: &mut CanvasRenderer) {
        match self {
            DrawInstruction::Line {
                width,
                color,
                points,
            } => {
                renderer.render_line(*width, *color, points);
            }
            DrawInstruction::DashedLine {
                width,
                color,
                points,
                interval,
            } => {
                renderer.render_dashed_line(*interval, *width, *color, points);
            }
            DrawInstruction::Text {
                text,
                font_size,
                font_family,
                justify,
                align,
                point,
            } => {
                renderer.render_text(text, *font_size, *font_family, *justify, *align, *point);
            }
            DrawInstruction::Circle {
                radius,
                color,
                origin,
            } => {
                renderer.render_circle(*radius, *color, *origin);
            }
            DrawInstruction::Curve { color, points } => {
                renderer.render_curve(*color, *points);
            }
            DrawInstruction::Shape { color, points } => {
                renderer.render_shape(*color, points);
            }
        }
    }
}

pub struct CanvasRenderer {
    surface: Surface,
}

impl CanvasRenderer {
    pub fn new(width: i32, height: i32) -> Self {
        let surface = surfaces::raster_n32_premul((width, height)).expect("surface");
        Self { surface }
    }

    pub fn canvas(&mut self) -> &Canvas {
        self.surface.canvas()
    }

    pub fn clear(&mut self) {
        self.canvas().clear(Color::WHITE);
    }

    pub fn encode(&mut self) -> Vec<u8> {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        match image.encode(context.as_mut(), EncodedImageFormat::WEBP, None) {
            Some(data) => data.as_bytes().to_vec(),
            None => vec![],
        }
    }

    pub fn render_line(&mut self, width: f32, color: Color, points: &[(f32, f32)]) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke(true);
        paint.set_stroke_width(width);
        paint.set_color(color);

        let mut path_builder = PathBuilder::new();
        let first = points[0];
        path_builder.move_to((first.0, first.1));

        for point in &points[1..] {
            path_builder.line_to((point.0, point.1));
        }
        let path = path_builder.detach();

        self.canvas().draw_path(&path, &paint);
    }

    pub fn render_dashed_line(
        &mut self,
        interval: f32,
        width: f32,
        color: Color,
        points: &[(f32, f32)],
    ) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_stroke(true);
        paint.set_stroke_width(width);
        paint.set_color(color);

        paint.set_path_effect(PathEffect::dash(&[interval, interval], 0.0));

        let mut path_builder = PathBuilder::new();
        let first = points[0];
        path_builder.move_to((first.0, first.1));

        for point in &points[1..] {
            path_builder.line_to((point.0, point.1));
        }
        let path = path_builder.detach();

        self.canvas().draw_path(&path, &paint);
    }

    pub fn render_text(
        &mut self,
        text: &str,
        font_size: f32,
        font_family: FontFamily,
        justify: Justify,
        align: Align,
        point: (f32, f32),
    ) {
        let typeface = match font_family {
            FontFamily::Bravura => &FONTS.bravura,
            FontFamily::LibreBaskerville => &FONTS.libre_baskerville,
        };

        let skia_font = SkiaFont::new(typeface.clone(), font_size);

        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(Color::BLACK);

        let (_, metrics) = skia_font.metrics();

        let justify_skia = match justify {
            Justify::Start => SkiaAlign::Left,
            Justify::Middle => SkiaAlign::Center,
            Justify::End => SkiaAlign::Right,
        };

        let offset_y = match align {
            Align::Top => metrics.x_height,
            Align::Middle => (metrics.x_height) / 2.0,
            Align::Bottom => 0.0,
        };

        self.canvas().draw_text_align(
            text,
            SkiaPoint {
                x: point.0,
                y: point.1 + offset_y,
            },
            &skia_font,
            &paint,
            justify_skia,
        );
    }

    pub fn render_circle(&mut self, radius: f32, color: Color, origin: (f32, f32)) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color);

        self.canvas().draw_circle(
            SkiaPoint {
                x: origin.0,
                y: origin.1,
            },
            radius,
            &paint,
        );
    }

    pub fn render_curve(&mut self, color: Color, points: [(f32, f32, f32); 3]) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color);
        paint.set_style(PaintStyle::Fill);

        // Create control points for the thick curve outline
        let control_points = [
            // Top curve
            (points[0].0, points[0].1 + points[0].2 / 2.0),
            (
                points[1].0,
                2.0 * (points[1].1 + points[1].2 / 2.0)
                    - 0.5 * (points[0].1 + points[0].2 / 2.0)
                    - 0.5 * (points[2].1 + points[2].2 / 2.0),
            ),
            (points[2].0, points[2].1 + points[2].2 / 2.0),
            // Bottom curve
            (points[2].0, points[2].1 - points[2].2 / 2.0),
            (
                points[1].0,
                2.0 * (points[1].1 - points[1].2 / 2.0)
                    - 0.5 * (points[0].1 - points[0].2 / 2.0)
                    - 0.5 * (points[2].1 - points[2].2 / 2.0),
            ),
            (points[0].0, points[0].1 - points[0].2 / 2.0),
        ];

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(control_points[0]);

        // Draw top curve
        path_builder.quad_to(control_points[1], control_points[2]);
        // Draw to bottom curve start
        path_builder.line_to(control_points[3]);
        // Draw bottom curve
        path_builder.quad_to(control_points[4], control_points[5]);
        // Close the path
        path_builder.close();

        let path = path_builder.detach();
        self.canvas().draw_path(&path, &paint);
    }

    pub fn render_shape(&mut self, color: Color, points: &[(f32, f32)]) {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        paint.set_color(color);
        paint.set_style(PaintStyle::Fill);

        let mut path_builder = PathBuilder::new();
        let first = points[0];
        path_builder.move_to((first.0, first.1));

        for point in &points[1..] {
            path_builder.line_to((point.0, point.1));
        }
        let path = path_builder.detach();

        self.canvas().draw_path(&path, &paint);
    }
}

pub struct Renderer {
    converter: Converter,
    instructions: Vec<DrawInstruction>,
    width: f32,
    height: f32,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            converter: Converter::new(1.0, 1.0),
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn set_converter(&mut self, converter: Converter) {
        self.converter = converter;
    }

    pub fn set_dimensions(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn clear_instructions(&mut self) {
        self.instructions.clear();
    }

    pub fn line(&mut self, width: Space, color: Color, points: Vec<Point>) {
        let width_px = self.converter.spaces_to_px(width);
        let points_px = points
            .into_iter()
            .map(|p| {
                let skia_point = p.to_skia(&self.converter);
                (skia_point.x, skia_point.y)
            })
            .collect();

        self.instructions.push(DrawInstruction::Line {
            width: width_px,
            color,
            points: points_px,
        });
    }

    pub fn dashed_line(&mut self, width: Space, color: Color, points: Vec<Point>) {
        let width_px = self.converter.spaces_to_px(width);
        let interval_px = self.converter.spaces_to_px(4.0 / 7.0);
        let points_px = points
            .into_iter()
            .map(|p| {
                let skia_point = p.to_skia(&self.converter);
                (skia_point.x, skia_point.y)
            })
            .collect();

        self.instructions.push(DrawInstruction::DashedLine {
            width: width_px,
            color,
            points: points_px,
            interval: interval_px,
        });
    }

    pub fn text(&mut self, text: &str, font: &Font, point: &Point) {
        let font_px = self.converter.spaces_to_px(font.size);
        let skia_point = point.to_skia(&self.converter);
        let point_px = (skia_point.x, skia_point.y);

        self.instructions.push(DrawInstruction::Text {
            text: text.to_string(),
            font_size: font_px,
            font_family: font.family,
            justify: font.justify,
            align: font.align,
            point: point_px,
        });
    }

    pub fn circle(&mut self, radius: Space, color: Color, origin: &Point) {
        let radius_px = self.converter.spaces_to_px(radius);
        let skia_point = origin.to_skia(&self.converter);
        let origin_px = (skia_point.x, skia_point.y);

        self.instructions.push(DrawInstruction::Circle {
            radius: radius_px,
            color,
            origin: origin_px,
        });
    }

    pub fn curve(&mut self, color: Color, points: [CurvePoint; 3]) {
        let points_px = points.map(|p| {
            let skia_point = Point { x: p.x, y: p.y }.to_skia(&self.converter);
            (
                skia_point.x,
                skia_point.y,
                self.converter.spaces_to_px(p.thickness),
            )
        });

        self.instructions.push(DrawInstruction::Curve {
            color,
            points: points_px,
        });
    }

    pub fn shape(&mut self, color: Color, points: Vec<Point>) {
        let points_px = points
            .into_iter()
            .map(|p| {
                let skia_point = p.to_skia(&self.converter);
                (skia_point.x, skia_point.y)
            })
            .collect();

        self.instructions.push(DrawInstruction::Shape {
            color,
            points: points_px,
        });
    }

    pub fn data(&mut self) -> Vec<u8> {
        let mut renderer = CanvasRenderer::new(self.width as i32, self.height as i32);
        renderer.clear();

        // Render only instructions that intersect with the tile
        for instruction in &self.instructions {
            instruction.render(&mut renderer);
        }

        renderer.encode()
    }
}
