use crate::app::{Message, WeatherApp, WeatherElement};
use crate::weather::{self, WeatherDataPoint};
use chrono::{DateTime, Local, Utc};
use iced::{
    alignment, theme,
    widget::{button, column, container, row, text, Space},
    Alignment, Color, Element, Length,
};

pub fn create_status_display(app: &WeatherApp) -> Element<Message> {
    let status_text = if app.loading {
        text("Loading weather data...").size(18)
    } else if let Some(error) = &app.error {
        text(format!("Error: {}", error))
            .size(16)
            .style(Color::from_rgb(0.8, 0.2, 0.2))
    } else {
        text("No data available").size(16)
    };

    container(status_text)
        .padding(20)
        .center_x()
        .center_y()
        .width(Length::Fill)
        .height(Length::Fixed(150.0))
        .into()
}

pub fn create_temperature_card<'a>(
    app: &'a WeatherApp,
    data_point: &'a WeatherDataPoint,
) -> Element<'a, Message> {
    let city_text = text(&app.current_city).size(24);
    let temp_text = match data_point.temperature() {
        Some(temp) => text(format!("{:.0}°", temp)).size(42),
        None => text("--°").size(42),
    };

    // Data type badge
    let data_badge = match data_point {
        WeatherDataPoint::Observation(_) => container(text("OBSERVATION").size(12))
            .padding([4, 12])
            .style(theme::Container::Box),
        WeatherDataPoint::Forecast(_) => container(text("FORECAST").size(12))
            .padding([4, 12])
            .style(theme::Container::Box),
    };

    // Time display
    let time_info = if let Ok(dt) = DateTime::parse_from_rfc3339(data_point.valid_date()) {
        let utc_time: DateTime<Utc> = dt.with_timezone(&Utc);
        let local_time: DateTime<Local> = utc_time.with_timezone(&Local);

        column![
            row![
                text(format!("{}", local_time.format("%-I:%M %p"))).size(18),
                Space::with_width(Length::Fixed(6.0)),
                text(format!("{}", local_time.format("%Z"))).size(14),
            ]
            .align_items(Alignment::Center),
            text(format!("{}", local_time.format("%a %m/%d"))).size(16),
        ]
        .spacing(2)
    } else {
        column![text("Invalid time").size(14)]
    };

    // Navigation buttons
    let prev_button = if app.timeline_index > 0 {
        button(
            text("← Earlier")
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::PreviousHour)
        .style(theme::Button::Secondary)
        .width(Length::Fixed(90.0))
    } else {
        button(
            text("← Earlier")
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .style(theme::Button::Secondary)
        .width(Length::Fixed(90.0))
    };

    let next_button = if app.timeline_index < app.combined_timeline.len() - 1 {
        button(
            text("Later →")
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .on_press(Message::NextHour)
        .style(theme::Button::Secondary)
        .width(Length::Fixed(90.0))
    } else {
        button(
            text("Later →")
                .size(14)
                .horizontal_alignment(alignment::Horizontal::Center),
        )
        .style(theme::Button::Secondary)
        .width(Length::Fixed(90.0))
    };

    let now_button = button(text("Now").size(14).horizontal_alignment(alignment::Horizontal::Center))
        .on_press(Message::GoToNow)
        .style(theme::Button::Secondary)
        .width(Length::Fixed(70.0));

    let time_nav = row![prev_button, now_button, next_button].spacing(8);

    // API response time
    let api_time = if let Some(updated) = &app.last_updated {
        text(format!("Updated: {}", updated.format("%I:%M:%S %p")))
            .size(12)
            .style(Color::from_rgb(0.5, 0.5, 0.5))
    } else {
        text("").size(12)
    };

    // Forecast cycle time
    let cycle_time = if let Some(weather) = &app.weather_data {
        if let Some(first_forecast) = weather.forecasts_instant.first() {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&first_forecast.cycle_date) {
                let local_dt: DateTime<Local> = dt.with_timezone(&Local);
                text(format!(
                    "Forecast Cycle: {}",
                    local_dt.format("%I:%M %p %m/%d")
                ))
                .size(12)
                .style(Color::from_rgb(0.5, 0.5, 0.5))
            } else {
                text("").size(12)
            }
        } else {
            text("").size(12)
        }
    } else {
        text("").size(12)
    };

    container(
        column![
            row![city_text, Space::with_width(Length::Fill), data_badge]
                .align_items(Alignment::Center),
            api_time,
            cycle_time,
            Space::with_height(Length::Fixed(10.0)),
            temp_text,
            Space::with_height(Length::Fixed(10.0)),
            time_info,
            Space::with_height(Length::Fill),
            time_nav,
        ]
        .spacing(4)
        .align_items(Alignment::Center),
    )
    .padding(16)
    .style(theme::Container::Box)
    .width(Length::Fixed(320.0))
    .height(Length::Fixed(324.0))
    .into()
}

