/// how large to draw the text
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum Size {
    Small,
    Medium,
    Large,
}

use macroquad::color::Color;
use macroquad::text::{TextDimensions, TextParams, draw_text_ex};

use crate::context::Context;

/// draw the text to the screen, simpler API than Macroquad's with a default font and enum for size
pub fn draw_text(ctx: &Context, text: &str, x: f32, y: f32, size: Size, color: Color) {
    draw_text_ex(text, x, y, TextParams {
        font_size: text_size(size),
        font: Some(&ctx.font),
        color,
        ..Default::default()
    });
}

#[allow(unused)]
pub fn draw_text_screen_centered(ctx: &Context, text: &str, y: f32, size: Size, color: Color) {
    let text_size = measure_text(ctx, text, size);
    draw_text(
        ctx,
        text,
        (ctx.screen_size.x - text_size.width) / 2.,
        y,
        size,
        color,
    );
}

#[allow(unused)]
pub fn draw_text_centered_pos(ctx: &Context, text: &str, x: f32, y: f32, size: Size, color: Color) {
    let text_size = measure_text(ctx, text, size);
    draw_text(
        ctx,
        text,
        x - text_size.width / 2.,
        y - text_size.height / 2.,
        size,
        color,
    );
}

#[allow(unused)]
pub fn measure_text(ctx: &Context, text: &str, size: Size) -> TextDimensions {
    macroquad::text::measure_text(text, Some(&ctx.font), text_size(size), 1.0)
}

pub const fn text_size(size: Size) -> u16 {
    match size {
        Size::Small => 20u16,
        Size::Medium => 32u16,
        Size::Large => 48u16,
    }
}
