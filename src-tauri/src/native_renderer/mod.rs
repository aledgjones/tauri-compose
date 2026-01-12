#![allow(non_snake_case)]

use objc2::rc::{autoreleasepool, Retained};
use objc2::ClassType;
use objc2_foundation::NSObjectProtocol;
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLTexture};
use objc2_quartz_core::CAMetalDrawable;
use skia_safe::gpu;
use skia_safe::gpu::mtl;
use skia_safe::utils::text_utils::Align as SkiaAlign;
use skia_safe::{Color, Paint, PaintStyle, Path, Point as SkiaPoint, Shader, SurfaceProps};
use std::time::Instant;
use tauri::Emitter;

use crate::renderer::{DrawInstruction, Renderer};

#[allow(dead_code)]
pub struct NativeRenderer {
    context: gpu::DirectContext,
    metal_layer: objc2::rc::Retained<objc2_quartz_core::CAMetalLayer>,
    skia_view: objc2::rc::Retained<objc2_app_kit::NSView>,
    device: objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2_metal::MTLDevice>>,
    queue: objc2::rc::Retained<objc2::runtime::ProtocolObject<dyn objc2_metal::MTLCommandQueue>>,
    display_link: Option<objc2::rc::Retained<objc2_quartz_core::CADisplayLink>>,
    bounds: (f32, f32, f32, f32), // (x, y, width, height) in CSS pixels
    scale: f64,                   // Device pixel ratio (e.g., 2.0 on retina)
}

impl NativeRenderer {
    pub fn new(window: &tauri::WebviewWindow) -> Result<Self, Box<dyn std::error::Error>> {
        use objc2::rc::Retained;
        use objc2::MainThreadOnly;
        use objc2_app_kit::NSWindow;
        use objc2_foundation::{NSPoint, NSRect, NSSize};
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};
        use objc2_quartz_core::{CALayer, CAMetalLayer};

        let mtm = objc2::MainThreadMarker::new().ok_or("Not on main thread")?;

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
            mtl::BackendContext::new(
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
                        context: skia_safe::gpu::direct_contexts::make_metal(
                            &backend_context,
                            None,
                        )
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
        let skia_view: Retained<objc2_app_kit::NSView> =
            objc2_app_kit::NSView::initWithFrame(objc2_app_kit::NSView::alloc(mtm), full_frame);

        {
            metal_layer.setDevice(Some(&device));
            metal_layer.setPixelFormat(objc2_metal::MTLPixelFormat::BGRA8Unorm);
            metal_layer.setFramebufferOnly(false);
            metal_layer.setOpaque(false); // Enable transparency
            metal_layer.setPresentsWithTransaction(false); // Better compositing
            metal_layer.setAllowsNextDrawableTimeout(false);
            metal_layer.setContentsScale(scale);
            metal_layer.setFrame(full_frame);

            skia_view.setWantsLayer(true);
            let layer_ptr = Retained::as_ptr(&metal_layer) as *const CALayer;
            skia_view.setLayer(Some(unsafe { &*layer_ptr }));

            content_view.addSubview_positioned_relativeTo(
                &skia_view,
                objc2_app_kit::NSWindowOrderingMode(-1),
                None,
            );
        }

        // --- Create Skia Metal context ---
        let context = skia_safe::gpu::direct_contexts::make_metal(&backend_context, None)
            .ok_or("Failed to create Skia DirectContext")?;

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
    ) -> Result<(), Box<dyn std::error::Error>> {
        use objc2_foundation::{NSPoint, NSRect, NSSize};

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

        // Drawable size = CSS pixels × scale (physical pixels for retina)
        self.metal_layer
            .setDrawableSize(NSSize::new(width as f64 * scale, height as f64 * scale));

        // Store the bounds (CSS pixels) and scale
        self.bounds = (x, y, width, height);
        self.scale = scale;

        Ok(())
    }

