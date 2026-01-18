#![allow(non_snake_case)]

use crate::renderer::{converter::Converter, fonts::FONTS, DrawInstruction, Renderer};
use crate::score::components::text::{Align, FontFamily, Justify};
use objc2::rc::{autoreleasepool, Retained};
use objc2::runtime::ProtocolObject;
use objc2::{ClassType, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{NSView, NSWindow, NSWindowOrderingMode};
use objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize};
use objc2_metal::{
    MTLCommandBuffer, MTLCommandQueue, MTLCreateSystemDefaultDevice, MTLDevice, MTLPixelFormat,
    MTLTexture,
};
use objc2_quartz_core::{CALayer, CAMetalDrawable, CAMetalLayer};
use skia_safe::gpu::backend_render_targets::make_mtl;
use skia_safe::gpu::direct_contexts::make_metal;
use skia_safe::gpu::mtl::{BackendContext, TextureInfo};
use skia_safe::gpu::surfaces::wrap_backend_render_target;
use skia_safe::gpu::{DirectContext, SurfaceOrigin};
use skia_safe::utils::text_utils::Align as SkiaAlign;
use skia_safe::{
    Canvas, ClipOp, Color, ColorSpace, ColorType, Font, Paint, PaintStyle, Path, PathBuilder,
    PathEffect, Point as SkiaPoint, Rect, Shader, Surface, SurfaceProps, TileMode,
};
use std::error::Error;
use std::time::Instant;
use tauri::{Emitter, WebviewWindow};

const PADDING: f32 = 100.0;
const SCROLLBAR_WIDTH: f32 = 7.0;
const SCROLLBAR_PADDING: f32 = 4.0;
const SCROLLBAR_COLOR: Color = Color::from_argb(200, 60, 60, 60);
const BACKGROUND_COLOR: Color = Color::from_rgb(10, 10, 10);
const GRADIENT_COLORS: [Color; 4] = [
    Color::from_rgb(2, 94, 149),
    Color::from_rgb(3, 161, 222),
    Color::from_rgb(107, 199, 236),
    Color::from_rgb(180, 218, 221),
];
const GRADIENT_POSITIONS: [f32; 4] = [0.0, 0.33, 0.66, 1.0];

// Type aliases for cleaner code
type MetalDevice = Retained<ProtocolObject<dyn objc2_metal::MTLDevice>>;
type MetalCommandQueue = Retained<ProtocolObject<dyn objc2_metal::MTLCommandQueue>>;
type MetalDrawable = Retained<ProtocolObject<dyn objc2_metal::MTLDrawable>>;
type CAMetalLayerRetained = Retained<objc2_quartz_core::CAMetalLayer>;
type NSViewRetained = Retained<objc2_app_kit::NSView>;
type CADisplayLinkRetained = Retained<objc2_quartz_core::CADisplayLink>;

fn constrain_dimension(pan: f32, scaled_size: f32, bounds_size: f32) -> f32 {
    if scaled_size + (PADDING * 2.0) <= bounds_size {
        (bounds_size - scaled_size) / 2.0
    } else {
        let max_pan = PADDING;
        let min_pan = -(scaled_size - bounds_size + PADDING);
        pan.clamp(min_pan, max_pan)
    }
}

fn draw_scrollbar(canvas: &Canvas, x: f32, y: f32, width: f32, height: f32) {
    let mut paint = Paint::default();
    paint.set_color(SCROLLBAR_COLOR);
    paint.set_anti_alias(true);
    paint.set_style(PaintStyle::Fill);
    canvas.draw_round_rect(
        Rect::from_xywh(x, y, width, height),
        SCROLLBAR_WIDTH / 2.0,
        SCROLLBAR_WIDTH / 2.0,
        &paint,
    );
}

#[allow(dead_code)]
pub struct NativeRenderer {
    context: DirectContext,
    metal_layer: CAMetalLayerRetained,
    skia_view: NSViewRetained,
    device: MetalDevice,
    queue: MetalCommandQueue,
    display_link: Option<CADisplayLinkRetained>,
    bounds: (f32, f32, f32, f32), // (x, y, width, height) in CSS pixels
    scale: f64,                   // Device pixel ratio (e.g., 2.0 on retina)
}

