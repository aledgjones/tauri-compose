pub mod converter;
pub mod fonts;
pub mod text_measurer;

use crate::score::components::measurements::{CurvePoint, Point};
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use converter::{Converter, Space};

#[derive(Clone)]
pub enum DrawInstruction {
    Line {
        width: f32,
        color: skia_safe::Color,
        points: Vec<(f32, f32)>,
    },
    DashedLine {
        width: f32,
        color: skia_safe::Color,
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
        color: skia_safe::Color,
        origin: (f32, f32),
    },
    Curve {
        color: skia_safe::Color,
        points: [(f32, f32, f32); 3], // (x, y, thickness)
    },
    Shape {
        color: skia_safe::Color,
        points: Vec<(f32, f32)>,
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: skia_safe::Color,
    },
}

#[derive(Clone)]
pub struct Renderer {
    converter: Converter,
    pub px_per_mm: f32,
    pub instructions: Vec<DrawInstruction>,
    width: f32,
    height: f32,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            converter: Converter::new(1.0, 1.0),
            px_per_mm: 3.78,
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

    pub fn line(&mut self, width: Space, color: skia_safe::Color, points: Vec<Point>) {
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

    pub fn dashed_line(&mut self, width: Space, color: skia_safe::Color, points: Vec<Point>) {
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

    pub fn circle(&mut self, radius: Space, color: skia_safe::Color, origin: &Point) {
        let radius_px = self.converter.spaces_to_px(radius);
        let skia_point = origin.to_skia(&self.converter);
        let origin_px = (skia_point.x, skia_point.y);

        self.instructions.push(DrawInstruction::Circle {
            radius: radius_px,
            color,
            origin: origin_px,
        });
    }

    pub fn curve(&mut self, color: skia_safe::Color, points: [CurvePoint; 3]) {
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

    pub fn shape(&mut self, color: skia_safe::Color, points: Vec<Point>) {
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

    pub fn rectangle(
        &mut self,
        x: Space,
        y: Space,
        width: Space,
        height: Space,
        color: skia_safe::Color,
    ) {
        let x_px = self.converter.spaces_to_px(x);
        let y_px = self.converter.spaces_to_px(y);
        let width_px = self.converter.spaces_to_px(width);
        let height_px = self.converter.spaces_to_px(height);

        self.instructions.push(DrawInstruction::Rectangle {
            x: x_px,
            y: y_px,
            width: width_px,
            height: height_px,
            color,
        });
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
