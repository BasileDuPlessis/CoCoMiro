use headless_chrome::{
    Browser, LaunchOptionsBuilder, Tab,
    browser::tab::{element::Element, point::Point},
    protocol::cdp::Input,
};
use std::{
    env,
    error::Error,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const HOST: &str = "127.0.0.1";
const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
const SERVER_POLL_INTERVAL: Duration = Duration::from_millis(50);
const PAN_UPDATE_TIMEOUT: Duration = Duration::from_secs(2);
const PAN_POLL_INTERVAL: Duration = Duration::from_millis(25);
const TRUNK_START_ATTEMPTS: usize = 5;
const DRAG_STEPS: usize = 6;
const DRAG_DISTANCE_X: f64 = 140.0;
const DRAG_DISTANCE_Y: f64 = 90.0;
const MIN_EXPECTED_PAN_X_DELTA: f64 = 80.0;
const MIN_EXPECTED_PAN_Y_DELTA: f64 = 50.0;

struct ChildGuard(Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn reserve_port() -> Result<u16, Box<dyn Error>> {
    let listener = TcpListener::bind((HOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn wait_for_server(
    address: &str,
    child: &mut Child,
    timeout: Duration,
) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("server exited early with status {status}").into());
        }

        if TcpStream::connect(address).is_ok() {
            return Ok(());
        }

        thread::sleep(SERVER_POLL_INTERVAL);
    }

    Err(format!("timed out waiting for {address}").into())
}

fn find_on_path(names: &[&str]) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    for directory in env::split_paths(&path_var) {
        for name in names {
            let candidate = directory.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    None
}

fn chrome_binary() -> Option<PathBuf> {
    if let Some(configured_path) = env::var_os("CHROME_BIN") {
        let path = PathBuf::from(configured_path);
        if path.exists() {
            return Some(path);
        }
    }

    let candidate_paths = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
        "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    ];

    candidate_paths
        .into_iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .or_else(|| {
            find_on_path(&[
                "google-chrome",
                "google-chrome-stable",
                "chromium",
                "chromium-browser",
                "chrome",
                "chrome.exe",
            ])
        })
}

fn wait_for_canvas_ready(tab: &Tab, timeout: Duration) -> Result<(), Box<dyn Error>> {
    // Wait for the app's own ready marker instead of relying on a fixed sleep.
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if tab.find_element("#infinite-canvas[data-ready=\"true\"]").is_ok() {
            return Ok(());
        }

        thread::sleep(SERVER_POLL_INTERVAL);
    }

    Err("timed out waiting for the canvas app to become interactive".into())
}

fn spawn_trunk() -> Result<(ChildGuard, String), Box<dyn Error>> {
    let mut last_error: Option<Box<dyn Error>> = None;

    for attempt in 1..=TRUNK_START_ATTEMPTS {
        let port = reserve_port()?;
        let address = format!("{HOST}:{port}");
        let mut trunk = Command::new("trunk")
            .args([
                "serve",
                "--address",
                HOST,
                "--port",
                &port.to_string(),
                "--no-autoreload",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        match wait_for_server(&address, &mut trunk, STARTUP_TIMEOUT) {
            Ok(()) => return Ok((ChildGuard(trunk), format!("http://{address}/"))),
            Err(error) => {
                last_error = Some(
                    format!("attempt {attempt} failed to start the Trunk server on {address}: {error}")
                        .into(),
                );
                let _ = trunk.kill();
                let _ = trunk.wait();
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "failed to start Trunk after multiple attempts".into()))
}

fn open_home_page() -> Result<(ChildGuard, Browser, Arc<Tab>), Box<dyn Error>> {
    let (trunk_guard, url) = spawn_trunk()?;

    let launch_options = LaunchOptionsBuilder::default()
        .path(chrome_binary())
        .headless(true)
        .build()
        .map_err(|message| format!("failed to build Chrome launch options: {message}"))?;

    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&url)?;
    wait_for_canvas_ready(tab.as_ref(), STARTUP_TIMEOUT)?;

    Ok((trunk_guard, browser, tab))
}

fn attribute_as_f64(element: &Element<'_>, name: &str) -> Result<f64, Box<dyn Error>> {
    let value = element
        .get_attribute_value(name)?
        .ok_or_else(|| format!("missing attribute {name}"))?;

    Ok(value.parse::<f64>()?)
}

fn dispatch_mouse_event(
    tab: &Tab,
    event_type: Input::DispatchMouseEventTypeOption,
    point: Point,
    button: Option<Input::MouseButton>,
    buttons: Option<u32>,
) -> Result<(), Box<dyn Error>> {
    tab.call_method(Input::DispatchMouseEvent {
        Type: event_type,
        x: point.x,
        y: point.y,
        modifiers: None,
        timestamp: None,
        button,
        buttons,
        click_count: Some(1),
        force: None,
        tangential_pressure: None,
        tilt_x: None,
        tilt_y: None,
        twist: None,
        delta_x: None,
        delta_y: None,
        pointer_Type: None,
    })?;

    Ok(())
}

fn wait_for_pan_update(
    tab: &Tab,
    initial_pan_x: f64,
    initial_pan_y: f64,
    timeout: Duration,
) -> Result<(f64, f64), Box<dyn Error>> {
    // Poll the exported pan attributes so slower CI machines do not race the render loop.
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        let canvas = tab.wait_for_element("#infinite-canvas[data-ready=\"true\"]")?;
        let final_pan_x = attribute_as_f64(&canvas, "data-pan-x")?;
        let final_pan_y = attribute_as_f64(&canvas, "data-pan-y")?;

        if (final_pan_x - initial_pan_x).abs() > 1.0 || (final_pan_y - initial_pan_y).abs() > 1.0 {
            return Ok((final_pan_x, final_pan_y));
        }

        thread::sleep(PAN_POLL_INTERVAL);
    }

    Err("timed out waiting for drag pan coordinates to update".into())
}

fn assert_dragging_canvas_updates_pan_coordinates(tab: &Tab) -> Result<(), Box<dyn Error>> {
    let canvas = tab.wait_for_element("#infinite-canvas[data-ready=\"true\"]")?;
    let initial_pan_x = attribute_as_f64(&canvas, "data-pan-x")?;
    let initial_pan_y = attribute_as_f64(&canvas, "data-pan-y")?;

    let bounds = canvas.get_box_model()?.margin_viewport();
    let start = Point {
        x: bounds.x + (bounds.width / 2.0),
        y: bounds.y + (bounds.height / 2.0),
    };
    let end = Point {
        x: start.x + DRAG_DISTANCE_X,
        y: start.y + DRAG_DISTANCE_Y,
    };

    tab.move_mouse_to_point(start)?;
    dispatch_mouse_event(
        tab,
        Input::DispatchMouseEventTypeOption::MousePressed,
        start,
        Some(Input::MouseButton::Left),
        Some(1),
    )?;

    for step in 1..=DRAG_STEPS {
        let progress = step as f64 / DRAG_STEPS as f64;
        let point = Point {
            x: start.x + (end.x - start.x) * progress,
            y: start.y + (end.y - start.y) * progress,
        };

        dispatch_mouse_event(
            tab,
            Input::DispatchMouseEventTypeOption::MouseMoved,
            point,
            Some(Input::MouseButton::Left),
            Some(1),
        )?;
    }

    dispatch_mouse_event(
        tab,
        Input::DispatchMouseEventTypeOption::MouseReleased,
        end,
        Some(Input::MouseButton::Left),
        Some(1),
    )?;

    let (final_pan_x, final_pan_y) =
        wait_for_pan_update(tab, initial_pan_x, initial_pan_y, PAN_UPDATE_TIMEOUT)?;

    assert!(final_pan_x - initial_pan_x > MIN_EXPECTED_PAN_X_DELTA);
    assert!(final_pan_y - initial_pan_y > MIN_EXPECTED_PAN_Y_DELTA);

    Ok(())
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_home -- --ignored`"]
fn home_page_supports_dragging_without_header_copy() -> Result<(), Box<dyn Error>> {
    let (_trunk_guard, _browser, tab) = open_home_page()?;
    let canvas = tab.wait_for_element("#infinite-canvas[data-ready=\"true\"]")?;

    assert!(tab.find_element("h1").is_err());
    assert!(tab.find_element(".subtitle").is_err());
    assert!(attribute_as_f64(&canvas, "data-pan-x")?.abs() < 0.01);
    assert!(attribute_as_f64(&canvas, "data-pan-y")?.abs() < 0.01);
    assert!((attribute_as_f64(&canvas, "data-zoom")? - 1.0).abs() < 0.01);

    assert_dragging_canvas_updates_pan_coordinates(tab.as_ref())?;

    Ok(())
}
