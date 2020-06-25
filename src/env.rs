use std::net::SocketAddr;

#[derive(Debug)]
pub enum EnvErrorKind {
    Env(std::env::VarError),
    Parse,
}

#[derive(Debug)]
pub struct EnvError<'a>(&'a str, EnvErrorKind);

pub fn var<T: std::str::FromStr>(key: &'static str) -> Result<T, EnvError<'static>> {
    let str = std::env::var(key).map_err(|e| EnvError(key, EnvErrorKind::Env(e)))?;
    str.parse().or(Err(EnvError(key, EnvErrorKind::Parse)))
}

pub fn with_default<'a, T>(
    var_opt: Result<T, EnvError<'a>>,
    default: T,
) -> Result<T, EnvError<'a>> {
    match var_opt {
        Err(EnvError(_, EnvErrorKind::Env(std::env::VarError::NotPresent))) => Ok(default),
        res => res,
    }
}

pub fn get_conf<'a>() -> Result<EnvConfiguration, EnvError<'a>> {
    let email = var("DIWI_EMAIL")?;
    let password = var("DIWI_PASSWORD")?;
    let club = var("DIWI_CLUB")?;
    let socketaddr = with_default(var("DIWI_SOCKETADDR"), "127.0.0.1:8080".parse().unwrap())?;

    Ok(EnvConfiguration {
        email,
        password,
        club,
        socketaddr,
    })
}

#[derive(Clone)]
pub struct EnvConfiguration {
    pub email: String,
    pub password: String,
    pub club: String,
    pub socketaddr: SocketAddr,
}
