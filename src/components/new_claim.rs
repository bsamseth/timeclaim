use leptos::*;
use leptos_router::*;

use crate::claim::TimeClaim;
use crate::error::TimeClaimError;
use crate::qr;

#[component]
pub fn NewClaim(cx: Scope) -> impl IntoView {
    let claim_qr = create_local_resource(
        cx,
        || (),
        |_| async move {
            let claim = TimeClaim::new().await?;
            let origin = web_sys::window()
                .expect("window to be available")
                .location()
                .origin()
                .expect("origin to be available");
            let url = format!("{}/validate/{}", origin, claim.as_b64());
            let qr = qr::make_qr(&url)?;
            Ok::<_, TimeClaimError>((url, qr))
        },
    );
    let render = move || match claim_qr.read(cx) {
        None => view! { cx, <p>"Loading..."</p> }.into_view(cx),
        Some(Err(e)) => view! { cx, <p>"Error: "{e.to_string()}</p> }.into_view(cx),
        Some(Ok((url, claim_qr))) => view! {
            cx,
            <div inner_html=claim_qr/>
            <A href={url}>"Click here to see what scanning the QR code looks like."</A>
        }
        .into_view(cx),
    };

    view! { cx,
        <h1>"Your time claim:"</h1>
        {render}
    }
}
