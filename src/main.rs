use iced::advanced::image;
use iced::mouse::ScrollDelta;
use iced::widget::{self, button};
use iced::widget::{center, column, mouse_area};
use iced::Element;
use iced::{Point, Rectangle, Size, Subscription};
use num::complex::Complex;
use rayon::prelude::*;
use std::time::Instant;

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

#[derive(Debug, Clone)]
enum Message {
    Regenerate,
    Zoom(ScrollDelta),
}

fn zoom_in(region: &mut Rectangle<u32>) {
    let width_to_remove = region.width as f32 * 0.10;
    let height_to_remove = region.height as f32 * 0.10;
    region.x += (width_to_remove / 2.0) as u32;
    region.y += (height_to_remove / 2.0) as u32;
    region.width -= width_to_remove as u32;
    region.height -= height_to_remove as u32;
}

fn zoom_out(region: &mut Rectangle<u32>, max_size: &Size<u32>) {
    let width_to_add: u32 = (region.width as f32 * 0.10) as u32;
    let height_to_add: u32 = (region.height as f32 * 0.10) as u32;

    let mut x_bounded = false;
    let mut y_bounded = false;
    let mut width_bounded = false;
    let mut height_bounded = false;

    if region.x > width_to_add / 2 {
        region.x -= width_to_add / 2;
    } else {
        x_bounded = true;
    }

    if region.y > height_to_add / 2 {
        region.y -= height_to_add / 2;
    } else {
        y_bounded = true;
    }

    if region.width + width_to_add < max_size.width {
        region.width += width_to_add;
    } else {
        width_bounded = true;
    }

    if region.height + height_to_add < max_size.height {
        region.height += height_to_add;
    } else {
        height_bounded = true;
    }

    if x_bounded && y_bounded && width_bounded && height_bounded {
        region.x = 0;
        region.y = 0;
        region.width = max_size.width;
        region.height = max_size.height;
    }
}

fn translate_image_space_to_set_space(
    image_space: &Rectangle<u32>,
    max_size: &Size<u32>,
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

struct MandelbrotViewer {
    mandelbrot_image: MandelbrotImage,
    image_region: Rectangle<u32>,
    max_size: Size<u32>,
}

impl MandelbrotViewer {
    fn new() -> Self {
        Self {
            mandelbrot_image: MandelbrotImage::new(),
            image_region: Rectangle {
                x: 0,
                y: 0,
                width: 1800,
                height: 1200,
            },
            max_size: Size {
                width: 1800,
                height: 1200,
            },
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Regenerate => {
                let new_set_space = translate_image_space_to_set_space(
                    &self.image_region,
                    &self.max_size,
                    &self.mandelbrot_image.current_region,
                );
                self.mandelbrot_image.image = mandelbrot_image(new_set_space);
                self.mandelbrot_image.current_region = new_set_space;
                self.image_region = Rectangle {
                    x: 0,
                    y: 0,
                    width: self.max_size.width,
                    height: self.max_size.height,
                }
            }
            Message::Zoom(scroll_delta) => match scroll_delta {
                ScrollDelta::Lines { x: _x, y } => {
                    if y > 0.0 {
                        zoom_in(&mut self.image_region);
                    } else if y < 0.0 {
                        zoom_out(&mut self.image_region, &self.max_size);
                    }
                }
                ScrollDelta::Pixels { x: _, y: _ } => {
                    panic!("Don't support this yet");
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            mouse_area(center(
                widget::image(self.mandelbrot_image.image.clone())
                    .crop(self.image_region)
                    .expand(true)
            ))
            .on_scroll(Message::Zoom),
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

    let start = Instant::now();
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

    let duration = start.elapsed();
    println!("{:?}", duration);

    image::Handle::from_rgba(image_size.width, image_size.height, pixels)
}
