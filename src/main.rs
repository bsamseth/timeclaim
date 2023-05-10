mod btc;
mod claim;
mod components;
mod error;
mod qr;

use components::app::{App, AppProps};
use leptos::{mount_to_body, view};

fn main() {
    mount_to_body(|cx| view! { cx, <App/> })
}
