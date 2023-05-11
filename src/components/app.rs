use leptos::*;
use leptos_router::*;

use crate::components::new_claim::{NewClaim, NewClaimProps};
use crate::components::validate::{Validate, ValidateProps};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
      <Router>
        <nav> </nav>
        <main>
          <Routes>
            <Route path="timeclaim/validate/:claim" view=|cx| view! { cx, <Validate/> }/>
            <Route path="/*any" view=|cx| view! { cx, <NewClaim/> }/>
          </Routes>
        </main>
      </Router>
    }
}
