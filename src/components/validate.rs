use chrono::prelude::*;
use leptos::*;
use leptos_router::*;

use crate::claim::{TimeClaim, UNVERIFIED};

#[component]
pub fn Validate(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let payload = move || params.with(|params| params.get("claim").cloned().unwrap_or_default());
    let claim = create_local_resource(cx, payload, |payload| async move {
        let claim = payload.parse::<TimeClaim<UNVERIFIED>>()?;
        claim.validate().await
    });

    let rendered = move || match claim.read(cx) {
        None => "Validating...".to_string(),
        Some(Err(crate::error::TimeClaimError::InvalidClaim)) => {
            format!("Invalid time claim. Do not trust the source of this QR code.")
        }
        Some(Err(e)) => e.to_string(),
        Some(Ok(claim)) => format!(
            "Valid time claim. This QR code was made no earlier than {}",
            {
                let dt = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(claim.timestamp, 0)
                        .expect("timestamp to be valid as it was just validated"),
                    Utc,
                )
                .with_timezone(&Local);
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            }
        ),
    };
    view! { cx,
        <h1>"Validate your time claim"</h1>
        <p>{rendered}</p>
    }
}
