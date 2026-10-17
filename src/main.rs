#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod generate;

use gpui_kit::prelude::*;
use gpui_kit::{Bounds, WindowBounds, WindowOptions, px, size};

fn main() {
    gpui_kit::application().run(|cx| {
        gpui_kit::init(cx);
        app::init_theme(cx);

        let bounds = Bounds::centered(None, size(px(480.), px(640.)), cx);
        gpui_kit::open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            cx,
            |_, cx| cx.new(app::SuimiApp::new),
        )
        .expect("打开窗口失败");
    });
}