    fn constrain_pan(
        &self,
        pan_x: f32,
        pan_y: f32,
        page_width_spaces: f32,
        page_height_spaces: f32,
        zoom: f32,
        converter: &crate::renderer::converter::Converter,
    ) -> (f32, f32) {
        let padding = 100.0; // CSS pixels

        // Convert page dimensions from spaces to CSS pixels using converter
        // Note: converter.spaces_to_px() includes a 2.0 multiplier for rendering,
        // so we divide by the actual scale to get actual CSS pixel dimensions
        let scale = self.scale as f32;
        let page_width_px = converter.spaces_to_px(page_width_spaces) / scale;
        let page_height_px = converter.spaces_to_px(page_height_spaces) / scale;

        // Get bounds in CSS pixels
        let bounds_width = self.bounds.2;
        let bounds_height = self.bounds.3;

        // Calculate scaled page dimensions
        let scaled_page_width = page_width_px * zoom;
        let scaled_page_height = page_height_px * zoom;

        // Constrain horizontal pan
        let constrained_pan_x = if scaled_page_width + (padding * 2.0) <= bounds_width {
            // Page with padding fits horizontally - center it
            (bounds_width - scaled_page_width) / 2.0
        } else {
            // Page larger than bounds - allow panning with padding
            let max_pan = padding;
            let min_pan = -(scaled_page_width - bounds_width + padding);
            pan_x.clamp(min_pan, max_pan)
        };

        // Constrain vertical pan
        let constrained_pan_y = if scaled_page_height + (padding * 2.0) <= bounds_height {
            // Page with padding fits vertically - center it
            (bounds_height - scaled_page_height) / 2.0
        } else {
            // Page larger than bounds - allow panning with padding
            let max_pan = padding;
            let min_pan = -(scaled_page_height - bounds_height + padding);
            pan_y.clamp(min_pan, max_pan)
        };

        (constrained_pan_x, constrained_pan_y)
    }

