use base64::{engine::general_purpose, Engine as _};
use leptos::*;
use leptos_router::*;
use qrcode::render::svg;
use qrcode::QrCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
        Some(Ok(data)) => view! { cx,  <div inner_html=data/>  }.into_view(cx),
    };

    view! { cx,
        <h1>"Your time claim:"</h1>
        {render}
    }
}

#[component]
fn Validate(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let payload = move || params.with(|params| params.get("claim").cloned().unwrap_or_default());
    let claim = move || {
        let claim_decoded = general_purpose::URL_SAFE
            .decode(payload().as_bytes())
            .unwrap();
        let claim_str = std::str::from_utf8(&claim_decoded).unwrap().to_string();
        let claim: TimeClaim = serde_json::from_str(&claim_str).unwrap();
        claim.to_owned()
    };
    let claim_pretty = move || {
        let claim = claim();
        serde_json::to_string_pretty(&claim).unwrap()
    };
    view! { cx,
        <h1>"Validate your time claim"</h1>
        <p>"Payload: "{claim_pretty}</p>
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
    #[error("qr produce error")]
    Qr,
}

async fn make_claim_qr() -> Result<String, TimeClaimError> {
    let claim = make_claim().await?;
    let claim_str = claim.ser().map_err(|_| TimeClaimError::SerializeError)?;
    let claim_b64: String = general_purpose::URL_SAFE.encode(claim_str.as_bytes());
    let url = format!(
        "https://webhook.site/deca9549-d4d8-4ddd-9af7-46614ea86383?claim={}",
        claim_b64
    );

    let code = QrCode::new(url).map_err(|_| TimeClaimError::Qr)?;
    let image: String = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok(image)
}

async fn make_claim() -> Result<TimeClaim, TimeClaimError> {
    let btc_block = latest_btc_block().await?;
    Ok(TimeClaim {
        timestamp: btc_block.timestamp,
        evidence: Evidence::BtcBlockHash(btc_block.hash),
    })
}

async fn latest_btc_block() -> Result<BtcBlock, TimeClaimError> {
    let res: ChainApiBlock =
        reqwasm::http::Request::get("https://chain.api.btc.com/v3/block/latest")
            .send()
            .await
            .map_err(|_| TimeClaimError::ChainApiError)?
            .json()
            .await
            .map_err(|_| TimeClaimError::DeserializeError)?;
    Ok(res.data)
}
