use librustymastodon::{build_school, get_activities, FishLegend, InputData, Legend};
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
        ],
    };
    match data {
        Ok(school) => librustymastodon::build_response(InputData {
            legend: Some(legend),
            school,
        }),
        Err(err) => librustymastodon::build_error_response(err.to_string()),
    }
}

// Start the runtime with the handler
#[allow(dead_code)]
fn main() -> Result<(), Box<dyn Error>> {
    Ok(lambda!(handler))
}
