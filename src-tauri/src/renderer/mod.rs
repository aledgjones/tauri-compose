pub mod backend;
pub mod backends;
pub mod converter;
pub mod fonts;
pub mod hit_test;
pub mod text_measurer;

use crate::renderer::backend::RenderBackend;
use crate::renderer::backends::metal::MetalRenderer;
use crate::score::components::measurements::{CurvePoint, Point};
use crate::score::components::text::{Align, Font, FontFamily, Justify};
use crate::ui::sequencer::Selection;
use crate::RendererContextState;
use converter::{Converter, Space};
use skia_safe::Color;
use std::sync::Mutex;

fn constrain_dimension(pan: f32, scaled_size: f32, bounds_size: f32) -> f32 {
    if scaled_size + (100.0 * 2.0) <= bounds_size {
        (bounds_size - scaled_size) / 2.0
    } else {
        let max_pan = 100.0;
        let min_pan = -(scaled_size - bounds_size + 100.0);
        pan.clamp(min_pan, max_pan)
    }
}

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
        color: Color,
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
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: skia_safe::Color,
    },
}

#[derive(Clone)]
pub struct HitTarget {
    pub entry: Selection,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub popover_x: f32,
    pub popover_y: f32,
}

pub struct RendererContext {
    pub backend: Option<MetalRenderer>,
    pub converter: Converter,
    pub px_per_mm: f32,
    pub instructions: Vec<DrawInstruction>,
    pub width: f32,
    pub height: f32,
    pub hit_targets: Vec<HitTarget>,
    pub bounds: (f32, f32, f32, f32),
    pub pixel_density: f64,
}

impl RendererContext {
    pub fn new() -> RendererContextState {
        Mutex::new(RendererContext {
            backend: None,
            converter: Converter::new(1.0, 1.0),
            px_per_mm: 3.78,
            instructions: Vec::new(),
            width: 0.0,
            height: 0.0,
            hit_targets: Vec::new(),
            bounds: (0.0, 0.0, 0.0, 0.0),
            pixel_density: 1.0,
        })
    }

    pub fn clear(&mut self) {
        self.instructions.clear();
        self.hit_targets.clear();
    }

    pub fn constrain_pan(
        &self,
        pan_x: f32,
        pan_y: f32,
        page_width_spaces: f32,
        page_height_spaces: f32,
        zoom: f32,
    ) -> (f32, f32) {
        // Convert page dimensions from spaces to CSS pixels
        let page_width_px = self.converter.spaces_to_px(page_width_spaces);
        let page_height_px = self.converter.spaces_to_px(page_height_spaces);

        // Get bounds in CSS pixels
        let bounds_width = self.bounds.2;
        let bounds_height = self.bounds.3;

        // Calculate scaled page dimensions
        let scaled_page_width = page_width_px * zoom;
        let scaled_page_height = page_height_px * zoom;

        // Constrain horizontal and vertical pan
        let constrained_pan_x = constrain_dimension(pan_x, scaled_page_width, bounds_width);
        let constrained_pan_y = constrain_dimension(pan_y, scaled_page_height, bounds_height);

        (constrained_pan_x, constrained_pan_y)
    }

    pub fn render(
        &mut self,
        app: &tauri::AppHandle,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        let constrained_pan = self.constrain_pan(pan_x, pan_y, self.width, self.height, zoom);
        if let Some(backend) = &mut self.backend {
            backend.render(
                app,
                self.width,
                self.height,
                &self.instructions,
                &self.converter,
                &self.bounds,
                self.pixel_density,
                constrained_pan.0,
                constrained_pan.1,
                zoom,
            )?;
            Ok(constrained_pan)
        } else {
            Ok((pan_x, pan_y))
        }
    }

    pub fn set_backend(&mut self, backend: MetalRenderer) {
        self.backend = Some(backend);
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

    pub fn text(&mut self, text: &str, font: &Font, point: &Point, color: Color) {
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
            color,
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

    pub fn update_bounds(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.bounds = (x, y, width, height);
        self.pixel_density = scale;

        if let Some(backend) = &mut self.backend {
            let _ = backend.update_bounds(x, y, width, height, scale);
        }

        Ok(())
    }
}
