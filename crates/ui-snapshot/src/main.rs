//! Development tool: renders the freakymate UI headless (software renderer, no window, no GPU)
//! against a running (fake) Moonraker and writes one PNG per view.
//!
//! `cargo run -p ui-snapshot -- [--moonraker <url>] [--out <folder>] [--size 800x480] [--no-fit] [--scenes pages,network|all]`

mod scenes;
mod session;

use scenes::Scene;
use session::Session;
use std::path::PathBuf;
use std::process::ExitCode;

struct Options {
    moonraker: String,
    out: PathBuf,
    width: u32,
    height: u32,
    /// Scales the layout to the size like the app does; off shows the plain window of that size.
    fit: bool,
    scenes: Vec<Scene>,
}

fn parse_options() -> Result<Options, String> {
    let mut options = Options {
        moonraker: "http://127.0.0.1:7125".to_owned(),
        out: std::env::temp_dir().join("kt-ui-snapshots"),
        width: 800,
        height: 480,
        fit: true,
        scenes: Scene::DEFAULT.to_vec(),
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        if flag == "--no-fit" {
            options.fit = false;
            continue;
        }
        let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--moonraker" => options.moonraker = value,
            "--out" => options.out = PathBuf::from(value),
            "--size" => {
                let (width, height) = value.split_once('x').ok_or("size must be WIDTHxHEIGHT")?;
                options.width = width.parse().map_err(|_| "invalid width")?;
                options.height = height.parse().map_err(|_| "invalid height")?;
            }
            "--scenes" => options.scenes = Scene::parse_list(&value)?,
            _ => return Err(format!("unknown option {flag}")),
        }
    }
    Ok(options)
}

fn run() -> Result<(), String> {
    let options = parse_options()?;
    let scenes = options.scenes.clone();
    let mut session = Session::start(options)?;
    scenes::run(&mut session, &scenes)
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("ui-snapshot: {error}");
            ExitCode::FAILURE
        }
    }
}