    pub fn render(
        &mut self,
        renderer: &Renderer,
        app: &tauri::AppHandle,
        pan_x: f32,
        pan_y: f32,
        zoom: f32,
    ) -> Result<(f32, f32), Box<dyn std::error::Error>> {
        let total_start = Instant::now();

        autoreleasepool(|_| -> Result<(f32, f32), Box<dyn std::error::Error>> {
            // Skip rendering if bounds are zero (used for screens that don't show rendered score)
            if self.bounds.2 == 0.0 || self.bounds.3 == 0.0 {
                return Ok((pan_x, pan_y));
            }

            let setup_start = Instant::now();

            if let Some(drawable) = self.metal_layer.nextDrawable() {
                let texture = drawable.texture();
                let texture_ptr = Retained::as_ptr(&texture) as *const std::ffi::c_void;

                // Get texture size from the drawable
                let texture_width = texture.width() as i32;
                let texture_height = texture.height() as i32;

                let backend_texture = unsafe { gpu::mtl::TextureInfo::new(texture_ptr as _) };
                let backend_render_target = gpu::backend_render_targets::make_mtl(
                    (texture_width, texture_height),
                    &backend_texture,
                );

                if let Some(mut surface) = gpu::surfaces::wrap_backend_render_target(
                    &mut self.context,
                    &backend_render_target,
                    gpu::SurfaceOrigin::TopLeft,
                    skia_safe::ColorType::BGRA8888,
                    skia_safe::ColorSpace::new_srgb(),
                    Some(&SurfaceProps::default()),
                ) {
                    let canvas = surface.canvas();

                    // Apply pan constraints based on page size
                    let (constrained_pan_x, constrained_pan_y) = self.constrain_pan(
                        pan_x,
                        pan_y,
                        renderer.width,
                        renderer.height,
                        zoom,
                        &renderer.converter,
                    );

                    // Clear the entire canvas with neutral-950 background
                    canvas.clear(Color::from_rgb(10, 10, 10));

                    // Draw gradient background over the entire view
                    let sky_700 = Color::from_rgb(3, 105, 161);
                    let sky_900 = Color::from_rgb(12, 74, 110);

                    let colors = [sky_700, sky_900];
                    let positions = [0.0f32, 1.0f32];
                    let gradient_shader = Shader::linear_gradient(
                        (
                            SkiaPoint::new(0.0, 0.0),
                            SkiaPoint::new(0.0, texture_height as f32),
                        ),
                        &colors[..],
                        Some(&positions[..]),
                        skia_safe::TileMode::Clamp,
                        None,
                        None,
                    );

                    let mut background_paint = Paint::default();
                    background_paint.set_shader(gradient_shader);
                    background_paint.set_style(PaintStyle::Fill);
                    canvas.draw_rect(
                        skia_safe::Rect::from_xywh(
                            0.0,
                            0.0,
                            texture_width as f32,
                            texture_height as f32,
                        ),
                        &background_paint,
                    );

                    // --- Draw instructions ---
                    canvas.save();

                    // Canvas is in physical pixels, pan is in CSS pixels
                    let scale_f32 = self.scale as f32;

                    // Apply constrained pan (scale from CSS to physical pixels)
                    canvas
                        .translate((constrained_pan_x * scale_f32, constrained_pan_y * scale_f32));

                    // Apply zoom scaling
                    canvas.scale((zoom, zoom));

                    // Clip to renderer bounds (scale bounds from CSS to physical pixels)
                    let clip_rect = skia_safe::Rect::from_xywh(
                        -constrained_pan_x * scale_f32 / zoom,
                        -constrained_pan_y * scale_f32 / zoom,
                        self.bounds.2 * scale_f32 / zoom,
                        self.bounds.3 * scale_f32 / zoom,
                    );
                    canvas.clip_rect(clip_rect, skia_safe::ClipOp::Intersect, true);

                    let setup_duration = setup_start.elapsed();
                    let draw_start = Instant::now();

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
                                canvas.draw_circle(
                                    SkiaPoint::new(origin.0, origin.1),
                                    *radius,
                                    &paint,
                                );
                            }
                            DrawInstruction::Shape { color, points } => {
                                if points.len() >= 3 {
                                    let mut paint = Paint::default();
                                    paint.set_color(*color);
                                    paint.set_anti_alias(true);
                                    paint.set_style(PaintStyle::Fill);
                                    let skia_points: Vec<SkiaPoint> =
                                        points.iter().map(|p| SkiaPoint::new(p.0, p.1)).collect();
                                    canvas.draw_path(
                                        &Path::polygon(&skia_points, true, None, None),
                                        &paint,
                                    );
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
                                canvas.draw_rect(
                                    skia_safe::Rect::from_xywh(*x, *y, *width, *height),
                                    &paint,
                                );
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

                                    let mut path_builder = skia_safe::PathBuilder::new();
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
                                    crate::score::components::text::FontFamily::Bravura => &crate::renderer::fonts::FONTS.bravura,
                                    crate::score::components::text::FontFamily::LibreBaskerville => &crate::renderer::fonts::FONTS.libre_baskerville,
                                };

                                // Create font with correct size
                                let font = skia_safe::Font::new(typeface, *font_size);

                                // Get font metrics for alignment
                                let (_, metrics) = font.metrics();

                                let justify_skia = match justify {
                                    crate::score::components::text::Justify::Start => {
                                        SkiaAlign::Left
                                    }
                                    crate::score::components::text::Justify::Middle => {
                                        SkiaAlign::Center
                                    }
                                    crate::score::components::text::Justify::End => {
                                        SkiaAlign::Right
                                    }
                                };

                                let offset_y = match align {
                                    crate::score::components::text::Align::Top => metrics.x_height,
                                    crate::score::components::text::Align::Middle => {
                                        metrics.x_height / 2.0
                                    }
                                    crate::score::components::text::Align::Bottom => 0.0,
                                };

                                let adjusted_point = SkiaPoint::new(point.0, point.1 + offset_y);

                                canvas.draw_text_align(
                                    text,
                                    adjusted_point,
                                    &font,
                                    &paint,
                                    justify_skia,
                                );
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
                                    paint.set_path_effect(skia_safe::PathEffect::dash(
                                        &intervals, 0.0,
                                    ));
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

                    // Draw scrollbars to indicate position in score
                    let page_width_px = renderer.converter.spaces_to_px(renderer.width) / scale_f32;
                    let page_height_px = renderer.converter.spaces_to_px(renderer.height) / scale_f32;
                    let scaled_page_width = page_width_px * zoom;
                    let scaled_page_height = page_height_px * zoom;

                    let scrollbar_width = 7.0 * scale_f32;
                    let scrollbar_padding = 4.0 * scale_f32;
                    let scrollbar_color = Color::from_argb(200, 60, 60, 60); // Semi-transparent dark gray

                    let padding = 100.0; // CSS pixels
                    let show_horizontal = scaled_page_width + (padding * 2.0) > self.bounds.2;
                    let show_vertical = scaled_page_height + (padding * 2.0) > self.bounds.3;

                    // Draw horizontal scrollbar if content is wider than viewport
                    if show_horizontal {
                        let visible_ratio = self.bounds.2 / (scaled_page_width + padding * 2.0);
                        let available_width = if show_vertical {
                            // Leave space for vertical scrollbar
                            self.bounds.2 * scale_f32 - scrollbar_width - scrollbar_padding
                        } else {
                            self.bounds.2 * scale_f32
                        };
                        let scrollbar_length =
                            (available_width - 2.0 * scrollbar_padding) * visible_ratio;

                        // Calculate position: map pan position to scrollbar position
                        let max_pan = 100.0; // padding
                        let min_pan = -(scaled_page_width - self.bounds.2 + 100.0);
                        let pan_range = max_pan - min_pan;
                        let pan_normalized = (constrained_pan_x - max_pan) / pan_range; // 0 at start, 1 at end
                        let scrollbar_range =
                            available_width - 2.0 * scrollbar_padding - scrollbar_length;
                        let scrollbar_x = scrollbar_padding + scrollbar_range * -pan_normalized;

                        let scrollbar_y =
                            texture_height as f32 - scrollbar_width - scrollbar_padding;

                        let mut scrollbar_paint = Paint::default();
                        scrollbar_paint.set_color(scrollbar_color);
                        scrollbar_paint.set_anti_alias(true);
                        scrollbar_paint.set_style(PaintStyle::Fill);

                        canvas.draw_round_rect(
                            skia_safe::Rect::from_xywh(
                                scrollbar_x,
                                scrollbar_y,
                                scrollbar_length,
                                scrollbar_width,
                            ),
                            scrollbar_width / 2.0,
                            scrollbar_width / 2.0,
                            &scrollbar_paint,
                        );
                    }

                    // Draw vertical scrollbar if content is taller than viewport
                    if show_vertical {
                        let visible_ratio = self.bounds.3 / (scaled_page_height + padding * 2.0);
                        let available_height = if show_horizontal {
                            // Leave space for horizontal scrollbar
                            self.bounds.3 * scale_f32 - scrollbar_width - scrollbar_padding
                        } else {
                            self.bounds.3 * scale_f32
                        };
                        let scrollbar_length =
                            (available_height - 2.0 * scrollbar_padding) * visible_ratio;

                        // Calculate position: map pan position to scrollbar position
                        let max_pan = 100.0; // padding
                        let min_pan = -(scaled_page_height - self.bounds.3 + 100.0);
                        let pan_range = max_pan - min_pan;
                        let pan_normalized = (constrained_pan_y - max_pan) / pan_range; // 0 at start, 1 at end
                        let scrollbar_range =
                            available_height - 2.0 * scrollbar_padding - scrollbar_length;
                        let scrollbar_y = scrollbar_padding + scrollbar_range * -pan_normalized;

                        let scrollbar_x =
                            texture_width as f32 - scrollbar_width - scrollbar_padding;

                        let mut scrollbar_paint = Paint::default();
                        scrollbar_paint.set_color(scrollbar_color);
                        scrollbar_paint.set_anti_alias(true);
                        scrollbar_paint.set_style(PaintStyle::Fill);

                        canvas.draw_round_rect(
                            skia_safe::Rect::from_xywh(
                                scrollbar_x,
                                scrollbar_y,
                                scrollbar_width,
                                scrollbar_length,
                            ),
                            scrollbar_width / 2.0,
                            scrollbar_width / 2.0,
                            &scrollbar_paint,
                        );
                    }

                    let draw_duration = draw_start.elapsed();
                    let flush_start = Instant::now();

                    // Flush the surface to ensure all drawing is complete
                    self.context.flush_and_submit();

                    let command_buffer = self
                        .queue
                        .commandBuffer()
                        .ok_or("Failed to create command buffer")?;
                    let mtl_drawable: &objc2::runtime::ProtocolObject<
                        dyn objc2_metal::MTLDrawable,
                    > = unsafe { std::mem::transmute(&*drawable) };
                    command_buffer.presentDrawable(mtl_drawable);
                    command_buffer.commit();

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

                    return Ok((constrained_pan_x, constrained_pan_y));
                }
            }
            Ok((pan_x, pan_y))
        })
    }
}

// Unsafe impl to allow NativeRenderer to be Send + Sync
// This is safe because NativeRenderer is only used on the main thread
unsafe impl Send for NativeRenderer {}
unsafe impl Sync for NativeRenderer {}
