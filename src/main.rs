use gameplay::Gameplay;
use macroquad::{conf::Conf, miniquad, window::next_frame};

mod game;
mod gnome;
mod grid;
mod job;
mod tile;
mod tileset;
mod context;
mod draw;
mod gameplay;
mod toolbar;
mod block;

const PKG_NAME: &str = "gnome-io";

fn window_conf() -> Conf {
    Conf {
        miniquad_conf: miniquad::conf::Conf {
            fullscreen: false,
            // high-dpi seems to change the zoom on webassembly??
            high_dpi: true,
            // icon: Some(Icon {
            //     small: include_bytes!("../icons/16x16.rgba").to_owned(),
            //     medium: include_bytes!("../icons/32x32.rgba").to_owned(),
            //     big: include_bytes!("../icons/64x64.rgba").to_owned(),
            // }),
            // platform: miniquad::conf::Platform {
            //     linux_backend: miniquad::conf::LinuxBackend::WaylandWithX11Fallback,
            //     ..Default::default()
            // },
            window_height: 720,
            window_resizable: true,
            window_title: String::from(PKG_NAME),
            window_width: 1280,
            ..Default::default()
        },
        default_filter_mode: miniquad::FilterMode::Nearest,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    #[cfg(target_arch = "wasm32")]
    sapp_console_log::init().unwrap();
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Initialize logging, and log the "info" level for this crate only, unless
        // the environment contains `RUST_LOG`.
        let env = env_logger::Env::new().default_filter_or("info");
        env_logger::Builder::from_env(env)
            .format_module_path(false)
            .format_source_path(true)
            .format_timestamp(None)
            .format_target(false)
            .init();
    }

    let mut ctx = context::Context::new().await;

    let mut g = Gameplay::new(&mut ctx);

    log::info!("Finished init");

    loop {
        ctx.update();
        g.update(&mut ctx);
        g.draw(&mut ctx);
        next_frame().await;
    }
}
