use iced::advanced::image;
use iced::mouse::ScrollDelta;
use iced::widget::{self, button};
use iced::widget::{center, column, mouse_area};
use iced::Element;
use iced::{Point, Rectangle, Size, Subscription};
use num::complex::Complex;
use rayon::prelude::*;

fn main() -> iced::Result {
    iced::application(
        MandelbrotViewer::new,
        MandelbrotViewer::update,
        MandelbrotViewer::view,
    )
    .subscription(MandelbrotViewer::subscription)
    .run()
}

struct MandelbrotImage {
    current_region: Rectangle<f64>,
    image: image::Handle,
}

impl MandelbrotImage {
    fn new() -> Self {
        let starting_region: Rectangle<f64> = {
            let top_left = Point::new(-2.0, 1.0);
            let size = Size::new(3.0, 2.0);
            Rectangle {
                x: top_left.x,
                y: top_left.y,
                width: size.width,
                height: size.height,
            }
        };
        Self {
            current_region: starting_region,
            image: mandelbrot_image(starting_region),
        }
    }
}

fn translate_image_space_to_set_space(
    image_space: &Rectangle<f32>,
    max_size: &Size<f32>,
    current_set_space: &Rectangle<f64>,
) -> Rectangle<f64> {
    let relative_x_position = image_space.x as f64 / max_size.width as f64;
    let relative_y_position = image_space.y as f64 / max_size.height as f64;
    let relative_width = image_space.width as f64 / max_size.width as f64;
    let relative_height = image_space.height as f64 / max_size.height as f64;

    Rectangle {
        x: current_set_space.x + current_set_space.width * relative_x_position,
        y: current_set_space.y - current_set_space.height * relative_y_position,
        width: relative_width * current_set_space.width,
        height: relative_height * current_set_space.height,
    }
}

struct PanningState {
    cursor_position: Point,
    panning: bool,
    pan_start: Point,
}

impl PanningState {
    fn new() -> Self {
        Self {
            cursor_position: Point::default(),
            panning: false,
            pan_start: Point::default(),
        }
    }

    fn update_cursor_position(&mut self, cursor_position: Point) {
        self.cursor_position = cursor_position;
    }

    fn start_panning(&mut self) {
        self.panning = true;
        self.pan_start = self.cursor_position;
    }

    fn stop_panning(&mut self) {
        self.panning = false;
    }

    fn pan_region(&mut self, region: &mut Rectangle<f32>, max_size: &Size<f32>) {
        let offset = self.pan_start - self.cursor_position;
        let new_left = region.x + offset.x;
        let new_right = region.x + region.width + offset.x;
        let new_top = region.y + offset.y;
        let new_bottom = region.y + region.height + offset.y;

        if new_left >= 0.0 && new_right <= max_size.width {
            region.x += offset.x;
        }

        if new_top >= 0.0 && new_bottom <= max_size.height {
            region.y += offset.y;
        }

        self.pan_start = self.cursor_position;
    }
}

#[derive(Debug, Clone)]
enum Message {
    Regenerate,
    Zoom(ScrollDelta),
    CursorMoved(Point),
    StartPanning,
    StopPanning,
}

struct ImageViewer {
    panning_state: PanningState,
    image_region: Rectangle<f32>,
    max_size: Size<f32>,
}

