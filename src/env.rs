use std::str::FromStr;

#[derive(Debug)]
pub enum EnvError {
    Env(std::env::VarError),
    Parse,
}

pub fn var<T: FromStr>(key: &'static str) -> Result<T, EnvError> {
    let str = std::env::var(key).map_err(|e| EnvError::Env(e))?;
    str.parse().or(Err(EnvError::Parse))
}

pub fn with_default<T>(var_opt: Result<T, EnvError>, default: T) -> Result<T, EnvError> {
    match var_opt {
        Err(EnvError::Env(std::env::VarError::NotPresent)) => Ok(default),
        res => res,
    }
}

pub fn get_conf() -> Result<EnvConfiguration, EnvError> {
    let email = var("DIWI_EMAIL")?;
    let password = var("DIWI_PASSWORD")?;
    let club = var("DIWI_CLUB")?;
    let host = with_default(var("DIWI_HOST"), "127.0.0.1".into())?;
    let port = with_default(var("DIWI_PORT"), 8080)?;

    Ok(EnvConfiguration {
        email,
        password,
        club,
        host,
        port,
    })
}

#[derive(Clone)]
pub struct EnvConfiguration {
    pub email: String,
    pub password: String,
    pub club: String,
    pub host: String,
    pub port: u16,
}
