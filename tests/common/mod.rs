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

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

pub const HOST: &str = "127.0.0.1";
pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);
pub const SERVER_POLL_INTERVAL: Duration = Duration::from_millis(50);
pub const PAN_UPDATE_TIMEOUT: Duration = Duration::from_secs(2);
pub const PAN_POLL_INTERVAL: Duration = Duration::from_millis(25);
pub const TRUNK_START_ATTEMPTS: usize = 5;
pub const DRAG_STEPS: usize = 6;
pub const DRAG_DISTANCE_X: f64 = 140.0;
pub const DRAG_DISTANCE_Y: f64 = 90.0;
pub const MIN_EXPECTED_PAN_X_DELTA: f64 = 80.0;
pub const CANVAS_SELECTOR: &str = "#infinite-canvas[data-ready=\"true\"]";
pub const TOOLBAR_SELECTOR: &str = "#floating-toolbar";
pub const DEFAULT_VIEW_TOLERANCE: f64 = 0.01;

pub struct ChildGuard(pub Child);

pub struct HomePageSession {
    pub _trunk_guard: ChildGuard,
    pub _browser: Browser,
    pub tab: Arc<Tab>,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

pub fn reserve_port() -> TestResult<u16> {
    let listener = TcpListener::bind((HOST, 0))?;
    Ok(listener.local_addr()?.port())
}

pub fn wait_for_server(address: &str, child: &mut Child, timeout: Duration) -> TestResult {
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

pub fn find_on_path(names: &[&str]) -> Option<PathBuf> {
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

pub fn chrome_binary() -> Option<PathBuf> {
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

pub fn wait_for_canvas_ready(tab: &Tab, timeout: Duration) -> TestResult {
    // Wait for the app's own ready marker instead of relying on a fixed sleep.
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if tab.find_element(CANVAS_SELECTOR).is_ok() {
            return Ok(());
        }

        thread::sleep(SERVER_POLL_INTERVAL);
    }

    Err("timed out waiting for the canvas app to become interactive".into())
}

pub fn spawn_trunk() -> TestResult<(ChildGuard, String)> {
    let mut last_error: Option<Box<dyn Error>> = None;

    for attempt in 1..=TRUNK_START_ATTEMPTS {
        let port = reserve_port()?;
        let address = format!("{HOST}:{port}");

        // Build the project first to ensure CSS is processed, but only if not already built
        let dist_dir = std::env::current_dir()?.join("dist");
        if !dist_dir.exists() || !dist_dir.join("index.html").exists() {
            let build_result = Command::new("trunk")
                .args(["build", "--release"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()?;

            if !build_result.success() {
                return Err(format!(
                    "trunk build failed with exit code {}",
                    build_result.code().unwrap_or(-1)
                )
                .into());
            }
        }

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
                    format!(
                        "attempt {attempt} failed to start the Trunk server on {address}: {error}"
                    )
                    .into(),
                );
                let _ = trunk.kill();
                let _ = trunk.wait();
            }
        }
    }

    Err(last_error.unwrap_or_else(|| "failed to start Trunk after multiple attempts".into()))
}

impl HomePageSession {
    pub fn launch() -> TestResult<Self> {
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

        Ok(Self {
            _trunk_guard: trunk_guard,
            _browser: browser,
            tab,
        })
    }

    pub fn tab(&self) -> &Tab {
        self.tab.as_ref()
    }
}

pub fn attribute_as_f64(element: &Element<'_>, name: &str) -> TestResult<f64> {
    let value = element
        .get_attribute_value(name)?
        .ok_or_else(|| format!("missing attribute {name}"))?;

    Ok(value.parse::<f64>()?)
}

pub fn ready_canvas(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(CANVAS_SELECTOR)?)
}

pub fn ready_toolbar(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(TOOLBAR_SELECTOR)?)
}

pub fn pan_coordinates(canvas: &Element<'_>) -> TestResult<(f64, f64)> {
    Ok((
        attribute_as_f64(canvas, "data-pan-x")?,
        attribute_as_f64(canvas, "data-pan-y")?,
    ))
}

pub fn assert_within_tolerance(label: &str, actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{label} expected {expected} ± {tolerance}, got {actual}"
    );
}

pub fn dispatch_mouse_event(
    tab: &Tab,
    event_type: Input::DispatchMouseEventTypeOption,
    point: Point,
    button: Option<Input::MouseButton>,
    buttons: Option<u32>,
) -> TestResult {
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

pub fn wait_for_pan_update(
    tab: &Tab,
    initial_pan_x: f64,
    initial_pan_y: f64,
    timeout: Duration,
) -> TestResult<(f64, f64)> {
    // Poll the exported pan attributes so slower CI machines do not race the render loop.
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        let canvas = ready_canvas(tab)?;
        let (final_pan_x, final_pan_y) = pan_coordinates(&canvas)?;

        if (final_pan_x - initial_pan_x).abs() > 1.0 || (final_pan_y - initial_pan_y).abs() > 1.0 {
            return Ok((final_pan_x, final_pan_y));
        }

        thread::sleep(PAN_POLL_INTERVAL);
    }

    Err("timed out waiting for drag pan coordinates to update".into())
}

pub fn drag_pointer(tab: &Tab, start: Point, end: Point) -> TestResult {
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

    Ok(())
}