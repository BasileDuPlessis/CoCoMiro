use headless_chrome::{Browser, LaunchOptionsBuilder, Tab, protocol::cdp::Page};
use image::ImageFormat;
use base64;
use std::{
    env,
    error::Error,
    fs,
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
const TRUNK_START_ATTEMPTS: usize = 5;
const CANVAS_SELECTOR: &str = "#infinite-canvas[data-ready=\"true\"]";


struct ChildGuard(Child);

struct VisualRegressionSession {
    _trunk_guard: ChildGuard,
    _browser: Browser,
    tab: Arc<Tab>,
    baselines_dir: PathBuf,
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl VisualRegressionSession {
    fn launch() -> TestResult<Self> {
        let (trunk_guard, url) = spawn_trunk()?;
        let baselines_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("baselines");

        // Create baselines directory if it doesn't exist
        fs::create_dir_all(&baselines_dir)?;

        let launch_options = LaunchOptionsBuilder::default()
            .path(chrome_binary())
            .headless(true)
            .window_size(Some((1280, 720)))
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
            baselines_dir,
        })
    }

    fn tab(&self) -> &Tab {
        self.tab.as_ref()
    }

    fn capture_screenshot(&self, _name: &str) -> TestResult<Vec<u8>> {
        let screenshot_data = self.tab.call_method(Page::CaptureScreenshot {
            format: Some(Page::CaptureScreenshotFormatOption::Png),
            quality: None,
            clip: None,
            from_surface: Some(true),
            capture_beyond_viewport: Some(false),
            optimize_for_speed: Some(false),
        })?;

        // Data is base64 encoded string
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &screenshot_data.data)
            .map_err(|e| format!("Failed to decode base64 screenshot data: {}", e).into())
    }

    fn save_baseline(&self, name: &str, data: &[u8]) -> TestResult<PathBuf> {
        let path = self.baselines_dir.join(format!("{}.png", name));
        fs::write(&path, data)?;
        println!("Saved baseline: {}", path.display());
        Ok(path)
    }

    fn load_baseline(&self, name: &str) -> TestResult<Vec<u8>> {
        let path = self.baselines_dir.join(format!("{}.png", name));
        fs::read(&path)
            .map_err(|e| format!("Failed to load baseline {}: {}", path.display(), e).into())
    }

    fn compare_images(
        &self,
        baseline_data: &[u8],
        current_data: &[u8],
        threshold: f64,
    ) -> TestResult<bool> {
        let baseline_img = image::load_from_memory_with_format(baseline_data, ImageFormat::Png)?;
        let current_img = image::load_from_memory_with_format(current_data, ImageFormat::Png)?;

        // Convert to RGBA for comparison
        let baseline_rgba = baseline_img.to_rgba8();
        let current_rgba = current_img.to_rgba8();

        // Ensure images have the same dimensions
        if baseline_rgba.dimensions() != current_rgba.dimensions() {
            return Ok(false);
        }

        let (width, height) = baseline_rgba.dimensions();
        let total_pixels = (width * height) as usize;
        let mut different_pixels = 0;

        // Compare pixel by pixel
        for (baseline_pixel, current_pixel) in baseline_rgba.pixels().zip(current_rgba.pixels()) {
            let diff_r = (baseline_pixel[0] as i32 - current_pixel[0] as i32).abs();
            let diff_g = (baseline_pixel[1] as i32 - current_pixel[1] as i32).abs();
            let diff_b = (baseline_pixel[2] as i32 - current_pixel[2] as i32).abs();
            let diff_a = (baseline_pixel[3] as i32 - current_pixel[3] as i32).abs();

            // Consider pixels different if any channel differs significantly
            if diff_r > 5 || diff_g > 5 || diff_b > 5 || diff_a > 5 {
                different_pixels += 1;
            }
        }

        let diff_ratio = different_pixels as f64 / total_pixels as f64;
        println!(
            "Image comparison: {:.2}% pixels differ (threshold: {:.2}%)",
            diff_ratio * 100.0,
            threshold * 100.0
        );

        Ok(diff_ratio <= threshold)
    }

    fn assert_visual_match(&self, name: &str, threshold: f64) -> TestResult {
        let current_screenshot = self.capture_screenshot(name)?;

        match self.load_baseline(name) {
            Ok(baseline_data) => {
                if self.compare_images(&baseline_data, &current_screenshot, threshold)? {
                    println!("✅ Visual test '{}' passed", name);
                    Ok(())
                } else {
                    // Save the failing screenshot for debugging
                    let fail_path = self.baselines_dir.join(format!("{}_fail.png", name));
                    fs::write(&fail_path, &current_screenshot)?;
                    println!(
                        "❌ Visual test '{}' failed - saved failing screenshot to {}",
                        name,
                        fail_path.display()
                    );
                    Err(format!("Visual regression detected in '{}'", name).into())
                }
            }
            Err(_) => {
                // No baseline exists, create it
                self.save_baseline(name, &current_screenshot)?;
                println!("📸 Created new baseline for '{}'", name);
                Ok(())
            }
        }
    }

    fn update_baseline(&self, name: &str) -> TestResult {
        let screenshot = self.capture_screenshot(name)?;
        self.save_baseline(name, &screenshot)?;
        println!("🔄 Updated baseline for '{}'", name);
        Ok(())
    }
}

