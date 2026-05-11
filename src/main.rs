use iced::{Element};
use iced::widget;
use iced::Subscription;
use iced::advanced::image;

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
    let mut pixels: Vec<u8> = Vec::new();
    for row in 0..512 {
        for column in 0..512 {
            let brightness_value: u8 = (row as f32 * column as f32 / (512.0 * 512.0) * 255.0) as u8;
            pixels.push(brightness_value); 
            pixels.push(brightness_value); 
            pixels.push(brightness_value); 
            pixels.push(255);
        }
    }
    image::Handle::from_rgba(512, 512, pixels)
}
