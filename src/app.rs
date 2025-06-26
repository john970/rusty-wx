use crate::weather::{self, ApiWeatherData, WeatherDataPoint};
use chrono::{DateTime, Local, Utc};
use iced::{
    widget::{column, container, scrollable, text, Space},
    Application, Color, Command, Element, Length, Theme,
};

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherElement {
    Temperature,
    ApparentTemperature,
    WBGT,
    WindSpeed,
    Pressure,
    Humidity,
    Dewpoint,
    CloudCover,
    Visibility,
    SolarFlux,
    ThunderstormProbability,
    CAPE,
    PrecipitationProbability,
}

#[derive(Debug, Clone)]
pub enum Message {
    WeatherFetched(Result<ApiWeatherData, String>),
    FetchWeather(String, f64, f64),
    LatInputChanged(String),
    LonInputChanged(String),
    FetchCustomLocation,
    PreviousHour,
    NextHour,
    GoToNow,
    MeteogramClicked(usize), // Index of the clicked time point
    SelectWeatherElement(WeatherElement),
}

pub struct WeatherApp {
    pub current_city: String,
    pub weather_data: Option<ApiWeatherData>,
    pub combined_timeline: Vec<WeatherDataPoint>,
    pub loading: bool,
    pub error: Option<String>,
    pub lat_input: String,
    pub lon_input: String,
    pub timeline_index: usize,
    pub last_updated: Option<DateTime<Local>>,
    pub meteogram_scroll_id: scrollable::Id,
    pub should_scroll_to_now: bool,
    pub selected_weather_element: WeatherElement,
}

impl Default for WeatherApp {
    fn default() -> Self {
        Self {
            current_city: String::new(),
            weather_data: None,
            combined_timeline: Vec::new(),
            loading: false,
            error: None,
            lat_input: String::new(),
            lon_input: String::new(),
            timeline_index: 0,
            last_updated: None,
            meteogram_scroll_id: scrollable::Id::unique(),
            should_scroll_to_now: false,
            selected_weather_element: WeatherElement::PrecipitationProbability, // Default to Precipitation Probability
        }
    }
}