impl NativeRenderer {
    pub fn new(window: &WebviewWindow) -> Result<Self, Box<dyn Error>> {
        let mtm = MainThreadMarker::new().ok_or("Not on main thread")?;

        // --- Create Metal device + queue ---
        let device = MTLCreateSystemDefaultDevice().ok_or("Failed to create Metal device")?;
        let queue = device
            .newCommandQueue()
            .ok_or("Failed to create Metal command queue")?;

        // --- Get NSWindow from Tauri ---
        let ns_window = window
            .ns_window()
            .map_err(|e| format!("Failed to get NSWindow: {}", e))?
            as *mut NSWindow;
        let ns_window =
            unsafe { Retained::retain(ns_window) }.ok_or("Failed to retain NSWindow")?;
        let content_view = ns_window.contentView().ok_or("Failed to get contentView")?;
        let scale = window.scale_factor().unwrap_or(1.0);

        // --- Create Skia Metal backend context first ---
        let backend_context = unsafe {
            BackendContext::new(
                Retained::as_ptr(&device) as _,
                Retained::as_ptr(&queue) as _,
            )
        };

        // --- Check for existing Metal layer ---
        let subviews = content_view.subviews();
        for subview in subviews {
            if let Some(layer) = subview.layer() {
                if layer.isKindOfClass(CAMetalLayer::class()) {
                    // Found existing Metal layer, reuse it
                    let metal_layer = layer
                        .downcast::<CAMetalLayer>()
                        .map_err(|_| "Failed to downcast to CAMetalLayer")?;
                    return Ok(Self {
                        context: make_metal(&backend_context, None)
                            .ok_or("Failed to create Skia DirectContext")?,
                        metal_layer,
                        skia_view: subview.clone(),
                        device: device.clone(),
                        queue: queue.clone(),
                        display_link: None,
                        bounds: (0.0, 0.0, 0.0, 0.0),
                        scale,
                    });
                }
            }
        }
        let metal_layer = CAMetalLayer::new();
        let _size = window.inner_size()?;

        // Cover full content view for now
        let _frame = content_view.frame();
        let full_frame = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(_frame.size.width, _frame.size.height),
        );

        // --- Create skia view with full frame ---
        let skia_view: Retained<NSView> = NSView::initWithFrame(NSView::alloc(mtm), full_frame);

        {
            metal_layer.setDevice(Some(&device));
            metal_layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            metal_layer.setFramebufferOnly(false);
            metal_layer.setOpaque(true);
            metal_layer.setPresentsWithTransaction(false); // Better compositing
            metal_layer.setAllowsNextDrawableTimeout(false);
            metal_layer.setContentsScale(scale);
            metal_layer.setFrame(full_frame);

            skia_view.setWantsLayer(true);
            let layer_ptr = Retained::as_ptr(&metal_layer) as *const CALayer;
            skia_view.setLayer(Some(unsafe { &*layer_ptr }));

            content_view.addSubview_positioned_relativeTo(
                &skia_view,
                NSWindowOrderingMode(-1),
                None,
            );
        }

        // --- Create Skia Metal context ---
        let context =
            make_metal(&backend_context, None).ok_or("Failed to create Skia DirectContext")?;