impl ImageViewer {
    fn new() -> Self {
        Self {
            panning_state: PanningState::new(),
            image_region: Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1800.0,
                height: 1200.0,
            },
            max_size: Size {
                width: 1800.0,
                height: 1200.0,
            },
        }
    }

    fn reset_viewer_size(&mut self) {
        self.image_region = Rectangle {
            x: 0.0,
            y: 0.0,
            width: self.max_size.width,
            height: self.max_size.height,
        }
    }

    fn zoom_in(&mut self) {
        let region = &mut self.image_region;
        let width_to_remove = region.width * 0.10;
        let height_to_remove = region.height * 0.10;
        region.x += width_to_remove / 2.0;
        region.y += height_to_remove / 2.0;
        region.width -= width_to_remove;
        region.height -= height_to_remove;
    }

    fn zoom_out(&mut self) {
        let region = &mut self.image_region;
        let width_to_add = region.width * 0.10;
        let height_to_add = region.height * 0.10;

        let mut x_bounded = false;
        let mut y_bounded = false;
        let mut width_bounded = false;
        let mut height_bounded = false;

        if region.x > width_to_add / 2.0 {
            region.x -= width_to_add / 2.0;
        } else {
            x_bounded = true;
        }

        if region.y > height_to_add / 2.0 {
            region.y -= height_to_add / 2.0;
        } else {
            y_bounded = true;
        }

        if region.width + width_to_add < self.max_size.width {
            region.width += width_to_add;
        } else {
            width_bounded = true;
        }

        if region.height + height_to_add < self.max_size.height {
            region.height += height_to_add;
        } else {
            height_bounded = true;
        }

        if x_bounded && y_bounded && width_bounded && height_bounded {
            region.x = 0.0;
            region.y = 0.0;
            region.width = self.max_size.width;
            region.height = self.max_size.height;
        }
    }

    // not sure about the duplication/fake encapsulation of the state here

    fn update_cursor_position(&mut self, cursor_position: Point) {
        self.panning_state.update_cursor_position(cursor_position);
    }

    fn start_panning(&mut self) {
        self.panning_state.start_panning();
    }

    fn stop_panning(&mut self) {
        self.panning_state.stop_panning();
    }

    fn pan_region(&mut self) {
        self.panning_state
            .pan_region(&mut self.image_region, &self.max_size);
    }

    fn panning(&self) -> bool {
        self.panning_state.panning
    }

    fn image_region(&self) -> Rectangle<u32> {
        Rectangle {
            x: self.image_region.x as u32,
            y: self.image_region.y as u32,
            width: self.image_region.width as u32,
            height: self.image_region.height as u32,
        }
    }
}

struct MandelbrotViewer {
    mandelbrot_image: MandelbrotImage,
    image_viewer: ImageViewer,
}

impl MandelbrotViewer {
    fn new() -> Self {
        Self {
            mandelbrot_image: MandelbrotImage::new(),
            image_viewer: ImageViewer::new(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Regenerate => {
                let new_set_space = translate_image_space_to_set_space(
                    &self.image_viewer.image_region,
                    &self.image_viewer.max_size,
                    &self.mandelbrot_image.current_region,
                );
                self.mandelbrot_image.image = mandelbrot_image(new_set_space);
                self.mandelbrot_image.current_region = new_set_space;
                self.image_viewer.reset_viewer_size();
            }
            Message::Zoom(scroll_delta) => match scroll_delta {
                ScrollDelta::Lines { x: _x, y } => {
                    if y > 0.0 {
                        self.image_viewer.zoom_in();
                    } else if y < 0.0 {
                        self.image_viewer.zoom_out();
                    }
                }
                ScrollDelta::Pixels { x: _, y: _ } => {
                    panic!("Don't support this yet");
                }
            },
            Message::CursorMoved(point) => {
                self.image_viewer.update_cursor_position(point);
                if self.image_viewer.panning() {
                    self.image_viewer.pan_region();
                }
            }
            Message::StartPanning => {
                self.image_viewer.start_panning();
            }
            Message::StopPanning => {
                self.image_viewer.stop_panning();
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            mouse_area(center(
                widget::image(self.mandelbrot_image.image.clone())
                    .crop(self.image_viewer.image_region())
                    .expand(true)
            ))
            .on_scroll(Message::Zoom)
            .on_move(Message::CursorMoved)
            .on_press(Message::StartPanning)
            .on_release(Message::StopPanning),
            button("Regenerate Image").on_press(Message::Regenerate)
        ]
        .into()
    }
}

fn mandelbrot_image(region: Rectangle<f64>) -> image::Handle {
    let image_size = Size::new(1800, 1200);
    let max_iterations = 1000;
    let x_scale: f64 = region.width as f64 / image_size.width as f64;
    let y_scale: f64 = region.height as f64 / image_size.height as f64;

    let rows = 0..image_size.height;
    let pixels: Vec<u8> = rows
        .into_par_iter()
        .map(|row| {
            let mut pixels: Vec<u8> = Vec::new();
            for column in 0..image_size.width {
                let mut colour = 255;
                let x_position: f64 = region.x as f64 + x_scale * column as f64;
                let y_position: f64 = region.y as f64 - y_scale * row as f64;
                let c = Complex::new(x_position, y_position);

                let mut iteration = 0;
                let mut z: Complex<f64> = Complex::new(0.0, 0.0);
                while z.norm_sqr() <= 4.0 && iteration < max_iterations {
                    z = z * z + c;
                    iteration += 1;
                }

                if iteration == max_iterations {
                    colour = 0;
                }

                pixels.push(colour);
                pixels.push(colour);
                pixels.push(colour);
                pixels.push(255);
            }

            return pixels;
        })
        .flatten()
        .collect();

    image::Handle::from_rgba(image_size.width, image_size.height, pixels)
}
