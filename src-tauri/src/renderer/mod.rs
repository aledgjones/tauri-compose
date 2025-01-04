use skia_safe::{surfaces, Color, EncodedImageFormat, Paint, Path, Point, Surface};

pub struct Renderer {
    surface: Surface,
}

impl Renderer {
    pub fn new(width: i32, height: i32) -> Renderer {
        let mut surface = surfaces::raster_n32_premul((width, height)).expect("surface");
        surface.canvas().clear(Color::WHITE);
        Renderer { surface }
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

    pub fn data(&mut self) -> Vec<u8> {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        match image.encode(context.as_mut(), EncodedImageFormat::WEBP, None) {
            Some(data) => data.as_bytes().to_vec(),
            None => vec![],
        }
    }
}

#[tauri::command(rename_all = "snake_case")]
pub fn render() -> Vec<u8> {
    let mut renderer = Renderer::new(1500, 400);
    renderer.line(vec![Point::new(100.0, 100.5), Point::new(200.0, 100.5)]);
    renderer.data()
}
