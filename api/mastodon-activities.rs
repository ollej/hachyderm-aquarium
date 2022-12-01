use librustymastodon::{build_school, get_activities, FishData, FishLegend, InputData, Legend};
use std::error::Error;
use vercel_lambda::{error::VercelError, lambda, IntoResponse, Request};

#[allow(dead_code)]
fn handler(_: Request) -> Result<impl IntoResponse, VercelError> {
    let data = get_activities().map(build_school);
    let legend = Legend {
        description: "Weekly activity on Hachyderm.io\nSize show number of registrations.\nSpeed show number of statuses.\nBubbles show number of logins."
            .to_string(),
        fish_legends: vec![
            FishLegend {
                fish: "clownfish".to_string(),
                description: "One week of activity".to_string(),
            },
            FishLegend {
                fish: "ferris".to_string(),
                description: "Ferris helps to monitor the aquarium".to_string(),
            },
        ],
    };
    match data {
        Ok(mut school) => {
            school.push(FishData {
                fish: "ferris".to_string(),
                size: 1.0,
                speed: 1.0,
                bubbles: 0.0,
            });
            librustymastodon::build_response(InputData {
                legend: Some(legend),
                school,
            })
        }
        Err(err) => librustymastodon::build_error_response(err.to_string()),
    }
}

// Start the runtime with the handler
#[allow(dead_code)]
fn main() -> Result<(), Box<dyn Error>> {
    Ok(lambda!(handler))
}
