use leptos::html::base;
use leptos::*;

use crate::claim::TimeClaim;
use crate::error::TimeClaimError;
use crate::qr;

#[component]
pub fn NewClaim(cx: Scope) -> impl IntoView {
    let base_url = move || {
        base(cx)
            .base_uri()
            .expect("base uri to be ok")
            .expect("base uri to exist")
    };
    let claim_qr = create_local_resource(cx, base_url, |base_url| async move {
        let claim = TimeClaim::new().await?;
        let url = format!("{}/validate/{}", base_url, claim.as_b64());
        let qr = qr::make_qr(&url)?;
        Ok::<_, TimeClaimError>((url, qr))
    });
    let render = move || match claim_qr.read(cx) {
        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
        Some(Err(e)) => view! { cx, <p>"Error: "{e.to_string()}</p> }.into_view(cx),
        Some(Ok((url, claim_qr))) => view! {
            cx,
            <div inner_html=claim_qr/>
            <a href={url}>"Click here to see what scanning the QR code looks like."</a>
        }
        .into_view(cx),
    };

    view! { cx,
        <h1>"Your time claim:"</h1>
        {render}
    }
}
