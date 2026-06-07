use std::fmt::Display;

use askama::Template;
use axum::{
    Router,
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, request::Parts},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
//use sqlx::postgres::PgPoolOptions;
use tower_http::services::ServeDir;

mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    //let database_url = std::env::var("DATABASE_URL").unwrap();

    //let pool = PgPoolOptions::new()
    //    .max_connections(5)
    //    .connect(&database_url)
    //    .await?;

    //let app = get_app_router().await.layer(axum::Extension(pool));
    let app = get_app_router().await;

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn get_app_router() -> Router {
    return Router::new()
        .route("/status_check", get(status_check))
        .route("/", get(home))
        .route("/business", get(business))
        .route("/contact", get(contact))
        .route("/financing", get(financing))
        .route("/api/toggle_theme", post(api::toggle_theme))
        .route("/calculator", get(calculator))
        .route("/api/calculate-savings", post(api::calculate_savings))
        .route("/savings-report", get(savings_report_page))
        .route("/api/savings-report", post(api::savings_report))
        .nest_service("/public", ServeDir::new("public"));
}

async fn status_check() -> &'static str {
    "running"
}

#[derive(Template)]
#[template(path = "index.html")]
struct HomeTemplate {
    theme: Theme,
}

#[derive(Template)]
#[template(path = "index.html", block = "content")]
struct HomeContent;
#[derive(Template)]
#[template(path = "business.html", block = "content")]
struct BusinessContent;
#[derive(Template)]
#[template(path = "financing.html", block = "content")]
struct FinancingContent;
#[derive(Template)]
#[template(path = "contact.html", block = "content")]
struct ContactContent;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    #[default]
    Dark,
    Light,
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        };

        return write!(f, "{}", val);
    }
}

async fn home(UserTheme(theme): UserTheme, headers: HeaderMap) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(HomeContent.render().unwrap())
    } else {
        Html(
            HomeTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

#[derive(Template)]
#[template(path = "business.html")]
struct BusinessTemplate {
    theme: Theme,
}

#[derive(Template)]
#[template(path = "financing.html")]
struct FinancingTemplate {
    theme: Theme,
}

#[derive(Template)]
#[template(path = "contact.html")]
struct ContactTemplate {
    theme: Theme,
}

async fn business(headers: HeaderMap, UserTheme(theme): UserTheme) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(BusinessContent.render().unwrap())
    } else {
        Html(
            BusinessTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

async fn financing(headers: HeaderMap, UserTheme(theme): UserTheme) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(FinancingContent.render().unwrap())
    } else {
        Html(
            FinancingTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

async fn contact(headers: HeaderMap, UserTheme(theme): UserTheme) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(ContactContent.render().unwrap())
    } else {
        Html(
            ContactTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

#[derive(Template)]
#[template(path = "calculator.html")]
struct CalculatorTemplate {
    theme: Theme,
}

#[derive(Template)]
#[template(path = "calculator.html", block = "content")]
struct CalculatorContent;

async fn calculator(headers: HeaderMap, UserTheme(theme): UserTheme) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(CalculatorContent.render().unwrap())
    } else {
        Html(
            CalculatorTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

#[derive(Template)]
#[template(path = "savings_report.html")]
struct SavingsReportTemplate {
    theme: Theme,
}

#[derive(Template)]
#[template(path = "savings_report.html", block = "content")]
struct SavingsReportContent;

async fn savings_report_page(headers: HeaderMap, UserTheme(theme): UserTheme) -> impl IntoResponse {
    if headers.contains_key("HX-Request") {
        Html(SavingsReportContent.render().unwrap())
    } else {
        Html(
            SavingsReportTemplate {
                theme: theme.unwrap_or_default(),
            }
            .render()
            .unwrap(),
        )
    }
}

#[derive(Debug, Default)]
pub struct UserTheme(pub Option<Theme>);

pub const COOKIE_THEME: &'static str = "cookie_theme";

impl<S> FromRequestParts<S> for UserTheme
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let cookies = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let Some(theme_cookie) = cookies.get(COOKIE_THEME) else {
            return Ok(Default::default());
        };

        match theme_cookie.value() {
            "dark" => Ok(UserTheme(Some(Theme::Dark))),
            "light" => Ok(UserTheme(Some(Theme::Light))),
            _ => Ok(Default::default()),
        }
    }
}
