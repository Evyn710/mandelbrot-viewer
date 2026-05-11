use iced::{Element};
use iced::widget;
use iced::widget::{button};
use iced::advanced::image;

fn main() -> iced::Result {
    iced::run(update, view)
}

#[derive(Default)]
struct State {
}

#[derive(Debug, Clone)]
enum Message {
}

fn update(state: &mut State, message: Message) {
    match message {
    }
}

fn view(state: &State) -> Element<'_, Message>{
    widget::image(mandelbrot_slow()).into()
}

fn mandelbrot_slow() -> image::Handle {
    let mut pixels: Vec<u8> = Vec::new();
    for row in 0..255 {
        for column in 0..255 {
            pixels.push(column as u8); 
            pixels.push(column as u8); 
            pixels.push(column as u8); 
            pixels.push(255); 
        }
    }
    image::Handle::from_rgba(255, 255, pixels)
}
