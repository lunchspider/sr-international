use crate::{COOKIE_THEME, Theme, UserTheme};
use askama::Template;
use axum::{
    Form,
    http::{HeaderMap, header},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use serde::Deserialize;

pub async fn toggle_theme(UserTheme(theme): UserTheme, headers: HeaderMap) -> impl IntoResponse {
    let theme = match theme.unwrap_or_default() {
        Theme::Dark => Theme::Light,
        Theme::Light => Theme::Dark,
    };

    let cookie = Cookie::build((COOKIE_THEME, theme.to_string()))
        .path("/")
        .build();

    let jar = CookieJar::from_headers(&headers);

    let jar = jar.add(cookie);

    let referer = headers.get(header::REFERER);
    let path = match referer {
        Some(referer) => referer.to_str().expect("Failed to get referer path"),
        None => "/",
    };

    (jar, Redirect::to(path))
}

#[derive(Deserialize)]
pub struct SavingsInput {
    monthly_bill: f64,
    property_type: String,
}

#[derive(Template)]
#[template(path = "partials/savings_result.html")]
pub struct SavingsResult {
    system_size: String,
    monthly_savings: String,
    annual_savings: String,
    lifetime_savings: String,
    payback: String,
    co2: String,
    system_cost: String,
    subsidy: String,
    net_cost: String,
    monthly_bill: String,
}

// Shared savings math used by both the calculator and the lead report.
fn compute_savings(monthly_bill: f64, commercial: bool) -> SavingsResult {
    // ── assumptions: tweak these freely ──
    let tariff = if commercial { 9.5 } else { 8.0 }; // ₹ per unit (kWh)
    let cost_per_kw = if commercial { 50_000.0 } else { 60_000.0 }; // ₹ installed per kW
    let sun_hours = 4.2; // avg daily generation hours
    let offset = 0.90; // share of the bill solar offsets
    let co2_per_kwh = 0.71; // kg CO₂ per grid kWh

    let bill = monthly_bill.max(500.0);

    // size the system from consumption
    let daily_units = (bill / tariff) / 30.0;
    let system_kw = (daily_units / sun_hours).clamp(1.0, 500.0);
    let system_cost = system_kw * cost_per_kw;

    // PM Surya Ghar subsidy (residential only): ₹30k/kW for first 2kW, ₹18k for the 3rd, cap ₹78k
    let subsidy = if commercial {
        0.0
    } else {
        (30_000.0 * system_kw.min(2.0) + 18_000.0 * (system_kw - 2.0).clamp(0.0, 1.0)).min(78_000.0)
    };
    let net_cost = (system_cost - subsidy).max(0.0);

    let monthly_savings = bill * offset;
    let annual_savings = monthly_savings * 12.0;
    let lifetime_savings = (annual_savings * 25.0 - net_cost).max(0.0); // 25-yr net
    let payback_years = if annual_savings > 0.0 {
        net_cost / annual_savings
    } else {
        0.0
    };
    let co2_per_year = system_kw * sun_hours * 365.0 * co2_per_kwh / 1000.0; // tonnes/yr

    SavingsResult {
        system_size: format!("{:.1} kW", system_kw),
        monthly_savings: rupees(monthly_savings),
        annual_savings: rupees(annual_savings),
        lifetime_savings: lakhs(lifetime_savings),
        payback: format!("{:.1} years", payback_years),
        co2: format!("{:.1} t/yr", co2_per_year),
        system_cost: rupees(system_cost),
        subsidy: rupees(subsidy),
        net_cost: rupees(net_cost),
        monthly_bill: rupees(bill),
    }
}

pub async fn calculate_savings(Form(input): Form<SavingsInput>) -> impl IntoResponse {
    let result = compute_savings(input.monthly_bill, input.property_type == "commercial");
    Html(result.render().unwrap())
}

#[derive(Deserialize)]
pub struct ReportInput {
    name: String,
    email: String,
    phone: String,
    monthly_bill: f64,
    property_type: String,
}

#[derive(Template)]
#[template(path = "partials/report_sent.html")]
pub struct ReportSent {
    name: String,
    email: String,
    system_size: String,
    monthly_savings: String,
    annual_savings: String,
    lifetime_savings: String,
    payback: String,
    co2: String,
    system_cost: String,
    subsidy: String,
    net_cost: String,
    monthly_bill: String,
}

pub async fn savings_report(Form(input): Form<ReportInput>) -> impl IntoResponse {
    // TODO: persist the lead / send the report email here (CRM, DB, mailer, …).
    // For now we just log it so the captured contact details aren't silently dropped.
    println!(
        "New savings-report lead: {} <{}> {}",
        input.name, input.email, input.phone
    );

    let s = compute_savings(input.monthly_bill, input.property_type == "commercial");
    let view = ReportSent {
        name: input.name,
        email: input.email,
        system_size: s.system_size,
        monthly_savings: s.monthly_savings,
        annual_savings: s.annual_savings,
        lifetime_savings: s.lifetime_savings,
        payback: s.payback,
        co2: s.co2,
        system_cost: s.system_cost,
        subsidy: s.subsidy,
        net_cost: s.net_cost,
        monthly_bill: s.monthly_bill,
    };
    Html(view.render().unwrap())
}

// "₹3,12,000" — Indian digit grouping (last 3, then pairs)
fn rupees(n: f64) -> String {
    let n = n.round() as i64;
    let s = n.abs().to_string();
    let len = s.len();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        let from_right = len - i;
        if i > 0 && (from_right == 3 || (from_right > 3 && from_right % 2 == 1)) {
            out.push(',');
        }
        out.push(c);
    }
    format!("₹{}{}", if n < 0 { "-" } else { "" }, out)
}

// "₹13.5 L" / "₹1.2 Cr" for large figures
fn lakhs(n: f64) -> String {
    let n = n.max(0.0);
    if n >= 1e7 {
        format!("₹{:.1} Cr", n / 1e7)
    } else if n >= 1e5 {
        format!("₹{:.1} L", n / 1e5)
    } else {
        rupees(n)
    }
}
