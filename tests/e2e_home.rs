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

type TestResult<T = ()> = Result<T, Box<dyn Error>>;

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
const MIN_EXPECTED_TOOLBAR_X_DELTA: f64 = 40.0;
const MIN_EXPECTED_TOOLBAR_Y_DELTA: f64 = 30.0;
const CANVAS_SELECTOR: &str = "#infinite-canvas[data-ready=\"true\"]";
const TOOLBAR_SELECTOR: &str = "#floating-toolbar";
const TOOLBAR_HANDLE_SELECTOR: &str = "#floating-toolbar-handle";
const DEFAULT_VIEW_TOLERANCE: f64 = 0.01;

struct ChildGuard(Child);

struct HomePageSession {
    _trunk_guard: ChildGuard,
    _browser: Browser,
    tab: Arc<Tab>,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn reserve_port() -> TestResult<u16> {
    let listener = TcpListener::bind((HOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn wait_for_server(address: &str, child: &mut Child, timeout: Duration) -> TestResult {
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

fn wait_for_canvas_ready(tab: &Tab, timeout: Duration) -> TestResult {
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

fn spawn_trunk() -> TestResult<(ChildGuard, String)> {
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
    fn launch() -> TestResult<Self> {
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

    fn tab(&self) -> &Tab {
        self.tab.as_ref()
    }

    fn assert_starts_clean(&self) -> TestResult {
        assert_home_page_starts_clean(self.tab())
    }

    fn assert_drag_pans_canvas(&self) -> TestResult {
        assert_dragging_canvas_updates_pan_coordinates(self.tab())
    }

    fn assert_toolbar_is_visible(&self) -> TestResult {
        assert_toolbar_is_visible(self.tab())
    }

    fn assert_toolbar_can_be_dragged(&self) -> TestResult {
        assert_dragging_toolbar_repositions_it(self.tab())
    }
}

fn attribute_as_f64(element: &Element<'_>, name: &str) -> TestResult<f64> {
    let value = element
        .get_attribute_value(name)?
        .ok_or_else(|| format!("missing attribute {name}"))?;

    Ok(value.parse::<f64>()?)
}

fn ready_canvas(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(CANVAS_SELECTOR)?)
}

fn ready_toolbar(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(TOOLBAR_SELECTOR)?)
}

fn ready_toolbar_handle(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(TOOLBAR_HANDLE_SELECTOR)?)
}

fn pan_coordinates(canvas: &Element<'_>) -> TestResult<(f64, f64)> {
    Ok((
        attribute_as_f64(canvas, "data-pan-x")?,
        attribute_as_f64(canvas, "data-pan-y")?,
    ))
}

fn assert_within_tolerance(label: &str, actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() < tolerance,
        "{label} expected {expected} ± {tolerance}, got {actual}"
    );
}

fn assert_home_page_starts_clean(tab: &Tab) -> TestResult {
    let canvas = ready_canvas(tab)?;
    let (pan_x, pan_y) = pan_coordinates(&canvas)?;

    assert!(
        tab.find_element("h1").is_err(),
        "unexpected header copy found"
    );
    assert!(
        tab.find_element(".subtitle").is_err(),
        "unexpected subtitle copy found"
    );
    assert_within_tolerance("data-pan-x", pan_x, 0.0, DEFAULT_VIEW_TOLERANCE);
    assert_within_tolerance("data-pan-y", pan_y, 0.0, DEFAULT_VIEW_TOLERANCE);
    assert_within_tolerance(
        "data-zoom",
        attribute_as_f64(&canvas, "data-zoom")?,
        1.0,
        DEFAULT_VIEW_TOLERANCE,
    );

    Ok(())
}

fn assert_toolbar_is_visible(tab: &Tab) -> TestResult {
    let toolbar = ready_toolbar(tab)?;
    let bounds = toolbar.get_box_model()?.margin_viewport();

    assert!(bounds.height > bounds.width, "expected a vertical toolbar");
    assert!(
        attribute_as_f64(&toolbar, "data-x")? >= 0.0,
        "toolbar x position should be exposed"
    );
    assert!(
        attribute_as_f64(&toolbar, "data-y")? >= 0.0,
        "toolbar y position should be exposed"
    );

    Ok(())
}

fn drag_start_and_end_points(canvas: &Element<'_>) -> TestResult<(Point, Point)> {
    let bounds = canvas.get_box_model()?.margin_viewport();
    let start = Point {
        x: bounds.x + (bounds.width / 2.0),
        y: bounds.y + (bounds.height / 2.0),
    };
    let end = Point {
        x: start.x + DRAG_DISTANCE_X,
        y: start.y + DRAG_DISTANCE_Y,
    };

    Ok((start, end))
}

fn dispatch_mouse_event(
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

fn wait_for_pan_update(
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

fn drag_pointer(tab: &Tab, start: Point, end: Point) -> TestResult {
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

fn assert_dragging_canvas_updates_pan_coordinates(tab: &Tab) -> TestResult {
    let canvas = ready_canvas(tab)?;
    let (initial_pan_x, initial_pan_y) = pan_coordinates(&canvas)?;
    let (start, end) = drag_start_and_end_points(&canvas)?;

    drag_pointer(tab, start, end)?;

    let (final_pan_x, final_pan_y) =
        wait_for_pan_update(tab, initial_pan_x, initial_pan_y, PAN_UPDATE_TIMEOUT)?;

    assert!(
        final_pan_x - initial_pan_x > MIN_EXPECTED_PAN_X_DELTA,
        "expected horizontal pan change to exceed {MIN_EXPECTED_PAN_X_DELTA}, got {}",
        final_pan_x - initial_pan_x
    );
    assert!(
        final_pan_y - initial_pan_y > MIN_EXPECTED_PAN_Y_DELTA,
        "expected vertical pan change to exceed {MIN_EXPECTED_PAN_Y_DELTA}, got {}",
        final_pan_y - initial_pan_y
    );

    Ok(())
}

fn assert_dragging_toolbar_repositions_it(tab: &Tab) -> TestResult {
    let toolbar = ready_toolbar(tab)?;
    let handle = ready_toolbar_handle(tab)?;
    let initial_x = attribute_as_f64(&toolbar, "data-x")?;
    let initial_y = attribute_as_f64(&toolbar, "data-y")?;
    let bounds = handle.get_box_model()?.margin_viewport();
    let start = Point {
        x: bounds.x + (bounds.width / 2.0),
        y: bounds.y + (bounds.height / 2.0),
    };
    let end = Point {
        x: start.x + 90.0,
        y: start.y + 65.0,
    };

    drag_pointer(tab, start, end)?;

    let toolbar = ready_toolbar(tab)?;
    let final_x = attribute_as_f64(&toolbar, "data-x")?;
    let final_y = attribute_as_f64(&toolbar, "data-y")?;

    assert!(
        final_x - initial_x > MIN_EXPECTED_TOOLBAR_X_DELTA,
        "expected toolbar x to move by more than {MIN_EXPECTED_TOOLBAR_X_DELTA}, got {}",
        final_x - initial_x
    );
    assert!(
        final_y - initial_y > MIN_EXPECTED_TOOLBAR_Y_DELTA,
        "expected toolbar y to move by more than {MIN_EXPECTED_TOOLBAR_Y_DELTA}, got {}",
        final_y - initial_y
    );

    Ok(())
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_home -- --ignored`"]
fn home_page_supports_dragging_without_header_copy() -> TestResult {
    let session = HomePageSession::launch()?;

    session.assert_starts_clean()?;
    session.assert_toolbar_is_visible()?;
    session.assert_drag_pans_canvas()?;

    Ok(())
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_home -- --ignored`"]
fn floating_toolbar_can_be_dragged_over_canvas() -> TestResult {
    let session = HomePageSession::launch()?;

    session.assert_starts_clean()?;
    session.assert_toolbar_is_visible()?;
    session.assert_toolbar_can_be_dragged()?;

    Ok(())
}
