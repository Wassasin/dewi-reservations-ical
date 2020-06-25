//! HTTP server that periodically fetches reservations from a Dewi-online sports facility API and yields an ical file.

use crate::env::EnvConfiguration;
use actix_web::{
    client::SendRequestError,
    web::{self, Data},
    App, HttpResponse, HttpServer, ResponseError,
};
use awc::error::JsonPayloadError;
use chrono::{DateTime, Utc};
use chrono_tz::Europe::Amsterdam;
use serde::Serialize;

pub mod env;
pub mod upstream;

#[derive(Debug, Serialize)]
pub enum Error {
    UpstreamFailure,
    AuthenticationFailure,
    Inconsistency,
}

impl core::convert::From<SendRequestError> for Error {
    fn from(_: SendRequestError) -> Self {
        Error::UpstreamFailure
    }
}

impl core::convert::From<JsonPayloadError> for Error {
    fn from(_: JsonPayloadError) -> Self {
        Error::Inconsistency
    }
}

pub type Result<T> = core::result::Result<T, Error>;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        let mut response = match self {
            Error::UpstreamFailure => HttpResponse::ServiceUnavailable(),
            Error::AuthenticationFailure => HttpResponse::InternalServerError(),
            Error::Inconsistency => HttpResponse::InternalServerError(),
        };

        response.json(self)
    }
}

#[derive(Serialize)]
pub struct Reservation {
    id: u32,
    name: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl std::convert::TryFrom<upstream::Reservation> for Reservation {
    type Error = chrono::format::ParseError;
    fn try_from(value: upstream::Reservation) -> std::result::Result<Self, Self::Error> {
        use chrono::offset::TimeZone;

        let model = &value.model;
        let id = value.id;
        let name = value.name;
        let start = Amsterdam
            .datetime_from_str(&format!("{} {}", model.date, model.start_time), "%F %T")?
            .with_timezone(&Utc);
        let end = Amsterdam
            .datetime_from_str(&format!("{} {}", model.date, model.end_time), "%F %T")?
            .with_timezone(&Utc);

        Ok(Reservation {
            id,
            name,
            start,
            end,
        })
    }
}

async fn compute_reservations(conf: &EnvConfiguration) -> Result<Vec<Reservation>> {
    let params = upstream::login(conf).await?;
    let data = upstream::get_reservations(conf, &params).await?;

    use std::iter::FromIterator;
    Result::from_iter(data.upcoming_reservations.into_iter().map(|r| {
        use std::convert::TryInto;
        r.try_into().map_err(|_| Error::Inconsistency)
    }))
}

async fn get_json(conf: Data<EnvConfiguration>) -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(compute_reservations(&conf).await?))
}

fn instant_to_icalstr(t: &DateTime<Utc>) -> String {
    t.format("%Y%m%dT%H%M%SZ").to_string()
}

async fn get_ical(conf: Data<EnvConfiguration>) -> Result<HttpResponse> {
    use ics::{properties::*, *};

    let reservations = compute_reservations(&conf).await?;

    let mut calendar = ICalendar::new("2.0", "dewi-reservations");
    calendar.add_timezone(TimeZone::new(
        "UTC",
        ZoneTime::standard("19700329T020000", "+0000", "+0000"),
    ));

    reservations.into_iter().for_each(|r| {
        let mut event = Event::new(format!("{}", r.id), instant_to_icalstr(&r.start));
        event.push(DtStart::new(instant_to_icalstr(&r.start)));
        event.push(DtEnd::new(instant_to_icalstr(&r.end)));
        event.push(Summary::new(r.name));
        event.add_alarm(Alarm::display(
            Trigger::new("-P0DT1H00M0S"),
            Description::new("Time to sport!~"),
        ));
        calendar.add_event(event);
    });

    Ok(HttpResponse::Ok()
        .content_type("text/calendar")
        .body(calendar.to_string()))
}

#[derive(Clone)]
pub struct Conf(std::sync::Arc<EnvConfiguration>);

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let configuration = env::get_conf().unwrap();
    let socketaddr = configuration.socketaddr;

    HttpServer::new(move || {
        let configuration = configuration.clone();

        App::new()
            .data(configuration)
            .service(web::resource("/json").to(get_json))
            .service(web::resource("/ical").to(get_ical))
    })
    .bind(socketaddr)?
    .run()
    .await
}