pub fn create_weather_grid<'a>(
    app: &'a WeatherApp,
    data_point: &'a WeatherDataPoint,
) -> Element<'a, Message> {
    let mut weather_cards: Vec<Element<Message>> = Vec::new();

    // Extract data from weather point
    let (
        wind_spd,
        wind_dir,
        wind_gust,
        visibility,
        cloud_cover,
        cloud_ceiling,
        pressure,
        solar_flux,
        thunder_pct,
        cape,
        wbg_temp,
    ) = match data_point {
        WeatherDataPoint::Observation(obs) => (
            obs.wind_spd_10m_mph,
            obs.wind_dir_10m_deg_fm_n,
            obs.wind_gust_10m_mph,
            obs.visibility_m,
            obs.cloud_cover_pct,
            obs.cloud_ceiling_m,
            obs.pressure_h_pa,
            obs.solar_flux_w_m2,
            None,
            None,
            None,
        ),
        WeatherDataPoint::Forecast(fc) => (
            fc.wind_spd_10m_mph,
            fc.wind_dir_10m_deg_fm_n,
            fc.wind_gust_10m_mph,
            fc.visibility_m,
            fc.cloud_cover_pct,
            fc.cloud_ceiling_m,
            None,
            fc.solar_flux_w_m2,
            fc.prob_thunderstorm_pct,
            fc.cape_surface_j_kg,
            fc.wbg_temp_2m_f,
        ),
    };

    // Create all weather cards using simplified functions
    weather_cards.push(create_wind_card(wind_spd, wind_dir, wind_gust, app));
    weather_cards.push(create_solar_flux_card(solar_flux, app));
    weather_cards.push(create_cloud_cover_card(cloud_cover, cloud_ceiling, app));
    weather_cards.push(create_visibility_card(visibility, app));
    weather_cards.push(create_apparent_temp_card(data_point, app));
    weather_cards.push(create_dewpoint_card(data_point, app));
    weather_cards.push(create_wbgt_card(wbg_temp, app));
    weather_cards.push(create_humidity_card(data_point, app));
    weather_cards.push(create_thunderstorm_card(thunder_pct, app));
    weather_cards.push(create_cape_card(cape, app));
    weather_cards.push(create_pressure_card(pressure, app));
    weather_cards.push(create_precipitation_card(app, data_point));

    // Arrange cards in rows of 4
    let mut weather_grid = column![].spacing(12);
    let mut card_iter = weather_cards.into_iter();

    loop {
        let mut row_widget = row![].spacing(12);
        let mut cards_in_row = 0;

        for _ in 0..4 {
            if let Some(card) = card_iter.next() {
                row_widget = row_widget.push(card);
                cards_in_row += 1;
            } else {
                break;
            }
        }

        if cards_in_row == 0 {
            break;
        }

        for _ in cards_in_row..4 {
            row_widget = row_widget.push(Space::with_width(Length::Fill));
        }

        weather_grid = weather_grid.push(row_widget);
    }

    weather_grid.into()
}

