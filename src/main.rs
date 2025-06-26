mod app;
mod components;
mod meteogram;
mod view;
mod weather;

use app::WeatherApp;
use iced::{Application, Settings};

fn main() -> iced::Result {
    WeatherApp::run(Settings::default())
}