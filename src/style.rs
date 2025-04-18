use crate::Message;
use iced::{
    Alignment, Color, Element, Font, Length,
    widget::{container, row, text},
    window::{Icon, icon::from_rgba},
};

/// Function to style text based on amperage levels with an indicator
pub(crate) fn styled_text(pin: usize, value: f32) -> Element<'static, Message> {
    let color = if value < 6.0 {
        Color::from_rgb(0.4, 0.8, 0.5) // Softer, mint-like green
    } else if value < 9.0 {
        Color::from_rgb(0.95, 0.77, 0.38) // Amber/gold instead of harsh yellow
    } else {
        Color::from_rgb(0.9, 0.2, 0.2) // Red unchanged
    };

    // Create row with fixed-width containers
    row![
        // Pin label in fixed-width container
        container(
            text(format!("• Pin {}: ", pin))
                .size(24)
                .font(Font::with_name("Inter 24pt"))
        )
        .width(Length::Fixed(85.0))
        .align_x(Alignment::Start),
        // Amperage value in fixed-width container
        container(
            text(format!("{:.2} A", value))
                .size(24)
                .font(Font::with_name("Inter 24pt"))
                .color(color)
        )
        .width(Length::Fixed(90.0))
        .align_x(Alignment::Start)
    ]
    .spacing(2)
    .into()
}

pub(crate) fn build_icon() -> Result<Icon, anyhow::Error> {
    // Simple lightning bolt with star icon
    // Colors: Purple background with yellow/orange lightning
    let size = 32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    // Define colors
    let bg = [45, 10, 80, 0]; // Dark purple with transparency
    let star = [255, 240, 120, 255]; // Bright yellow
    let bolt = [255, 160, 20, 255]; // Orange-yellow
    let highlight = [230, 230, 255, 255]; // Bright highlight

    // Fill with transparent background
    for _ in 0..size * size {
        rgba.extend_from_slice(&bg);
    }

    // Draw lightning bolt
    for y in 0..size {
        for x in 0..size {
            let idx = ((y * size + x) * 4) as usize;

            // Lightning bolt shape
            if (x == 16 && (6..=12).contains(&y)) ||  // Top vertical
               ((12..=20).contains(&x) && y == 12) ||  // Upper horizontal
               (x == 12 && (12..=18).contains(&y)) ||  // Middle vertical
               ((8..=16).contains(&x) && y == 18) ||   // Lower horizontal
               (x == 8 && (18..=26).contains(&y))
            {
                // Bottom vertical
                rgba[idx..idx + 4].copy_from_slice(&bolt);
            }

            // Add highlight
            if (x == 15 && (7..=11).contains(&y)) ||  // Highlight on top vertical
               ((13..=19).contains(&x) && y == 11)
            {
                // Highlight on upper horizontal
                rgba[idx..idx + 4].copy_from_slice(&highlight);
            }

            // Star at the top
            if (x == 16 && y == 4)
                || (x == 15 && y == 5)
                || (x == 17 && y == 5)
                || ((14..=18).contains(&x) && y == 6)
            {
                rgba[idx..idx + 4].copy_from_slice(&star);
            }
        }
    }

    Ok(from_rgba(rgba, size, size)?)
}
