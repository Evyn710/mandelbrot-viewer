use iced::{Element};
use iced::widget;
use iced::{Point, Subscription, Size};
use iced::advanced::image;
use num::complex::Complex;
use std::time::Instant;
use rayon::prelude::*;

fn main() -> iced::Result {
    iced::application(
        MandelbrotViewer::new, 
        MandelbrotViewer::update, 
        MandelbrotViewer::view).subscription(MandelbrotViewer::subscription).run()
}

#[derive(Default)]
struct MandelbrotViewer {
}

#[derive(Debug, Clone)]
enum Message {
}

impl MandelbrotViewer {
    fn new() -> Self {
        MandelbrotViewer{}
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn update(&mut self, message: Message) {
        match message {
        }
    }

    fn view(&self) -> Element<'_, Message>{
        widget::image(mandelbrot_slow()).into()
    }
}

fn mandelbrot_slow() -> image::Handle {
    let image_size  = Size::new(1800, 1200);
    let set_size = Size::new(3, 2);
    let top_left = Point::new(-2, 1);
    let max_iterations = 1000;
    let x_scale: f64 = set_size.width as f64 / image_size.width as f64;
    let y_scale: f64 = set_size.height as f64 / image_size.height as f64;

    let start = Instant::now();
    let rows = 0..image_size.height;
    let pixels: Vec<u8> = rows.into_par_iter().map(|row| {
        let mut pixels: Vec<u8> = Vec::new();
        for column in 0..image_size.width {
            let mut colour = 255;
            let x_position: f64 = top_left.x as f64 + x_scale * column as f64;
            let y_position: f64 = top_left.y as f64 - y_scale * row as f64;
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

            // println!("C: {c}");
            // println!("Iterations: {iteration}");
            // println!("Colour: {colour}");

            pixels.push(colour); 
            pixels.push(colour); 
            pixels.push(colour); 
            pixels.push(255);

        }

        return pixels;
    }).flatten().collect();
    
    let duration = start.elapsed();
    println!("{:?}", duration);
    
    image::Handle::from_rgba(image_size.width, image_size.height, pixels)
}