// Helper functions (copied from e2e_home.rs for now)
fn reserve_port() -> TestResult<u16> {
    use std::net::TcpListener;
    let listener = TcpListener::bind((HOST, 0))?;
    Ok(listener.local_addr()?.port())
}

fn wait_for_server(address: &str, child: &mut Child, timeout: Duration) -> TestResult {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if let Some(status) = child.try_wait()? {
            return Err(format!("server exited early with status {status}").into());
        }

        if std::net::TcpStream::connect(address).is_ok() {
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

// Visual regression tests
#[test]
#[ignore = "opt-in visual regression; run with `cargo test --test visual_regression -- --ignored`"]
fn toolbar_initial_state() -> TestResult {
    let session = VisualRegressionSession::launch()?;
    session.assert_visual_match("toolbar_initial_state", 0.01)?;
    Ok(())
}

#[test]
#[ignore = "opt-in visual regression; run with `cargo test --test visual_regression -- --ignored`"]
fn canvas_initial_state() -> TestResult {
    let session = VisualRegressionSession::launch()?;
    session.assert_visual_match("canvas_initial_state", 0.01)?;
    Ok(())
}

#[test]
#[ignore = "opt-in visual regression; run with `cargo test --test visual_regression -- --ignored`"]
fn text_input_overlay() -> TestResult {
    let session = VisualRegressionSession::launch()?;

    // Create a note to trigger text input overlay
    let button = session.tab().find_element("#add-note-button")?;
    button.click()?;
    thread::sleep(Duration::from_millis(200));

    // Double-click the note to enter edit mode
    let canvas = session.tab().find_element(CANVAS_SELECTOR)?;
    let bounds = canvas.get_box_model()?.margin_viewport();
    let center_x = bounds.x + bounds.width / 2.0;
    let center_y = bounds.y + bounds.height / 2.0;

    // Simulate double-click using JavaScript
    let dblclick_script = format!(
        r#"
        (function() {{
            const canvas = document.querySelector('#infinite-canvas');
            if (canvas) {{
                const event = new MouseEvent('dblclick', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {0},
                    clientY: {1}
                }});
                canvas.dispatchEvent(event);
            }}
        }})()
        "#,
        center_x, center_y
    );

    session.tab().evaluate(&dblclick_script, false)?;
    thread::sleep(Duration::from_millis(500)); // Wait for edit mode

    session.assert_visual_match("text_input_overlay", 0.01)?;
    Ok(())
}

#[test]
#[ignore = "utility test to update baselines; run with `cargo test --test visual_regression -- --ignored -p update_baselines`"]
fn update_all_baselines() -> TestResult {
    let session = VisualRegressionSession::launch()?;

    // Update toolbar baseline
    session.update_baseline("toolbar_initial_state")?;

    // Update canvas baseline
    session.update_baseline("canvas_initial_state")?;

    // Update text input overlay baseline
    let button = session.tab().find_element("#add-note-button")?;
    button.click()?;
    thread::sleep(Duration::from_millis(200));

    let canvas = session.tab().find_element(CANVAS_SELECTOR)?;
    let bounds = canvas.get_box_model()?.margin_viewport();
    let center_x = bounds.x + bounds.width / 2.0;
    let center_y = bounds.y + bounds.height / 2.0;

    let dblclick_script = format!(
        r#"
        (function() {{
            const canvas = document.querySelector('#infinite-canvas');
            if (canvas) {{
                const event = new MouseEvent('dblclick', {{
                    bubbles: true,
                    cancelable: true,
                    clientX: {0},
                    clientY: {1}
                }});
                canvas.dispatchEvent(event);
            }}
        }})()
        "#,
        center_x, center_y
    );

    session.tab().evaluate(&dblclick_script, false)?;
    thread::sleep(Duration::from_millis(500));
    session.update_baseline("text_input_overlay")?;

    println!("✅ All baselines updated successfully");
    Ok(())
}
