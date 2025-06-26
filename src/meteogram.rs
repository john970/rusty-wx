use crate::app::{Message, WeatherElement};
use crate::weather::{self, ForecastPrecip, WeatherDataPoint};
use chrono::{DateTime, Datelike, Local, Timelike};
use iced::{
    widget::canvas::{self, Frame, Text},
    Color, Point, Rectangle, Theme,
};

pub struct Meteogram {
    timeline: Vec<WeatherDataPoint>,
    selected_index: usize,
    selected_element: WeatherElement,
    precip_1hr: Vec<ForecastPrecip>,
    precip_6hr: Vec<ForecastPrecip>,
    unified_temp_min: f64,
    unified_temp_max: f64,
}

impl Meteogram {
    pub fn new(
        timeline: Vec<WeatherDataPoint>,
        selected_index: usize,
        selected_element: WeatherElement,
        precip_1hr: Vec<ForecastPrecip>,
        precip_6hr: Vec<ForecastPrecip>,
        unified_temp_min: f64,
        unified_temp_max: f64,
    ) -> Self {
        Self {
            timeline,
            selected_index,
            selected_element,
            precip_1hr,
            precip_6hr,
            unified_temp_min,
            unified_temp_max,
        }
    }

    fn get_precipitation_probability(&self, target_time: &str) -> Option<f64> {
        if let Ok(target_dt) = DateTime::parse_from_rfc3339(target_time) {
            let target_timestamp = target_dt.timestamp();

            // First try 1hr forecasts for more granular data
            for precip in &self.precip_1hr {
                if let Ok(valid_dt) = DateTime::parse_from_rfc3339(&precip.valid_date) {
                    let valid_timestamp = valid_dt.timestamp();
                    if target_timestamp <= valid_timestamp
                        && target_timestamp > valid_timestamp - 3600
                    {
                        return Some(precip.prob_precip_pct);
                    }
                }
            }

            // Fallback to 6hr forecasts
            for precip in &self.precip_6hr {
                if let Ok(valid_dt) = DateTime::parse_from_rfc3339(&precip.valid_date) {
                    let valid_timestamp = valid_dt.timestamp();
                    if target_timestamp <= valid_timestamp
                        && target_timestamp > valid_timestamp - 21600
                    {
                        return Some(precip.prob_precip_pct);
                    }
                }
            }
        }
        None
    }

