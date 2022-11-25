use std::{
    env, fmt,
    ops::{Add, Div, Mul, Sub},
};
use {
    http::StatusCode,
    nanoserde::{DeJson, DeJsonErr, SerJson},
    vercel_lambda::{error::VercelError, Body, Response},
};

fn map_range<T: Copy>(from_range: (T, T), to_range: (T, T), s: T) -> T
where
    T: Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Div<T, Output = T>,
{
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

#[derive(Debug, PartialEq)]
pub enum RustyMastodonError {
    ExternalRequest,
    Configuration,
    Parsing,
}

impl fmt::Display for RustyMastodonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RustyMastodonError::ExternalRequest => {
                write!(f, "Failed reading data from external source")
            }
            RustyMastodonError::Parsing => {
                write!(f, "Failed parsing data from external source")
            }
            RustyMastodonError::Configuration => {
                write!(f, "Configuration error")
            }
        }
    }
}

impl From<reqwest::Error> for RustyMastodonError {
    fn from(_: reqwest::Error) -> Self {
        RustyMastodonError::ExternalRequest
    }
}

impl From<DeJsonErr> for RustyMastodonError {
    fn from(_: DeJsonErr) -> Self {
        RustyMastodonError::Parsing
    }
}

impl From<reqwest::header::InvalidHeaderValue> for RustyMastodonError {
    fn from(_: reqwest::header::InvalidHeaderValue) -> Self {
        RustyMastodonError::Configuration
    }
}

impl From<env::VarError> for RustyMastodonError {
    fn from(_: env::VarError) -> Self {
        RustyMastodonError::Configuration
    }
}

impl std::error::Error for RustyMastodonError {}

impl From<RustyMastodonError> for VercelError {
    fn from(error: RustyMastodonError) -> Self {
        VercelError::new(&format!("{}", error))
    }
}

#[derive(Clone, Debug, Default, SerJson)]
struct ErrorResponse {
    message: String,
}

#[derive(Clone, Debug, Default, SerJson)]
pub struct InputData {
    pub legend: Option<Legend>,
    pub school: Vec<FishData>,
}

#[derive(Clone, Debug, SerJson)]
pub struct Legend {
    pub description: String,
    pub fish_legends: Vec<FishLegend>,
}

#[derive(Clone, Debug, SerJson)]
pub struct FishLegend {
    pub fish: String,
    pub description: String,
}

#[derive(Clone, Debug, SerJson)]
#[nserde(default)]
pub struct FishData {
    pub fish: String,
    pub size: f32,
    pub speed: f32,
    pub bubbles: f32,
}

impl Default for FishData {
    fn default() -> FishData {
        FishData {
            fish: "clownfish".to_string(),
            size: 1.0,
            speed: 1.0,
            bubbles: 1.0,
        }
    }
}

pub type Activity = Vec<Week>;

#[derive(Default, Debug, Clone, PartialEq, DeJson)]
pub struct Week {
    pub week: String,
    pub statuses: String,
    pub logins: String,
    pub registrations: String,
}

pub fn get_activities() -> Result<Activity, RustyMastodonError> {
    let content = reqwest::blocking::Client::builder()
        .user_agent(env::var("MASTODON_API_USER_AGENT")?)
        .build()?
        .get(format!(
            "{}/api/v1/instance/activity",
            env::var("MASTODON_API_URL")?
        ))
        //.bearer_auth(env::var("MASTODON_API_TOKEN")?)
        .send()?
        .text()?;
    //fs::write("/tmp/mastodon-instances.json", &content).expect("Unable to write file");
    let activities: Activity = DeJson::deserialize_json(&content).unwrap();
    //eprintln!(
    //    "Loaded {} out of {} instances...",
    //    json.instances.len(),
    //    json.pagination.total
    //);
    Ok(activities)
}

pub fn build_school(activities: Activity) -> Vec<FishData> {
    activities
        .iter()
        .take(100)
        .map(|week| {
            let registrations = week.registrations.parse().unwrap_or(0.);
            let statuses = week.statuses.parse().unwrap_or(0.);
            let logins = week.logins.parse().unwrap_or(0.);
            FishData {
                fish: "clownfish".to_string(),
                size: map_range((0., 15_000.), (0.2, 1.0), registrations),
                speed: map_range((0., 100_000.), (0.2, 1.0), statuses),
                bubbles: map_range((0., 20_000.), (0.2, 1.0), logins),
            }
        })
        .collect()
}

/// Build a Vercel Response from a serializeable body
///
/// # Arguments
///
/// * `body` - Serialize into a JSON body
///
/// # Examples
/// ```
/// use {libquiz::{build_response}, vercel_lambda::Body};
/// let response = build_response("body string").unwrap();
/// assert_eq!(&Body::Text("\"body string\"".to_string()), response.body());
/// assert_eq!("application/json", response.headers().get(http::header::CONTENT_TYPE).unwrap());
/// ```
pub fn build_response<S>(body: S) -> Result<Response<Body>, VercelError>
where
    S: SerJson,
{
    let body = Body::Text(SerJson::serialize_json(&body));
    Response::builder()
        .status(StatusCode::OK)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            http::header::CACHE_CONTROL,
            format!(
                "max-age={}, public",
                env::var("HTTP_CACHE_IN_SECONDS").unwrap_or("60".to_string())
            ),
        )
        .body(body)
        .map_err(|_| VercelError::new("Couldn't build response"))
}

pub fn build_error_response<S>(error_message: S) -> Result<Response<Body>, VercelError>
where
    S: Into<String>,
{
    let error = ErrorResponse {
        message: error_message.into(),
    };
    let body = Body::Text(SerJson::serialize_json(&error));
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(body)
        .map_err(|_| VercelError::new("Couldn't build response"))
}
