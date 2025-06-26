
use serde::{Deserialize, Serialize};

// All the weather data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWeatherData {
    pub latitude: f64,
    pub longitude: f64,
    #[serde(rename = "gridId")]
    pub grid_id: Option<i32>,
    #[serde(rename = "normalTemperatures")]
    pub normal_temperatures: Vec<NormalTemperature>,
    #[serde(rename = "observationsInstant")]
    pub observations_instant: Vec<ObservationInstant>,
    #[serde(rename = "forecastsInstant")]
    pub forecasts_instant: Vec<ForecastInstant>,
    #[serde(rename = "forecastsPrecip1hr", default)]
    pub forecasts_precip_1hr: Vec<ForecastPrecip>,
    #[serde(rename = "forecastsPrecip6hr", default)]
    pub forecasts_precip_6hr: Vec<ForecastPrecip>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalTemperature {
    #[serde(rename = "validDate")]
    pub valid_date: String,
    #[serde(rename = "temperature2mF")]
    pub temperature_2m_f: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationInstant {
    #[serde(rename = "validDate")]
    pub valid_date: String,
    #[serde(rename = "temperature2mF")]
    pub temperature_2m_f: Option<f64>,
    #[serde(rename = "dewpoint2mF")]
    pub dewpoint_2m_f: Option<f64>,
    #[serde(rename = "specificHumidity2mDgKg")]
    pub specific_humidity_2m_dg_kg: Option<f64>,
    #[serde(rename = "cloudCoverPct")]
    pub cloud_cover_pct: Option<f64>,
    #[serde(rename = "cloudCeilingM")]
    pub cloud_ceiling_m: Option<f64>,
    #[serde(rename = "visibilityM")]
    pub visibility_m: Option<f64>,
    #[serde(rename = "pressureHPa")]
    pub pressure_h_pa: Option<f64>,
    #[serde(rename = "windDir10mDegFmN")]
    pub wind_dir_10m_deg_fm_n: Option<f64>,
    #[serde(rename = "windSpd10mMph")]
    pub wind_spd_10m_mph: Option<f64>,
    #[serde(rename = "windGust10mMph")]
    pub wind_gust_10m_mph: Option<f64>,
    #[serde(rename = "solarFluxWM2")]
    pub solar_flux_w_m2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastInstant {
    #[serde(rename = "validDate")]
    pub valid_date: String,
    #[serde(rename = "cycleDate")]
    pub cycle_date: String,
    #[serde(rename = "temperature2mF")]
    pub temperature_2m_f: Option<f64>,
    #[serde(rename = "dewpoint2mF")]
    pub dewpoint_2m_f: Option<f64>,
    #[serde(rename = "wbgTemp2mF")]
    pub wbg_temp_2m_f: Option<f64>,
    #[serde(rename = "cloudCeilingM")]
    pub cloud_ceiling_m: Option<f64>,
    #[serde(rename = "cloudCoverPct")]
    pub cloud_cover_pct: Option<f64>,
    #[serde(rename = "visibilityM")]
    pub visibility_m: Option<f64>,
    #[serde(rename = "capeSurfaceJKg")]
    pub cape_surface_j_kg: Option<f64>,
    #[serde(rename = "probThunderstormPct")]
    pub prob_thunderstorm_pct: Option<f64>,
    #[serde(rename = "windDir10mDegFmN")]
    pub wind_dir_10m_deg_fm_n: Option<f64>,
    #[serde(rename = "windSpd10mMph")]
    pub wind_spd_10m_mph: Option<f64>,
    #[serde(rename = "windGust10mMph")]
    pub wind_gust_10m_mph: Option<f64>,
    #[serde(rename = "solarFluxWM2")]
    pub solar_flux_w_m2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPrecip {
    #[serde(rename = "validDate")]
    pub valid_date: String,
    #[serde(rename = "probPrecipPct")]
    pub prob_precip_pct: f64,
}

#[derive(Debug, Clone)]
pub enum WeatherDataPoint {
    Observation(ObservationInstant),
    Forecast(ForecastInstant),
}

impl WeatherDataPoint {
    pub fn valid_date(&self) -> &str {
        match self {
            WeatherDataPoint::Observation(obs) => &obs.valid_date,
            WeatherDataPoint::Forecast(fc) => &fc.valid_date,
        }
    }

    pub fn temperature(&self) -> Option<f64> {
        match self {
            WeatherDataPoint::Observation(obs) => obs.temperature_2m_f,
            WeatherDataPoint::Forecast(fc) => fc.temperature_2m_f,
        }
    }
}

// Helper functions for humidity/dewpoint conversions
pub fn dewpoint_to_relative_humidity(dewpoint_f: f64, temp_f: f64) -> f64 {
    let temp_c = (temp_f - 32.0) * 5.0 / 9.0;
    let dewpoint_c = (dewpoint_f - 32.0) * 5.0 / 9.0;
    let a = 17.27;
    let b = 237.7;
    let alpha_t = (a * temp_c) / (b + temp_c) + (temp_c / (b + temp_c)).ln();
    let alpha_td = (a * dewpoint_c) / (b + dewpoint_c) + (dewpoint_c / (b + dewpoint_c)).ln();
    100.0 * (alpha_td - alpha_t).exp()
}

pub fn relative_humidity_to_dewpoint(rh_percent: f64, temp_f: f64) -> f64 {
    let temp_c = (temp_f - 32.0) * 5.0 / 9.0;
    let a = 17.27;
    let b = 237.7;
    let alpha = (rh_percent / 100.0).ln() + (a * temp_c) / (b + temp_c);
    let dewpoint_c = (b * alpha) / (a - alpha);
    dewpoint_c * 9.0 / 5.0 + 32.0
}

// Calculate heat index using NWS formula
pub fn calculate_heat_index(temp_f: f64, rh_percent: f64) -> f64 {
    if temp_f < 80.0 {
        return 0.5 * (temp_f + 61.0 + ((temp_f - 68.0) * 1.2) + (rh_percent * 0.094));
    }
    let t = temp_f;
    let rh = rh_percent;
    let mut hi = -42.379 + 2.04901523 * t + 10.14333127 * rh
        - 0.22475541 * t * rh
        - 0.00683783 * t * t
        - 0.05481717 * rh * rh
        + 0.00122874 * t * t * rh
        + 0.00085282 * t * rh * rh
        - 0.00000199 * t * t * rh * rh;
    if rh < 13.0 && t >= 80.0 && t <= 112.0 {
        let adjustment = ((13.0 - rh) / 4.0) * ((17.0 - (t - 95.0).abs()) / 17.0).sqrt();
        hi -= adjustment;
    } else if rh > 85.0 && t >= 80.0 && t <= 87.0 {
        let adjustment = ((rh - 85.0) / 10.0) * ((87.0 - t) / 5.0);
        hi += adjustment;
    }
    hi
}

// Calculate wind chill using NWS formula
pub fn calculate_wind_chill(temp_f: f64, wind_mph: f64) -> f64 {
    if temp_f > 50.0 || wind_mph <= 3.0 {
        return temp_f;
    }
    35.74 + 0.6215 * temp_f - 35.75 * wind_mph.powf(0.16) + 0.4275 * temp_f * wind_mph.powf(0.16)
}

// Calculate apparent temperature (heat index or wind chill)
pub fn calculate_apparent_temperature(temp_f: f64, rh_percent: f64, wind_mph: f64) -> f64 {
    if temp_f <= 50.0 && wind_mph > 3.0 {
        calculate_wind_chill(temp_f, wind_mph)
    } else if temp_f >= 80.0 {
        calculate_heat_index(temp_f, rh_percent)
    } else {
        temp_f
    }
}

// The async function to fetch weather data
pub async fn fetch_weather(lat: f64, lon: f64) -> Result<ApiWeatherData, String> {
    let url = format!(
        "http://api.ottoweather.com:8001/weather?lat={}&lon={}",
        lat, lon
    );
    println!("Fetching weather from: {}", url);
    let response = reqwest::get(&url).await.map_err(|e| {
        println!("Request failed: {}", e);
        e.to_string()
    })?;
    println!("Got response with status: {}", response.status());
    if response.status().is_success() {
        response
            .json::<ApiWeatherData>()
            .await
            .map_err(|e| e.to_string())
    } else {
        Err(format!("Error fetching weather: {}", response.status()))
    }
}
