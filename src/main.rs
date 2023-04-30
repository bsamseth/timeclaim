use base64::{engine::general_purpose, Engine as _};
use chrono::prelude::*;
use leptos::*;
use leptos_router::*;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use web_sys::window;

fn main() {
    leptos::mount_to_body(|cx| view! { cx, <App/> })
}

#[component]
fn App(cx: Scope) -> impl IntoView {
    view! { cx,
      <Router>
        <nav> </nav>
        <main>
          <Routes>
            <Route path="/validate/:claim" view=|cx| view! { cx, <Validate/> }/>
            <Route path="/*any" view=|cx| view! { cx, <Qr/> }/>
          </Routes>
        </main>
      </Router>
    }
}

#[component]
fn Qr(cx: Scope) -> impl IntoView {
    let claim_qr = create_local_resource(cx, || (), |_| async move { make_claim_qr().await });
    let render = move || match claim_qr.read(cx) {
        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
        Some(Err(e)) => view! { cx, <p>"Error: "{e.to_string()}</p> }.into_view(cx),
        Some(Ok((url, data))) => {
            view! { cx,  <A href={url}><div inner_html=data/></A>  }.into_view(cx)
        }
    };

    view! { cx,
        <h1>"Your time claim:"</h1>
        {render}
    }
}

async fn validate_payload(payload: String) -> Result<(u64, bool), TimeClaimError> {
    let decoded = general_purpose::URL_SAFE
        .decode(payload.as_bytes())
        .map_err(|_| TimeClaimError::BadPayload)?;
    let claim_str = std::str::from_utf8(&decoded).map_err(|_| TimeClaimError::BadPayload)?;
    let claim: TimeClaim =
        serde_json::from_str(&claim_str).map_err(|_| TimeClaimError::BadPayload)?;
    let validated = validate_claim(&claim).await?;
    Ok((claim.timestamp, validated))
}

#[component]
fn Validate(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let payload = move || params.with(|params| params.get("claim").cloned().unwrap_or_default());

    let validated =
        create_local_resource(cx, payload, |p| async move { validate_payload(p).await });

    let rendered = move || match validated.read(cx) {
        None => "Validating...".to_string(),
        Some(Err(e)) => e.to_string(),
        Some(Ok((timestamp, true))) => format!(
            "Valid time claim. This QR code was made no earlier than {}",
            {
                let dt = DateTime::<Utc>::from_utc(
                    NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
                        .expect("timestamp to be valid as it was just validated"),
                    Utc,
                )
                .with_timezone(&Local);
                dt.format("%Y-%m-%d %H:%M:%S").to_string()
            }
        ),
        Some(Ok((_, false))) => {
            format!("Invalid time claim. Do not trust the source of this QR code.")
        }
    };
    view! { cx,
        <h1>"Validate your time claim"</h1>
        <p>{rendered}</p>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TimeClaim {
    timestamp: u64,
    evidence: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Evidence {
    BtcBlockHash(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BtcBlock {
    hash: String,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChainApiBlock {
    data: BtcBlock,
}

#[derive(Error, Debug, Clone, Deserialize)]
pub enum TimeClaimError {
    #[error("error accessing chain.api.btc.com")]
    ChainApiError,
    #[error("serialize error")]
    SerializeError,
    #[error("deserialize error")]
    DeserializeError,
    #[error("bad validation payload")]
    BadPayload,
    #[error("qr produce error")]
    Qr,
}

async fn make_claim_qr() -> Result<(String, String), TimeClaimError> {
    let claim = make_claim().await?;
    let claim_str = claim.ser().map_err(|_| TimeClaimError::SerializeError)?;
    let claim_b64: String = general_purpose::URL_SAFE.encode(claim_str.as_bytes());
    let path = format!("/validate/{claim_b64}");
    let origin = window()
        .expect("window to be available")
        .location()
        .origin()
        .expect("origin to be available");
    let url = format!("{origin}{path}");
    let code = QrCode::new(&url).map_err(|_| TimeClaimError::Qr)?;
    let image: String = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok((path, image))
}

async fn make_claim() -> Result<TimeClaim, TimeClaimError> {
    let btc_block = latest_btc_block().await?;
    Ok(TimeClaim {
        timestamp: btc_block.timestamp,
        evidence: Evidence::BtcBlockHash(btc_block.hash),
    })
}

async fn latest_btc_block() -> Result<BtcBlock, TimeClaimError> {
    btc_block_by_hash("latest").await
}

async fn btc_block_by_hash(hash: &str) -> Result<BtcBlock, TimeClaimError> {
    let res: ChainApiBlock =
        reqwasm::http::Request::get(&format!("https://chain.api.btc.com/v3/block/{}", hash))
            .send()
            .await
            .map_err(|_| TimeClaimError::ChainApiError)?
            .json()
            .await
            .map_err(|_| TimeClaimError::DeserializeError)?;
    Ok(res.data)
}

async fn validate_claim(claim: &TimeClaim) -> Result<bool, TimeClaimError> {
    match &claim.evidence {
        Evidence::BtcBlockHash(bloch_hash) => {
            let block = btc_block_by_hash(&bloch_hash).await?;
            Ok(block.timestamp == claim.timestamp)
        }
    }
}