    fn get_element_value(&self, point: &WeatherDataPoint) -> Option<f64> {
        match self.selected_element {
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
                            50.0
                        };
                        (humidity, obs.wind_spd_10m_mph.unwrap_or(0.0))
                    }
                    WeatherDataPoint::Forecast(fc) => {
                        let humidity = if let Some(d) = fc.dewpoint_2m_f {
                            weather::dewpoint_to_relative_humidity(d, temp)
                        } else {
                            50.0
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

impl canvas::Program<Message> for Meteogram {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> (canvas::event::Status, Option<Message>) {
        match event {
            canvas::Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left)) => {
                if let Some(cursor_position) = cursor.position_in(bounds) {
                    if !self.timeline.is_empty() {
                        let start_time = if let Ok(dt) = DateTime::parse_from_rfc3339(
                            self.timeline.first().unwrap().valid_date(),
                        ) {
                            dt.timestamp()
                        } else {
                            return (canvas::event::Status::Ignored, None);
                        };

                        let end_time = if let Ok(dt) =
                            DateTime::parse_from_rfc3339(self.timeline.last().unwrap().valid_date())
                        {
                            dt.timestamp()
                        } else {
                            return (canvas::event::Status::Ignored, None);
                        };

                        let time_range = end_time - start_time;
                        let left_margin = 5.0;
                        let right_margin = 10.0;
                        let graph_width = bounds.width - left_margin - right_margin;

                        if cursor_position.x >= left_margin
                            && cursor_position.x <= bounds.width - right_margin
                        {
                            let click_ratio = (cursor_position.x - left_margin) / graph_width;
                            let clicked_time =
                                start_time + (click_ratio * time_range as f32) as i64;

                            let mut closest_index = 0;
                            let mut min_diff = i64::MAX;

                            for (i, point) in self.timeline.iter().enumerate() {
                                if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                                    let diff = (dt.timestamp() - clicked_time).abs();
                                    if diff < min_diff {
                                        min_diff = diff;
                                        closest_index = i;
                                    }
                                }
                            }

                            return (
                                canvas::event::Status::Captured,
                                Some(Message::MeteogramClicked(closest_index)),
                            );
                        }
                    }
                }
                (canvas::event::Status::Ignored, None)
            }
            _ => (canvas::event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb(0.95, 0.95, 0.95),
        );

        // Draw alternating day backgrounds
        if !self.timeline.is_empty() {
            let start_time = if let Ok(dt) =
                DateTime::parse_from_rfc3339(self.timeline.first().unwrap().valid_date())
            {
                dt.timestamp()
            } else {
                return vec![frame.into_geometry()];
            };

            let end_time = if let Ok(dt) =
                DateTime::parse_from_rfc3339(self.timeline.last().unwrap().valid_date())
            {
                dt.timestamp()
            } else {
                return vec![frame.into_geometry()];
            };

            let time_range = end_time - start_time;
            let left_margin = 5.0;
            let right_margin = 10.0;
            let graph_width = bounds.width - left_margin - right_margin;

            let mut current_day: i32 = -1;
            for (_i, point) in self.timeline.iter().enumerate() {
                if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                    let local_time: DateTime<Local> = dt.with_timezone(&Local);
                    let day = local_time.ordinal();

                    if current_day != -1 && day != current_day as u32 {
                        let x_pos = left_margin
                            + ((dt.timestamp() - start_time) as f32 / time_range as f32)
                                * graph_width;
                        frame.fill_rectangle(
                            Point::new(x_pos, 0.0),
                            iced::Size::new(bounds.width, bounds.height),
                            if day % 2 == 0 {
                                Color::from_rgb(0.9, 0.9, 0.9)
                            } else {
                                Color::from_rgb(0.95, 0.95, 0.95)
                            },
                        );
                    }
                    current_day = day as i32;
                }
            }
        }

        // Draw temperature line
        if !self.timeline.is_empty() {
            let start_time = if let Ok(dt) =
                DateTime::parse_from_rfc3339(self.timeline.first().unwrap().valid_date())
            {
                dt.timestamp()
            } else {
                return vec![frame.into_geometry()];
            };

            let end_time = if let Ok(dt) =
                DateTime::parse_from_rfc3339(self.timeline.last().unwrap().valid_date())
            {
                dt.timestamp()
            } else {
                return vec![frame.into_geometry()];
            };

            let time_range = end_time - start_time;
            let left_margin = 5.0;
            let right_margin = 10.0;
            let graph_width = bounds.width - left_margin - right_margin;
            let graph_height = 180.0; // Match the scale's available_height
            let top_margin = 40.0;

            // Use the unified temperature range passed from the app
            let min_temp = self.unified_temp_min;
            let max_temp = self.unified_temp_max;

            if min_temp != f64::MAX && max_temp != f64::MIN {
                // Draw selected element line first (behind temperature) if different from temperature
                if self.selected_element != WeatherElement::Temperature {
                    // For temperature-related elements (WBGT, ApparentTemperature), use the same scale as temperature
                    let use_temp_scale = matches!(
                        self.selected_element,
                        WeatherElement::WBGT | WeatherElement::ApparentTemperature
                    );

                    let (min_element, max_element) = if use_temp_scale {
                        // Use the same temperature scale
                        (min_temp, max_temp)
                    } else {
                        // Calculate separate scale for non-temperature elements
                        let element_values: Vec<Option<f64>> = self
                            .timeline
                            .iter()
                            .map(|point| self.get_element_value(point))
                            .collect();
                        let mut min_elem = f64::MAX;
                        let mut max_elem = f64::MIN;
                        for value in &element_values {
                            if let Some(v) = value {
                                min_elem = min_elem.min(*v);
                                max_elem = max_elem.max(*v);
                            }
                        }

                        if min_elem != f64::MAX && max_elem != f64::MIN {
                            let element_range = max_elem - min_elem;
                            if element_range > 0.0 {
                                min_elem -= element_range * 0.1;
                                max_elem += element_range * 0.1;
                            } else {
                                min_elem -= 1.0;
                                max_elem += 1.0;
                            }
                        }
                        (min_elem, max_elem)
                    };

                    if min_element != f64::MAX && max_element != f64::MIN {
                        let mut element_points = Vec::new();
                        for point in &self.timeline {
                            if let (Ok(dt), Some(value)) = (
                                DateTime::parse_from_rfc3339(point.valid_date()),
                                self.get_element_value(point),
                            ) {
                                let x = left_margin
                                    + ((dt.timestamp() - start_time) as f32 / time_range as f32)
                                        * graph_width;
                                let y = top_margin
                                    + (1.0
                                        - (value - min_element) as f32
                                            / (max_element - min_element) as f32)
                                        * graph_height;
                                element_points.push(Point::new(x, y));
                            }
                        }

                        // Draw element line in grey
                        for i in 1..element_points.len() {
                            frame.stroke(
                                &canvas::Path::line(element_points[i - 1], element_points[i]),
                                canvas::Stroke::default()
                                    .with_width(2.0)
                                    .with_color(Color::from_rgb(0.5, 0.5, 0.5)),
                            );
                        }
                    }
                }

                // Draw temperature line with color-coded segments - create smooth curve with many small segments
                let segments_per_section = 10; // Number of interpolated segments between each data point

                for i in 0..self.timeline.len() - 1 {
                    if let (Some(temp1), Some(temp2)) = (
                        self.timeline[i].temperature(),
                        self.timeline[i + 1].temperature(),
                    ) {
                        if let (Ok(dt1), Ok(dt2)) = (
                            DateTime::parse_from_rfc3339(self.timeline[i].valid_date()),
                            DateTime::parse_from_rfc3339(self.timeline[i + 1].valid_date()),
                        ) {
                            let time_offset1 = dt1.timestamp() - start_time;
                            let time_offset2 = dt2.timestamp() - start_time;
                            let x1 = left_margin
                                + (time_offset1 as f32 / time_range as f32) * graph_width;
                            let x2 = left_margin
                                + (time_offset2 as f32 / time_range as f32) * graph_width;
                            let y1 = top_margin
                                + (1.0 - ((temp1 - min_temp) / (max_temp - min_temp)) as f32)
                                    * graph_height;
                            let y2 = top_margin
                                + (1.0 - ((temp2 - min_temp) / (max_temp - min_temp)) as f32)
                                    * graph_height;

                            // Create many small segments for smooth gradient effect
                            for seg in 0..segments_per_section {
                                let t1 = seg as f32 / segments_per_section as f32;
                                let t2 = (seg + 1) as f32 / segments_per_section as f32;

                                // Interpolate positions
                                let x_start = x1 + t1 * (x2 - x1);
                                let x_end = x1 + t2 * (x2 - x1);
                                let y_start = y1 + t1 * (y2 - y1);
                                let y_end = y1 + t2 * (y2 - y1);

                                // Interpolate temperature for this micro-segment
                                let temp_mid = temp1 + ((t1 + t2) / 2.0) as f64 * (temp2 - temp1);

                                // Use actual temperature for realistic color scale
                                let color = match temp_mid {
                                    t if t < 32.0 => Color::from_rgb(0.0, 0.0, 1.0), // Blue - freezing
                                    t if t < 50.0 => Color::from_rgb(0.0, 0.5, 1.0), // Light blue - cold
                                    t if t < 65.0 => Color::from_rgb(0.0, 0.8, 0.8), // Cyan - cool
                                    t if t < 75.0 => Color::from_rgb(0.0, 1.0, 0.0), // Green - comfortable
                                    t if t < 85.0 => Color::from_rgb(1.0, 1.0, 0.0), // Yellow - warm
                                    t if t < 95.0 => Color::from_rgb(1.0, 0.5, 0.0), // Orange - hot
                                    _ => Color::from_rgb(1.0, 0.0, 0.0), // Red - very hot
                                };

                                // Draw micro-segment
                                frame.stroke(
                                    &canvas::Path::line(
                                        Point::new(x_start, y_start),
                                        Point::new(x_end, y_end),
                                    ),
                                    canvas::Stroke::default().with_color(color).with_width(4.0),
                                );
                            }
                        }
                    }
                }

                // Draw day of week labels centered over each day
                let mut labeled_days = std::collections::HashSet::new();
                for point in &self.timeline {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                        let local_dt: DateTime<Local> = dt.with_timezone(&Local);
                        let day = local_dt.ordinal();

                        if !labeled_days.contains(&day) {
                            // Find the center of this day by looking for noon (12:00)
                            let noon_time = local_dt
                                .date_naive()
                                .and_hms_opt(12, 0, 0)
                                .unwrap()
                                .and_local_timezone(Local)
                                .single()
                                .unwrap();

                            let noon_timestamp = noon_time.timestamp();
                            let time_offset = noon_timestamp - start_time;
                            let x = left_margin
                                + (time_offset as f32 / time_range as f32) * graph_width;

                            // Only draw if the center is within our time range
                            if time_offset >= 0 && time_offset <= time_range {
                                // Day of week label with date (abbreviated)
                                let weekday_name = match local_dt.weekday() {
                                    chrono::Weekday::Mon => "Mon",
                                    chrono::Weekday::Tue => "Tue",
                                    chrono::Weekday::Wed => "Wed",
                                    chrono::Weekday::Thu => "Thu",
                                    chrono::Weekday::Fri => "Fri",
                                    chrono::Weekday::Sat => "Sat",
                                    chrono::Weekday::Sun => "Sun",
                                };
                                let day_label = format!(
                                    "{} {}/{}",
                                    weekday_name,
                                    local_dt.month(),
                                    local_dt.day()
                                );

                                frame.fill_text(Text {
                                    content: day_label.to_string(),
                                    position: Point::new(x, 2.0),
                                    size: 14.0.into(),
                                    color: Color::from_rgb(0.2, 0.2, 0.2),
                                    font: iced::Font::default(),
                                    horizontal_alignment: iced::alignment::Horizontal::Center,
                                    vertical_alignment: iced::alignment::Vertical::Top,
                                    line_height: iced::widget::text::LineHeight::default(),
                                    shaping: iced::widget::text::Shaping::default(),
                                });

                                labeled_days.insert(day);
                            }
                        }
                    }
                }

                // Draw time labels approximately every 4 hours
                let mut last_labeled_hour: Option<u32> = None;

                for point in &self.timeline {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                        let local_dt: DateTime<Local> = dt.with_timezone(&Local);
                        let hour = local_dt.hour();

                        // Show labels roughly every 4 hours, but adapt to available data
                        let should_show_label = match last_labeled_hour {
                            None => true, // Always show first label
                            Some(last_hour) => {
                                // Calculate hour difference, handling day wrap-around
                                let hour_diff = if hour >= last_hour {
                                    hour - last_hour
                                } else {
                                    (24 - last_hour) + hour
                                };
                                hour_diff >= 4 // Show if at least 4 hours have passed
                            }
                        };

                        if should_show_label {
                            let time_offset = dt.timestamp() - start_time;
                            let x = left_margin
                                + (time_offset as f32 / time_range as f32) * graph_width;

                            // Time label
                            let time_label = if hour == 0 {
                                "12A".to_string()
                            } else if hour == 12 {
                                "12P".to_string()
                            } else if hour < 12 {
                                format!("{}A", hour)
                            } else {
                                format!("{}P", hour - 12)
                            };

                            frame.fill_text(Text {
                                content: time_label,
                                position: Point::new(x, 18.0),
                                size: 12.0.into(),
                                color: Color::BLACK,
                                font: iced::Font::default(),
                                horizontal_alignment: iced::alignment::Horizontal::Center,
                                vertical_alignment: iced::alignment::Vertical::Top,
                                line_height: iced::widget::text::LineHeight::default(),
                                shaping: iced::widget::text::Shaping::default(),
                            });

                            last_labeled_hour = Some(hour);
                        }
                    }
                }

                // Draw vertical line for "now" (current time)
                let now = Local::now();
                let mut closest_now_index = 0;
                let mut min_diff = i64::MAX;

                for (i, point) in self.timeline.iter().enumerate() {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(point.valid_date()) {
                        let local_dt: DateTime<Local> = dt.with_timezone(&Local);
                        let diff = (now.timestamp() - local_dt.timestamp()).abs();
                        if diff < min_diff {
                            min_diff = diff;
                            closest_now_index = i;
                        }
                    }
                }

                let now_x = if let Some(now_point) = self.timeline.get(closest_now_index) {
                    if let Ok(dt) = DateTime::parse_from_rfc3339(now_point.valid_date()) {
                        let time_offset = dt.timestamp() - start_time;
                        left_margin + (time_offset as f32 / time_range as f32) * graph_width
                    } else {
                        left_margin
                    }
                } else {
                    left_margin
                };

                frame.stroke(
                    &canvas::Path::line(
                        Point::new(now_x, top_margin),
                        Point::new(now_x, bounds.height - 20.0),
                    ),
                    canvas::Stroke::default()
                        .with_color(Color::from_rgb(0.0, 0.0, 1.0)) // Blue for "now"
                        .with_width(2.0),
                );

                // Draw selected point indicator
                if let Some(selected_point) = self.timeline.get(self.selected_index) {
                    if let (Ok(dt), Some(temp)) = (
                        DateTime::parse_from_rfc3339(selected_point.valid_date()),
                        selected_point.temperature(),
                    ) {
                        let x = left_margin
                            + ((dt.timestamp() - start_time) as f32 / time_range as f32)
                                * graph_width;
                        let y = top_margin
                            + (1.0 - (temp - min_temp) as f32 / (max_temp - min_temp) as f32)
                                * graph_height;
                        frame.fill(
                            &canvas::Path::circle(Point::new(x, y), 4.0),
                            Color::from_rgb(0.0, 0.0, 1.0),
                        );
                    }
                }
            }
        }

        vec![frame.into_geometry()]
    }
}