fn create_wind_card(
    wind_spd: Option<f64>,
    wind_dir: Option<f64>,
    wind_gust: Option<f64>,
    app: &WeatherApp,
) -> Element<Message> {
    let has_data = wind_spd.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Wind").size(14).style(dimmed_color)].spacing(4);

    if let Some(spd) = wind_spd {
        content = content.push(text(format!("{:.0} mph", spd)).size(20));

        let mut detail_parts = Vec::new();
        if let Some(dir) = wind_dir {
            detail_parts.push(format!("from {}°", dir));
        }
        if let Some(gust) = wind_gust {
            detail_parts.push(format!("gusts {:.0}", gust));
        }

        if !detail_parts.is_empty() {
            content = content.push(text(detail_parts.join(", ")).size(10));
        }
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::WindSpeed {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::WindSpeed))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_solar_flux_card(solar_flux: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = solar_flux.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Solar Flux").size(14).style(dimmed_color)].spacing(4);

    if let Some(solar) = solar_flux {
        content = content.push(text(format!("{:.0}", solar)).size(24));
        content = content.push(text("W/m²").size(12));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::SolarFlux {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(10)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::SolarFlux))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

// Simplified versions of remaining card functions
fn create_cloud_cover_card(
    cloud_cover: Option<f64>,
    cloud_ceiling: Option<f64>,
    app: &WeatherApp,
) -> Element<Message> {
    let has_data = cloud_cover.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Cloud Cover").size(14).style(dimmed_color)].spacing(4);

    if let Some(cloud) = cloud_cover {
        content = content.push(text(format!("{:.0}%", cloud)).size(24));
        if let Some(ceiling) = cloud_ceiling {
            content = content.push(text(format!("Ceiling: {:.0} ft", ceiling * 3.28084)).size(12));
        }
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::CloudCover {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::CloudCover))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_visibility_card(visibility: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = visibility.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Visibility").size(14).style(dimmed_color)].spacing(4);

    if let Some(vis) = visibility {
        content = content.push(text(format!("{:.1}", vis / 1609.34)).size(24));
        content = content.push(text("miles").size(12));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::Visibility {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::Visibility))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_apparent_temp_card<'a>(
    data_point: &'a WeatherDataPoint,
    app: &'a WeatherApp,
) -> Element<'a, Message> {
    let apparent_temp = match data_point {
        WeatherDataPoint::Observation(obs) => {
            if let Some(temp) = obs.temperature_2m_f {
                let humidity = if let Some(h) = obs.specific_humidity_2m_dg_kg {
                    h / 10.0
                } else if let Some(d) = obs.dewpoint_2m_f {
                    weather::dewpoint_to_relative_humidity(d, temp)
                } else {
                    50.0
                };
                let wind = obs.wind_spd_10m_mph.unwrap_or(0.0);
                Some(weather::calculate_apparent_temperature(
                    temp, humidity, wind,
                ))
            } else {
                None
            }
        }
        WeatherDataPoint::Forecast(fc) => {
            if let Some(temp) = fc.temperature_2m_f {
                let humidity = if let Some(d) = fc.dewpoint_2m_f {
                    weather::dewpoint_to_relative_humidity(d, temp)
                } else {
                    50.0
                };
                let wind = fc.wind_spd_10m_mph.unwrap_or(0.0);
                Some(weather::calculate_apparent_temperature(
                    temp, humidity, wind,
                ))
            } else {
                None
            }
        }
    };

    let has_data = apparent_temp.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Apparent Temp").size(14).style(dimmed_color)].spacing(4);

    if let Some(feels_like) = apparent_temp {
        content = content.push(text(format!("{:.0}°F", feels_like)).size(24));
        if let Some(temp) = data_point.temperature() {
            if feels_like < temp && temp <= 50.0 {
                content = content.push(
                    text("Wind Chill")
                        .size(10)
                        .style(Color::from_rgb(0.0, 0.5, 1.0)),
                );
            } else if feels_like > temp && temp >= 80.0 {
                content = content.push(
                    text("Heat Index")
                        .size(10)
                        .style(Color::from_rgb(1.0, 0.5, 0.0)),
                );
            }
        }
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::ApparentTemperature {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(
        WeatherElement::ApparentTemperature,
    ))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_dewpoint_card<'a>(
    data_point: &'a WeatherDataPoint,
    app: &'a WeatherApp,
) -> Element<'a, Message> {
    let dewpoint_value = match data_point {
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
    };

    let has_data = dewpoint_value.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Dewpoint").size(14).style(dimmed_color)].spacing(4);

    if let Some(dewpoint) = dewpoint_value {
        content = content.push(text(format!("{:.0}°F", dewpoint)).size(24));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::Dewpoint {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::Dewpoint))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_wbgt_card(wbg_temp: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = wbg_temp.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("WBGT").size(14).style(dimmed_color)].spacing(4);

    if let Some(wbgt) = wbg_temp {
        content = content.push(text(format!("{:.0}°F", wbgt)).size(24));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::WBGT {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::WBGT))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_humidity_card<'a>(
    data_point: &'a WeatherDataPoint,
    app: &'a WeatherApp,
) -> Element<'a, Message> {
    let humidity_value = match data_point {
        WeatherDataPoint::Observation(obs) => {
            if let Some(humidity) = obs.specific_humidity_2m_dg_kg {
                Some(humidity / 10.0)
            } else if let (Some(dewpoint), Some(temp)) = (obs.dewpoint_2m_f, obs.temperature_2m_f) {
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
    };

    let has_data = humidity_value.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Humidity").size(14).style(dimmed_color)].spacing(4);

    if let Some(humidity) = humidity_value {
        content = content.push(text(format!("{:.0}%", humidity)).size(24));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::Humidity {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::Humidity))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_thunderstorm_card(thunder_pct: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = thunder_pct.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("T-Storm Prob").size(14).style(dimmed_color)].spacing(4);

    if let Some(thunder) = thunder_pct {
        content = content.push(text(format!("{:.0}%", thunder)).size(24));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::ThunderstormProbability {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(
        WeatherElement::ThunderstormProbability,
    ))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_cape_card(cape: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = cape.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("CAPE").size(14).style(dimmed_color)].spacing(4);

    if let Some(cape_value) = cape {
        content = content.push(text(format!("{:.0}", cape_value)).size(24));
        content = content.push(text("J/kg").size(12));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::CAPE {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::CAPE))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_pressure_card(pressure: Option<f64>, app: &WeatherApp) -> Element<Message> {
    let has_data = pressure.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Pressure").size(14).style(dimmed_color)].spacing(2);

    if let Some(press) = pressure {
        let press_inhg = press * 0.02953;
        content = content.push(text(format!("{:.2}", press_inhg)).size(24));
        content = content.push(text("inHg").size(10));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::Pressure {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(WeatherElement::Pressure))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}

fn create_precipitation_card<'a>(
    app: &'a WeatherApp,
    data_point: &'a WeatherDataPoint,
) -> Element<'a, Message> {
    let precip_prob = app.get_precipitation_probability(data_point.valid_date());
    let has_data = precip_prob.is_some();
    let dimmed_color = if has_data {
        Color::from_rgb(0.0, 0.0, 0.0)
    } else {
        Color::from_rgb(0.7, 0.7, 0.7)
    };
    let mut content = column![text("Precip Prob").size(14).style(dimmed_color)].spacing(4);

    if let Some(prob) = precip_prob {
        content = content.push(text(format!("{:.0}%", prob)).size(24));
    } else {
        content = content.push(text("").size(24));
    }

    let card_style = if app.selected_weather_element == WeatherElement::PrecipitationProbability {
        theme::Button::Primary
    } else {
        theme::Button::Secondary
    };

    button(
        container(content)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Fixed(100.0)),
    )
    .on_press(Message::SelectWeatherElement(
        WeatherElement::PrecipitationProbability,
    ))
    .style(card_style)
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .into()
}
