#[macro_export]
macro_rules! ensure {
    ($cond:expr $(,)?) => {
        if !$cond {
            anyhow::Result::Err(anyhow::Error::msg(std::stringify!($cond)))?
        }
    };
}

use tracing::error;

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

/// Marker stashed in a response's extensions when the response was produced by
/// a handler returning `Err(MyError)`. The postgres-logging middleware reads it
/// to fill the `error` column, matching actix's separate error path.
#[derive(Clone, Debug)]
pub struct LoggedError(pub String);

impl axum::response::IntoResponse for MyError {
    fn into_response(self) -> axum::response::Response {
        error!("{self:?}");
        let err_string = format!("{self:?}");
        let mut resp = axum::response::IntoResponse::into_response((
            http::StatusCode::INTERNAL_SERVER_ERROR,
            "Something went wrong :)",
        ));
        resp.extensions_mut().insert(LoggedError(err_string));
        resp
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http::StatusCode;

    #[test]
    fn my_error_from_str_and_display() {
        let e: MyError = "boom".into();
        assert_eq!(e.to_string(), "boom");
        // Debug delegates to the inner anyhow::Error and includes the message.
        assert!(format!("{e:?}").contains("boom"));
    }

    #[test]
    fn my_error_from_anyhow_preserves_message() {
        let e: MyError = anyhow::anyhow!("kaboom {}", 7).into();
        assert_eq!(e.to_string(), "kaboom 7");
    }

    #[tokio::test]
    async fn into_response_is_500_with_generic_body() {
        let resp = MyError::from("secret detail leaked here").into_response();

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

        // The public body must NOT leak the internal error string.
        let (parts, body) = resp.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(text, "Something went wrong :)");
        assert!(!text.contains("secret detail leaked here"));

        // ...but the internal detail is stashed in the LoggedError extension so
        // the postgres-logging middleware can record it.
        let logged = parts
            .extensions
            .get::<LoggedError>()
            .expect("LoggedError extension must be present on error responses");
        assert!(logged.0.contains("secret detail leaked here"));
    }

    #[test]
    fn ensure_macro_ok_and_err() {
        fn check(pass: bool) -> anyhow::Result<()> {
            crate::ensure!(pass);
            Ok(())
        }
        assert!(check(true).is_ok());
        let err = check(false).unwrap_err();
        // The failing condition text is embedded in the error message.
        assert!(err.to_string().contains("pass"));
    }
}
