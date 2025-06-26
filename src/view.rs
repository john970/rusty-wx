use crate::app::{Message, WeatherApp};
use crate::components;
use crate::meteogram::Meteogram;
use iced::{
    theme,
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length,
};

pub fn view(app: &WeatherApp) -> Element<Message> {
    // Location selection card
    let location_card = create_location_card(app);

    // Weather display - always show the layout
    let weather_display: Element<Message> = if let Some(_weather) = &app.weather_data {
        // Get the current data point from timeline
        if let Some(data_point) = app.combined_timeline.get(app.timeline_index) {
            // Main weather card
            let temp_card = components::create_temperature_card(app, data_point);

            // Weather details grid
            let weather_grid = components::create_weather_grid(app, data_point);

            // Create meteogram for bottom
            let meteogram_container = create_meteogram_container(app);

            // Main content row - temperature on left, cards on right
            let main_content = row![
                temp_card,
                Space::with_width(Length::Fixed(12.0)),
                weather_grid
            ]
            .spacing(12)
            .width(Length::Fill);

            // Combine everything
            column![main_content, meteogram_container,]
                .spacing(12)
                .into()
        } else {
            text("No weather data available").size(16).into()
        }
    } else {
        // Show loading or error state
        components::create_status_display(app)
    };

    let content = column![location_card, weather_display,]
        .spacing(16)
        .padding(16)
        .align_items(Alignment::Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(10)
        .into()
}

fn create_location_card(app: &WeatherApp) -> Element<Message> {
    let denver_button = button(text("Denver").size(14))
        .on_press(Message::FetchWeather(
            "Denver".to_string(),
            39.7392,
            -104.9903,
        ))
        .padding([8, 16])
        .style(theme::Button::Primary);

    let miami_button = button(text("Miami").size(14))
        .on_press(Message::FetchWeather(
            "Miami".to_string(),
            25.7617,
            -80.1918,
        ))
        .padding([8, 16])
        .style(theme::Button::Primary);

    let la_button = button(text("Los Angeles").size(14))
        .on_press(Message::FetchWeather(
            "Los Angeles".to_string(),
            34.0522,
            -118.2437,
        ))
        .padding([8, 16])
        .style(theme::Button::Primary);

    let ny_button = button(text("New York").size(14))
        .on_press(Message::FetchWeather(
            "New York".to_string(),
            40.7128,
            -74.0060,
        ))
        .padding([8, 16])
        .style(theme::Button::Primary);

    let city_buttons = row![denver_button, la_button, miami_button, ny_button].spacing(10);

    // Custom coordinate inputs
    let lat_input = text_input("Latitude", &app.lat_input)
        .on_input(Message::LatInputChanged)
        .padding(8)
        .size(14)
        .width(Length::Fixed(100.0));

    let lon_input = text_input("Longitude", &app.lon_input)
        .on_input(Message::LonInputChanged)
        .padding(8)
        .size(14)
        .width(Length::Fixed(100.0));

    let custom_button = button(text("Get Weather").size(14))
        .on_press(Message::FetchCustomLocation)
        .padding([8, 16])
        .style(theme::Button::Primary);

    let custom_inputs = row![
        text("Or enter coordinates:").size(14),
        lat_input,
        lon_input,
        custom_button
    ]
    .spacing(10)
    .align_items(Alignment::Center);

    container(
        column![
            text("Select Location").size(18),
            row![
                city_buttons,
                Space::with_width(Length::Fixed(30.0)),
                custom_inputs
            ]
            .align_items(Alignment::Center),
        ]
        .spacing(12)
        .align_items(Alignment::Center),
    )
    .padding(16)
    .style(theme::Container::Box)
    .width(Length::Fill)
    .center_x()
    .into()
}

fn create_meteogram_container(app: &WeatherApp) -> Element<Message> {
    let (precip_1hr, precip_6hr) = if let Some(weather) = &app.weather_data {
        (
            weather.forecasts_precip_1hr.clone(),
            weather.forecasts_precip_6hr.clone(),
        )
    } else {
        (Vec::new(), Vec::new())
    };

    let (unified_temp_min, unified_temp_max) = app.get_unified_temp_range(&app.combined_timeline);
    let meteogram = Meteogram::new(
        app.combined_timeline.clone(),
        app.timeline_index,
        app.selected_weather_element.clone(),
        precip_1hr,
        precip_6hr,
        unified_temp_min,
        unified_temp_max,
    );

    let meteogram_canvas = iced::widget::canvas::Canvas::new(meteogram)
        .width(Length::Fixed(4000.0))
        .height(Length::Fixed(230.0));

    let meteogram_scrollable = scrollable(meteogram_canvas)
        .id(app.meteogram_scroll_id.clone())
        .direction(scrollable::Direction::Horizontal(
            scrollable::Properties::default(),
        ))
        .width(Length::Fill);

    // Create fixed temperature scale on the left
    let temp_scale = app.create_temp_scale(&app.combined_timeline);

    // Create fixed element scale on the right
    let element_scale =
        app.create_element_scale(&app.combined_timeline, &app.selected_weather_element);

    let meteogram_with_scale = row![temp_scale, meteogram_scrollable, element_scale].spacing(0);

    container(meteogram_with_scale)
        .padding(16)
        .style(theme::Container::Box)
        .width(Length::Fill)
        .into()
}