impl WeatherApp {
    pub fn get_precipitation_probability(&self, target_time: &str) -> Option<f64> {
        if let Some(weather) = &self.weather_data {
            if let Ok(target_dt) = DateTime::parse_from_rfc3339(target_time) {
                let target_timestamp = target_dt.timestamp();

                // First try 1hr forecasts for more granular data
                for precip in &weather.forecasts_precip_1hr {
                    if let Ok(valid_dt) = DateTime::parse_from_rfc3339(&precip.valid_date) {
                        let valid_timestamp = valid_dt.timestamp();
                        // Check if target time is within the forecast period (up to 1 hour before valid time)
                        if target_timestamp <= valid_timestamp
                            && target_timestamp > valid_timestamp - 3600
                        {
                            return Some(precip.prob_precip_pct);
                        }
                    }
                }

                // Fallback to 6hr forecasts
                for precip in &weather.forecasts_precip_6hr {
                    if let Ok(valid_dt) = DateTime::parse_from_rfc3339(&precip.valid_date) {
                        let valid_timestamp = valid_dt.timestamp();
                        // Check if target time is within the forecast period (up to 6 hours before valid time)
                        if target_timestamp <= valid_timestamp
                            && target_timestamp > valid_timestamp - 21600
                        {
                            return Some(precip.prob_precip_pct);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn build_timeline(&mut self) {
        if let Some(weather) = &self.weather_data {
            self.combined_timeline.clear();

            // Add observations in reverse order (oldest to newest)
            for obs in weather.observations_instant.iter().rev() {
                self.combined_timeline
                    .push(WeatherDataPoint::Observation(obs.clone()));
            }

            // Add forecasts (already in chronological order)
            for fc in &weather.forecasts_instant {
                self.combined_timeline
                    .push(WeatherDataPoint::Forecast(fc.clone()));
            }

            // Find the index closest to current time
            let now = Utc::now();
            let mut closest_index = 0;
            let mut smallest_diff = i64::MAX;

            for (i, point) in self.combined_timeline.iter().enumerate() {
                if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                    let diff = (dt.timestamp() - now.timestamp()).abs();
                    if diff < smallest_diff {
                        smallest_diff = diff;
                        closest_index = i;
                    }
                }
            }

            self.timeline_index = closest_index;
        }
    }
    pub fn create_temp_scale(&self, timeline: &[WeatherDataPoint]) -> Element<Message> {
        if timeline.is_empty() {
            return container(text(""))
                .width(Length::Fixed(40.0))
                .height(Length::Fixed(230.0))
                .into();
        }

        // Calculate unified temperature range (includes temperature, WBGT, and apparent temperature)
        let (min_temp, max_temp) = self.get_unified_temp_range(timeline);

        // Create temperature labels
        let mut temp_labels = column![]
            .spacing(0)
            .height(Length::Fixed(230.0))
            .width(Length::Fixed(40.0));

        let label_count = 5;
        let available_height = 180.0; // Account for margins (250 - 50 for margins)
        let top_margin = 40.0;

        for i in 0..=label_count {
            let temp = max_temp - (i as f64 / label_count as f64) * (max_temp - min_temp);
            let label_height = available_height / label_count as f32;
            let spacing_height = if i == 0 {
                top_margin
            } else {
                label_height - 16.0
            };

            if i > 0 {
                temp_labels = temp_labels.push(Space::with_height(Length::Fixed(spacing_height)));
            } else {
                temp_labels = temp_labels.push(Space::with_height(Length::Fixed(spacing_height)));
            }

            temp_labels = temp_labels.push(
                text(format!("{:.0}°", temp))
                    .size(12)
                    .horizontal_alignment(iced::alignment::Horizontal::Right),
            );
        }

        container(temp_labels)
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(230.0))
            .padding([0, 5, 0, 0])
            .into()
    }

    pub fn create_element_scale(
        &self,
        timeline: &[WeatherDataPoint],
        element: &WeatherElement,
    ) -> Element<Message> {
        if timeline.is_empty()
            || *element == WeatherElement::Temperature
            || *element == WeatherElement::WBGT
        {
            return container(text(""))
                .width(Length::Fixed(50.0))
                .height(Length::Fixed(230.0))
                .into();
        }

        // Get the range for the selected weather element
        let element_values: Vec<Option<f64>> = timeline
            .iter()
            .map(|point| self.get_element_value_for_scale(point, element))
            .collect();

        let mut min_element = f64::MAX;
        let mut max_element = f64::MIN;
        for value in &element_values {
            if let Some(v) = value {
                min_element = min_element.min(*v);
                max_element = max_element.max(*v);
            }
        }

        // Add padding to the element range
        let element_range = max_element - min_element;
        if element_range > 0.0 {
            min_element -= element_range * 0.1;
            max_element += element_range * 0.1;
        } else {
            // Handle case where all values are the same
            min_element -= 1.0;
            max_element += 1.0;
        }

        // Create element labels
        let mut element_labels = column![]
            .spacing(0)
            .height(Length::Fixed(230.0))
            .width(Length::Fixed(50.0));

        let label_count = 5;
        let available_height = 180.0; // Account for margins (250 - 50 for margins)
        let top_margin = 40.0;

        for i in 0..=label_count {
            let value = max_element - (i as f64 / label_count as f64) * (max_element - min_element);
            let label_height = available_height / label_count as f32;
            let spacing_height = if i == 0 {
                top_margin
            } else {
                label_height - 16.0
            };

            if i > 0 {
                element_labels =
                    element_labels.push(Space::with_height(Length::Fixed(spacing_height)));
            } else {
                element_labels =
                    element_labels.push(Space::with_height(Length::Fixed(spacing_height)));
            }

            // Format the value based on element type
            let label_text = match element {
                WeatherElement::WindSpeed => format!("{:.0}", value),
                WeatherElement::Pressure => format!("{:.0}", value),
                WeatherElement::CloudCover => format!("{:.0}%", value),
                WeatherElement::Visibility => format!("{:.1}", value),
                WeatherElement::SolarFlux => format!("{:.0}", value),
                WeatherElement::ThunderstormProbability => format!("{:.0}%", value),
                WeatherElement::Humidity => format!("{:.0}%", value),
                WeatherElement::Dewpoint => format!("{:.0}°", value),
                WeatherElement::CAPE => format!("{:.0}", value),
                WeatherElement::ApparentTemperature => format!("{:.0}°", value),
                WeatherElement::PrecipitationProbability => format!("{:.0}%", value),
                _ => format!("{:.1}", value),
            };

            element_labels = element_labels.push(
                text(label_text)
                    .size(12)
                    .style(Color::from_rgb(0.5, 0.5, 0.5)) // Grey to match the line
                    .horizontal_alignment(iced::alignment::Horizontal::Left),
            );
        }

        container(element_labels)
            .width(Length::Fixed(50.0))
            .height(Length::Fixed(230.0))
            .padding([0, 0, 0, 5])
            .into()
    }

    pub fn get_unified_temp_range(&self, timeline: &[WeatherDataPoint]) -> (f64, f64) {
        let mut min_temp = f64::MAX;
        let mut max_temp = f64::MIN;

        for point in timeline {
            // Regular temperature
            if let Some(temp) = point.temperature() {
                min_temp = min_temp.min(temp);
                max_temp = max_temp.max(temp);
            }

            // WBGT temperature
            if let Some(wbgt) = self.get_element_value_for_scale(point, &WeatherElement::WBGT) {
                min_temp = min_temp.min(wbgt);
                max_temp = max_temp.max(wbgt);
            }

            // Apparent temperature
            if let Some(apparent) =
                self.get_element_value_for_scale(point, &WeatherElement::ApparentTemperature)
            {
                min_temp = min_temp.min(apparent);
                max_temp = max_temp.max(apparent);
            }
        }

        let temp_range = max_temp - min_temp;
        min_temp -= temp_range * 0.1;
        max_temp += temp_range * 0.1;

        (min_temp, max_temp)
    }

    pub fn get_element_value_for_scale(
        &self,
        point: &WeatherDataPoint,
        element: &WeatherElement,
    ) -> Option<f64> {
        match element {
            WeatherElement::Temperature => point.temperature(),
            WeatherElement::WBGT => match point {
                WeatherDataPoint::Forecast(fc) => fc.wbg_temp_2m_f,
                WeatherDataPoint::Observation(_) => None,
            },
            WeatherElement::WindSpeed => match point {
                WeatherDataPoint::Observation(obs) => obs.wind_spd_10m_mph,
                WeatherDataPoint::Forecast(fc) => fc.wind_spd_10m_mph,
            },
            WeatherElement::Pressure => match point {
                WeatherDataPoint::Observation(obs) => obs.pressure_h_pa,
                WeatherDataPoint::Forecast(_) => None,
            },
            WeatherElement::Humidity => match point {
                WeatherDataPoint::Observation(obs) => {
                    if let Some(humidity) = obs.specific_humidity_2m_dg_kg {
                        Some(humidity / 10.0)
                    } else if let (Some(dewpoint), Some(temp)) =
                        (obs.dewpoint_2m_f, obs.temperature_2m_f)
                    {
                        Some(weather::dewpoint_to_relative_humidity(dewpoint, temp))
                    } else {
                        None
                    }
                }
                WeatherDataPoint::Forecast(fc) => {
                    if let (Some(dewpoint), Some(temp)) = (fc.dewpoint_2m_f, fc.temperature_2m_f) {
                        Some(weather::dewpoint_to_relative_humidity(dewpoint, temp))
                    } else {
                        None
                    }
                }
            },
            WeatherElement::Dewpoint => match point {
                WeatherDataPoint::Observation(obs) => {
                    if let Some(dewpoint) = obs.dewpoint_2m_f {
                        Some(dewpoint)
                    } else if let (Some(humidity), Some(temp)) =
                        (obs.specific_humidity_2m_dg_kg, obs.temperature_2m_f)
                    {
                        Some(weather::relative_humidity_to_dewpoint(
                            humidity / 10.0,
                            temp,
                        ))
                    } else {
                        None
                    }
                }
                WeatherDataPoint::Forecast(fc) => fc.dewpoint_2m_f,
            },
            WeatherElement::CloudCover => match point {
                WeatherDataPoint::Observation(obs) => obs.cloud_cover_pct,
                WeatherDataPoint::Forecast(fc) => fc.cloud_cover_pct,
            },
            WeatherElement::Visibility => match point {
                WeatherDataPoint::Observation(obs) => obs.visibility_m.map(|v| v / 1609.34),
                WeatherDataPoint::Forecast(fc) => fc.visibility_m.map(|v| v / 1609.34),
            },
            WeatherElement::SolarFlux => match point {
                WeatherDataPoint::Observation(obs) => obs.solar_flux_w_m2,
                WeatherDataPoint::Forecast(fc) => fc.solar_flux_w_m2,
            },
            WeatherElement::ThunderstormProbability => match point {
                WeatherDataPoint::Observation(_) => None,
                WeatherDataPoint::Forecast(fc) => fc.prob_thunderstorm_pct,
            },
            WeatherElement::CAPE => match point {
                WeatherDataPoint::Observation(_) => None,
                WeatherDataPoint::Forecast(fc) => fc.cape_surface_j_kg,
            },
            WeatherElement::ApparentTemperature => {
                let temp = point.temperature()?;
                let (humidity, wind) = match point {
                    WeatherDataPoint::Observation(obs) => {
                        let humidity = if let Some(h) = obs.specific_humidity_2m_dg_kg {
                            h / 10.0
                        } else if let Some(d) = obs.dewpoint_2m_f {
                            weather::dewpoint_to_relative_humidity(d, temp)
                        } else {
                            50.0 // Default if no humidity data
                        };
                        (humidity, obs.wind_spd_10m_mph.unwrap_or(0.0))
                    }
                    WeatherDataPoint::Forecast(fc) => {
                        let humidity = if let Some(d) = fc.dewpoint_2m_f {
                            weather::dewpoint_to_relative_humidity(d, temp)
                        } else {
                            50.0 // Default if no humidity data
                        };
                        (humidity, fc.wind_spd_10m_mph.unwrap_or(0.0))
                    }
                };
                Some(weather::calculate_apparent_temperature(
                    temp, humidity, wind,
                ))
            }
            WeatherElement::PrecipitationProbability => {
                self.get_precipitation_probability(point.valid_date())
            }
        }
    }
}

use crate::view;

impl Application for WeatherApp {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (WeatherApp, Command<Message>) {
        let mut app = WeatherApp::default();
        app.current_city = "Denver".to_string();
        let command = Command::perform(
            weather::fetch_weather(39.7392, -104.9903),
            Message::WeatherFetched,
        );
        (app, command)
    }

    fn title(&self) -> String {
        String::from("Weather App - Iced")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FetchWeather(city, lat, lon) => {
                self.current_city = city;
                self.loading = true;
                self.error = None;

                Command::perform(weather::fetch_weather(lat, lon), Message::WeatherFetched)
            }
            Message::WeatherFetched(result) => {
                self.loading = false;
                match result {
                    Ok(data) => {
                        self.weather_data = Some(data);
                        self.error = None;
                        self.last_updated = Some(Local::now());
                        self.build_timeline();
                        self.should_scroll_to_now = true;
                    }
                    Err(error) => {
                        self.error = Some(error);
                        self.weather_data = None;
                        self.combined_timeline.clear();
                        self.last_updated = None;
                    }
                }

                // Return scroll command if we need to scroll to now
                if self.should_scroll_to_now && !self.combined_timeline.is_empty() {
                    self.should_scroll_to_now = false;

                    // Calculate scroll position
                    let now = Local::now();
                    let start_time = if let Ok(dt) = DateTime::parse_from_rfc3339(
                        self.combined_timeline.first().unwrap().valid_date(),
                    ) {
                        dt.timestamp()
                    } else {
                        return Command::none();
                    };
                    let end_time = if let Ok(dt) = DateTime::parse_from_rfc3339(
                        self.combined_timeline.last().unwrap().valid_date(),
                    ) {
                        dt.timestamp()
                    } else {
                        return Command::none();
                    };
                    let time_range = end_time - start_time;

                    let canvas_width = 4000.0;
                    let now_time_offset = now.timestamp() - start_time;
                    let now_position = (now_time_offset as f32 / time_range as f32) * canvas_width;
                    let viewport_width = 800.0;
                    let scroll_offset = (now_position - viewport_width / 2.0).max(0.0);

                    scrollable::scroll_to(
                        self.meteogram_scroll_id.clone(),
                        scrollable::AbsoluteOffset {
                            x: scroll_offset,
                            y: 0.0,
                        },
                    )
                } else {
                    Command::none()
                }
            }
            Message::LatInputChanged(value) => {
                self.lat_input = value;
                Command::none()
            }
            Message::LonInputChanged(value) => {
                self.lon_input = value;
                Command::none()
            }
            Message::FetchCustomLocation => {
                match (self.lat_input.parse::<f64>(), self.lon_input.parse::<f64>()) {
                    (Ok(lat), Ok(lon)) => {
                        if lat >= -90.0 && lat <= 90.0 && lon >= -180.0 && lon <= 180.0 {
                            self.current_city = format!("{:.2}, {:.2}", lat, lon);
                            self.loading = true;
                            self.error = None;
                            Command::perform(
                                weather::fetch_weather(lat, lon),
                                Message::WeatherFetched,
                            )
                        } else {
                            self.error = Some("Invalid coordinates: Latitude must be between -90 and 90, Longitude between -180 and 180".to_string());
                            Command::none()
                        }
                    }
                    _ => {
                        self.error = Some(
                            "Invalid input: Please enter valid numeric coordinates".to_string(),
                        );
                        Command::none()
                    }
                }
            }
            Message::PreviousHour => {
                if self.timeline_index > 0 {
                    self.timeline_index -= 1;
                }
                Command::none()
            }
            Message::NextHour => {
                if self.timeline_index < self.combined_timeline.len().saturating_sub(1) {
                    self.timeline_index += 1;
                }
                Command::none()
            }
            Message::GoToNow => {
                // Find the index closest to current time
                let now = Utc::now();
                let mut closest_index = 0;
                let mut smallest_diff = i64::MAX;

                for (i, point) in self.combined_timeline.iter().enumerate() {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                        let diff = (dt.timestamp() - now.timestamp()).abs();
                        if diff < smallest_diff {
                            smallest_diff = diff;
                            closest_index = i;
                        }
                    }
                }

                self.timeline_index = closest_index;

                // Also scroll meteogram to "now" position
                if !self.combined_timeline.is_empty() {
                    let start_time = if let Ok(dt) = DateTime::parse_from_rfc3339(
                        self.combined_timeline.first().unwrap().valid_date(),
                    ) {
                        dt.timestamp()
                    } else {
                        return Command::none();
                    };
                    let end_time = if let Ok(dt) = DateTime::parse_from_rfc3339(
                        self.combined_timeline.last().unwrap().valid_date(),
                    ) {
                        dt.timestamp()
                    } else {
                        return Command::none();
                    };
                    let time_range = end_time - start_time;

                    let canvas_width = 4000.0;
                    let now_time_offset = now.timestamp() - start_time;
                    let now_position = (now_time_offset as f32 / time_range as f32) * canvas_width;
                    let viewport_width = 800.0;
                    let scroll_offset = (now_position - viewport_width / 2.0).max(0.0);

                    scrollable::scroll_to(
                        self.meteogram_scroll_id.clone(),
                        scrollable::AbsoluteOffset {
                            x: scroll_offset,
                            y: 0.0,
                        },
                    )
                } else {
                    Command::none()
                }
            }
            Message::MeteogramClicked(index) => {
                if index < self.combined_timeline.len() {
                    self.timeline_index = index;
                }
                Command::none()
            }
            Message::SelectWeatherElement(element) => {
                self.selected_weather_element = element;
                Command::none()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn view(&self) -> Element<Message> {
        view::view(self)
    }
}
