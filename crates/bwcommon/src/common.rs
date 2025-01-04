use actix_web::error::PayloadError;
use actix_web::HttpResponse;
use anyhow::Result;

#[macro_export]
macro_rules! ensure {
    ($cond:expr $(,)?) => {
        if !$cond {
            anyhow::Result::Err(anyhow::Error::msg(std::stringify!($cond)))?
        }
    };
}

use tracing::error;

pub async fn check_auth4(
    req: &actix_web::HttpRequest,
    pool: bb8_postgres::bb8::Pool<
        bb8_postgres::PostgresConnectionManager<bb8_postgres::tokio_postgres::NoTls>,
    >,
) -> Result<Option<i64>, anyhow::Error> {
    if let Some(cookie_username) = req.cookie("username") {
        if let Some(cookie_token) = req.cookie("token") {
            let con = pool.get().await?;
            let row = con
                .query_one(
                    "select id, token from account where username = $1",
                    &[&cookie_username.value()],
                )
                .await?;

            let db_idtoken = (row.get::<_, i64>(0), row.get::<_, String>(1));

            if cookie_token.value() == db_idtoken.1.as_str() {
                return Ok(Some(db_idtoken.0));
            }
        }
    }

    Ok(None)
}

#[derive(Debug)]
pub struct MyError {
    pub err: anyhow::Error,
}

// pub struct Error {
//     depth: usize,
//     inner: ErrorInner,
// }

impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl actix_web::error::ResponseError for MyError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        error!("{self:?}");
        HttpResponse::InternalServerError().body("Something went wrong :)".to_string())
    }
}

impl From<std::num::TryFromIntError> for MyError {
    fn from(err: std::num::TryFromIntError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<image::ImageError> for MyError {
    fn from(err: image::ImageError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<walkdir::Error> for MyError {
    fn from(err: walkdir::Error) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<PayloadError> for MyError {
    fn from(err: PayloadError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<anyhow::Error> for MyError {
    fn from(err: anyhow::Error) -> MyError {
        MyError { err }
    }
}

impl From<&str> for MyError {
    fn from(err: &str) -> MyError {
        MyError {
            err: anyhow::Error::msg(err.to_string()),
        }
    }
}

impl From<actix_multipart::MultipartError> for MyError {
    fn from(err: actix_multipart::MultipartError) -> MyError {
        MyError {
            err: anyhow::Error::msg(err.to_string()),
        }
    }
}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> MyError {
        MyError {
            err: anyhow::Error::msg(err.to_string()),
        }
    }
}

impl From<std::time::SystemTimeError> for MyError {
    fn from(err: std::time::SystemTimeError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(err: std::num::ParseIntError) -> MyError {
        MyError { err: err.into() }
    }
}

// impl From<hex::FromHexError> for MyError {
//     fn from(err: hex::FromHexError) -> MyError {
//         MyError { err: err.into() }
//     }
// }

impl<E: 'static + Send + Sync + std::error::Error> From<bb8_postgres::bb8::RunError<E>>
    for MyError
{
    fn from(err: bb8_postgres::bb8::RunError<E>) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<bb8_postgres::tokio_postgres::Error> for MyError {
    fn from(err: bb8_postgres::tokio_postgres::Error) -> MyError {
        MyError { err: err.into() }
    }
}

// impl From<r2d2_postgres::postgres::Error> for MyError {
//     fn from(err: r2d2_postgres::postgres::Error) -> MyError {
//         MyError { err: err.into() }
//     }
// }

impl<T: 'static + Send + Sync> From<std::sync::mpsc::SendError<T>> for MyError {
    fn from(err: std::sync::mpsc::SendError<T>) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<handlebars::RenderError> for MyError {
    fn from(err: handlebars::RenderError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<std::env::VarError> for MyError {
    fn from(err: std::env::VarError) -> MyError {
        MyError { err: err.into() }
    }
}

impl From<awc::error::SendRequestError> for MyError {
    fn from(err: awc::error::SendRequestError) -> MyError {
        MyError {
            err: anyhow::anyhow!("there was an error: {}", err),
        }
    }
}

// Result<PooledConnection<SqliteConnectionManager>, Error>

impl From<actix_web::error::BlockingError> for MyError {
    fn from(err: actix_web::error::BlockingError) -> MyError {
        MyError { err: err.into() }
    }
}
