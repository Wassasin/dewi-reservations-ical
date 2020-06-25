use crate::env::EnvConfiguration;
use crate::{Error, Result};
use actix_web::client::ClientBuilder;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct LoginParams<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub club_id: u32,
}

#[derive(Deserialize, Serialize)]
pub struct Reservation {
    pub id: u32,
    pub name: String,
    pub model: ReservationModel,
}

#[derive(Deserialize, Serialize)]
pub struct ReservationModel {
    pub date: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationsResponse {
    pub upcoming_reservations: Vec<Reservation>,
    pub old_reservations: Vec<Reservation>,
}

pub async fn login(conf: &EnvConfiguration) -> Result<LoginResponse> {
    let client = ClientBuilder::new()
        .no_default_headers()
        .header("User-Agent", "dewi-reservations-ical")
        .finish();

    let mut res = client
        .post(format!(
            "https://{}.dewi-online.nl/api/app/login",
            conf.club
        ))
        .send_form(&LoginParams {
            email: &conf.email,
            password: &conf.password,
        })
        .await?;

    if res.status() == 422 {
        return Err(Error::AuthenticationFailure);
    }

    if res.status() != 200 {
        return Err(Error::UpstreamFailure);
    }

    Ok(res.json().await?)
}

pub async fn get_reservations(
    conf: &EnvConfiguration,
    params: &LoginResponse,
) -> Result<ReservationsResponse> {
    let client = ClientBuilder::new()
        .no_default_headers()
        .header("User-Agent", "dewi-reservations-ical")
        .bearer_auth(params.access_token.as_str())
        .finish();

    // club_id is integer, and thus safe to insert into the url.
    Ok(client
        .get(format!(
            "https://{}.dewi-online.nl/api/app/club/{}/reservations",
            conf.club, params.club_id
        ))
        .send()
        .await?
        .json()
        .await?)
}
