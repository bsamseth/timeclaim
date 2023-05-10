use qrcode::render::svg;
use qrcode::QrCode;

use crate::error::{Result, TimeClaimError};

pub fn make_qr(data: &str) -> Result<String> {
    let code = QrCode::new(data).map_err(|_| TimeClaimError::Qr)?;
    let image: String = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(image)
}