        Ok(Self {
            context,
            metal_layer,
            skia_view,
            device,
            queue,
            display_link: None,
            bounds: (0.0, 0.0, 0.0, 0.0),
            scale,
        })
    }

    pub fn update_bounds(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        scale: f64,
    ) -> Result<(), Box<dyn Error>> {
        // Bounds come in as CSS pixels (points) - use directly
        // JS coordinates: (0,0) is top-left, Y increases downward
        // AppKit coordinates: (0,0) is bottom-left, Y increases upward
        let content_view =
            unsafe { self.skia_view.superview() }.ok_or("Failed to get superview")?;
        let content_frame = (*content_view).frame();
        let content_height = content_frame.size.height;

        // Convert web coordinates to AppKit coordinates
        let appkit_x = x as f64;
        let appkit_y = content_height - y as f64 - height as f64;

        // Update the view frame (in CSS pixels/points)
        let frame = NSRect::new(
            NSPoint::new(appkit_x, appkit_y),
            NSSize::new(width as f64, height as f64),
        );

        self.skia_view.setFrame(frame);
        self.metal_layer.setFrame(frame);
        self.metal_layer.setContentsScale(scale);

        self.metal_layer
            .setDrawableSize(NSSize::new(width as f64 * scale, height as f64 * scale));

        // Store the bounds (CSS pixels) and scale
        self.bounds = (x, y, width, height);
        self.scale = scale;

        Ok(())
    }

    pub fn get_bounds(&self) -> (f32, f32, f32, f32) {
        self.bounds
    }

    pub fn constrain_pan(
        &self,
        pan_x: f32,
        pan_y: f32,
        page_width_spaces: f32,
        page_height_spaces: f32,
        zoom: f32,
        converter: &Converter,
    ) -> (f32, f32) {
        // Convert page dimensions from spaces to CSS pixels
        let page_width_px = converter.spaces_to_px(page_width_spaces);
        let page_height_px = converter.spaces_to_px(page_height_spaces);

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

    fn draw_scrollbars(
        &self,
        canvas: &Canvas,
        renderer: &Renderer,
        constrained_pan_x: f32,
        constrained_pan_y: f32,
        zoom: f32,
    ) {
        canvas.save();
        canvas.scale((self.scale as f32, self.scale as f32));

        let page_width_px = renderer.converter.spaces_to_px(renderer.width);
        let page_height_px = renderer.converter.spaces_to_px(renderer.height);
        let scaled_page_width = page_width_px * zoom;
        let scaled_page_height = page_height_px * zoom;

        let show_horizontal = scaled_page_width + (PADDING * 2.0) > self.bounds.2;
        let show_vertical = scaled_page_height + (PADDING * 2.0) > self.bounds.3;

        // Draw horizontal scrollbar if content is wider than viewport
        if show_horizontal {
            let visible_ratio = self.bounds.2 / (scaled_page_width + PADDING * 2.0);
            let available_width = if show_vertical {
                self.bounds.2 - SCROLLBAR_WIDTH - SCROLLBAR_PADDING
            } else {
                self.bounds.2
            };
            let scrollbar_length = (available_width - 2.0 * SCROLLBAR_PADDING) * visible_ratio;

            // Calculate position: map pan position to scrollbar position
            let max_pan = PADDING;
            let min_pan = -(scaled_page_width - self.bounds.2 + PADDING);
            let pan_range = max_pan - min_pan;
            let pan_normalized = (max_pan - constrained_pan_x) / pan_range; // 0 at left, 1 at right
            let scrollbar_range = available_width - 2.0 * SCROLLBAR_PADDING - scrollbar_length;
            let scrollbar_x = SCROLLBAR_PADDING + scrollbar_range * pan_normalized;
            let scrollbar_y = self.bounds.3 - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

            draw_scrollbar(
                canvas,
                scrollbar_x,
                scrollbar_y,
                scrollbar_length,
                SCROLLBAR_WIDTH,
            );
        }

        // Draw vertical scrollbar if content is taller than viewport
        if show_vertical {
            let visible_ratio = self.bounds.3 / (scaled_page_height + PADDING * 2.0);
            let available_height = if show_horizontal {
                // Leave space for horizontal scrollbar
                self.bounds.3 - SCROLLBAR_WIDTH - SCROLLBAR_PADDING
            } else {
                self.bounds.3
            };
            let scrollbar_length = (available_height - 2.0 * SCROLLBAR_PADDING) * visible_ratio;

            // Calculate position: map pan position to scrollbar position
            let max_pan = PADDING;
            let min_pan = -(scaled_page_height - self.bounds.3 + PADDING);
            let pan_range = max_pan - min_pan;
            let pan_normalized = (max_pan - constrained_pan_y) / pan_range; // 0 at top, 1 at bottom
            let scrollbar_range = available_height - 2.0 * SCROLLBAR_PADDING - scrollbar_length;
            let scrollbar_y = SCROLLBAR_PADDING + scrollbar_range * pan_normalized;
            let scrollbar_x = self.bounds.2 - SCROLLBAR_WIDTH - SCROLLBAR_PADDING;

            draw_scrollbar(
                canvas,
                scrollbar_x,
                scrollbar_y,
                SCROLLBAR_WIDTH,
                scrollbar_length,
            );
        }

        canvas.restore();
    }

    pub fn render(
        &mut self,
        renderer: &Renderer,
        app: &tauri::AppHandle,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(f32, f32), Box<dyn Error>> {
        let total_start = Instant::now();

        autoreleasepool(|_| -> Result<(f32, f32), Box<dyn Error>> {
            // Skip rendering if bounds are zero (used for screens that don't show rendered score)
            if self.bounds.2 == 0.0 || self.bounds.3 == 0.0 {
                return Ok((pan_x, pan_y));
            }

            let (constrained_pan_x, constrained_pan_y) = self.constrain_pan(
                pan_x,
                pan_y,
                renderer.width,
                renderer.height,
                zoom,
                &renderer.converter,
            );

            let setup_start = Instant::now();

            if let Some((mut surface, texture_width, texture_height, drawable)) =
                self.setup_surface()
            {
                let setup_duration = setup_start.elapsed();
                let draw_start = Instant::now();
                let canvas = surface.canvas();
                self.draw_background(canvas, texture_width, texture_height);
                self.render_instructions(
                    canvas,
                    renderer,
                    constrained_pan_x,
                    constrained_pan_y,
                    zoom,
                );
                self.draw_scrollbars(canvas, renderer, constrained_pan_x, constrained_pan_y, zoom);
                let draw_duration = draw_start.elapsed();
                let flush_start = Instant::now();
                self.flush_and_present(drawable)?;
                let flush_duration = flush_start.elapsed();
                let total_duration = total_start.elapsed();
                // Emit timing events for native rendering
                let _ = app.emit(
                    "native-render-timing-setup",
                    setup_duration.as_millis() as u64,
                );
                let _ = app.emit(
                    "native-render-timing-draw",
                    draw_duration.as_millis() as u64,
                );
                let _ = app.emit(
                    "native-render-timing-flush",
                    flush_duration.as_millis() as u64,
                );
                let _ = app.emit(
                    "native-render-timing-total",
                    total_duration.as_millis() as u64,
                );
                Ok((constrained_pan_x, constrained_pan_y))
            } else {
                Ok((pan_x, pan_y))
            }
        })
    }

    fn setup_surface(&mut self) -> Option<(Surface, i32, i32, MetalDrawable)> {
        if let Some(drawable) = self.metal_layer.nextDrawable() {
            let texture = drawable.texture();
            let texture_ptr = Retained::as_ptr(&texture) as *const std::ffi::c_void;
            let texture_width = texture.width() as i32;
            let texture_height = texture.height() as i32;
            let backend_texture = unsafe { TextureInfo::new(texture_ptr as _) };
            let backend_render_target = make_mtl((texture_width, texture_height), &backend_texture);
            if let Some(surface) = wrap_backend_render_target(
                &mut self.context,
                &backend_render_target,
                SurfaceOrigin::TopLeft,
                ColorType::BGRA8888,
                ColorSpace::new_srgb(),
                Some(&SurfaceProps::default()),
            ) {
                let drawable = unsafe {
                    std::mem::transmute::<
                        Retained<ProtocolObject<dyn CAMetalDrawable>>,
                        MetalDrawable,
                    >(drawable)
                };
                Some((surface, texture_width, texture_height, drawable))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn draw_background(&self, canvas: &Canvas, texture_width: i32, texture_height: i32) {
        canvas.clear(BACKGROUND_COLOR);
        let colors = GRADIENT_COLORS;
        let positions = GRADIENT_POSITIONS;
        let gradient_shader = Shader::linear_gradient(
            (
                SkiaPoint::new(0.0, 0.0),
                SkiaPoint::new(0.0, texture_height as f32),
            ),
            &colors[..],
            Some(&positions[..]),
            TileMode::Clamp,
            None,
            None,
        );
        let mut background_paint = Paint::default();
        background_paint.set_shader(gradient_shader);
        background_paint.set_style(PaintStyle::Fill);
        canvas.draw_rect(
            Rect::from_xywh(0.0, 0.0, texture_width as f32, texture_height as f32),
            &background_paint,
        );
    }

    fn render_instructions(
        &self,
        canvas: &Canvas,
        renderer: &Renderer,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) {
        canvas.save();
        let scale_f32 = self.scale as f32;
        canvas.scale((scale_f32, scale_f32));
        canvas.translate((pan_x, pan_y));
        canvas.scale((zoom, zoom));
        let clip_rect = Rect::from_xywh(
            -pan_x / zoom,
            -pan_y / zoom,
            self.bounds.2 / zoom,
            self.bounds.3 / zoom,
        );
        canvas.clip_rect(clip_rect, ClipOp::Intersect, true);
        for instruction in &renderer.instructions {
            match instruction {
                DrawInstruction::Line {
                    width,
                    color,
                    points,
                } => {
                    if points.len() >= 2 {
                        let mut paint = Paint::default();
                        paint.set_color(*color);
                        paint.set_anti_alias(true);
                        paint.set_stroke_width(*width);
                        paint.set_style(PaintStyle::Stroke);
                        for i in 0..points.len() - 1 {
                            canvas.draw_line(
                                SkiaPoint::new(points[i].0, points[i].1),
                                SkiaPoint::new(points[i + 1].0, points[i + 1].1),
                                &paint,
                            );
                        }
                    }
                }
                DrawInstruction::Circle {
                    radius,
                    color,
                    origin,
                } => {
                    let mut paint = Paint::default();
                    paint.set_anti_alias(true);
                    paint.set_color(*color);
                    paint.set_style(PaintStyle::Fill);
                    canvas.draw_circle(SkiaPoint::new(origin.0, origin.1), *radius, &paint);
                }
                DrawInstruction::Shape { color, points } => {
                    if points.len() >= 3 {
                        let mut paint = Paint::default();
                        paint.set_color(*color);
                        paint.set_anti_alias(true);
                        paint.set_style(PaintStyle::Fill);
                        let skia_points: Vec<SkiaPoint> =
                            points.iter().map(|p| SkiaPoint::new(p.0, p.1)).collect();
                        canvas.draw_path(&Path::polygon(&skia_points, true, None, None), &paint);
                    }
                }
                DrawInstruction::Rectangle {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    let mut paint = Paint::default();
                    paint.set_color(*color);
                    paint.set_anti_alias(true);
                    paint.set_style(PaintStyle::Fill);
                    canvas.draw_rect(Rect::from_xywh(*x, *y, *width, *height), &paint);
                }
                DrawInstruction::Curve { color, points } => {
                    if points.len() == 3 {
                        let mut paint = Paint::default();
                        paint.set_color(*color);
                        paint.set_anti_alias(true);
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
                        canvas.draw_path(&path, &paint);
                    }
                }
                DrawInstruction::Text {
                    text,
                    font_size,
                    font_family,
                    justify,
                    align,
                    point,
                    color,
                } => {
                    let mut paint = Paint::default();
                    paint.set_color(*color);
                    paint.set_anti_alias(true);
                    // Get the correct typeface
                    let typeface = match font_family {
                        FontFamily::Bravura => &FONTS.bravura,
                        FontFamily::LibreBaskerville => &FONTS.libre_baskerville,
                    };
                    // Create font with correct size
                    let font = Font::new(typeface, *font_size);
                    // Get font metrics for alignment
                    let (_, metrics) = font.metrics();
                    let justify_skia = match justify {
                        Justify::Start => SkiaAlign::Left,
                        Justify::Middle => SkiaAlign::Center,
                        Justify::End => SkiaAlign::Right,
                    };
                    let offset_y = match align {
                        Align::Top => metrics.x_height,
                        Align::Middle => metrics.x_height / 2.0,
                        Align::Bottom => 0.0,
                    };
                    let adjusted_point = SkiaPoint::new(point.0, point.1 + offset_y);
                    canvas.draw_text_align(text, adjusted_point, &font, &paint, justify_skia);
                }
                DrawInstruction::DashedLine {
                    width,
                    color,
                    points,
                    interval,
                } => {
                    if points.len() >= 2 {
                        let mut paint = Paint::default();
                        paint.set_color(*color);
                        paint.set_anti_alias(true);
                        paint.set_stroke_width(*width);
                        paint.set_style(PaintStyle::Stroke);
                        let intervals = [*interval, *interval];
                        paint.set_path_effect(PathEffect::dash(&intervals, 0.0));
                        for i in 0..points.len() - 1 {
                            canvas.draw_line(
                                SkiaPoint::new(points[i].0, points[i].1),
                                SkiaPoint::new(points[i + 1].0, points[i + 1].1),
                                &paint,
                            );
                        }
                    }
                }
            }
        }
        canvas.restore();
    }

    fn flush_and_present(&mut self, drawable: MetalDrawable) -> Result<(), Box<dyn Error>> {
        self.context.flush_and_submit();
        let command_buffer = self
            .queue
            .commandBuffer()
            .ok_or("Failed to create command buffer")?;
        command_buffer.presentDrawable(&drawable);
        command_buffer.commit();
        Ok(())
    }
}

// Unsafe impl to allow NativeRenderer to be Send + Sync
// This is safe because NativeRenderer is only used on the main thread
unsafe impl Send for NativeRenderer {}
unsafe impl Sync for NativeRenderer {}
