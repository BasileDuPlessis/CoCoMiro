use headless_chrome::{
    Browser, LaunchOptionsBuilder, Tab,
    browser::tab::{element::Element, point::Point},
    protocol::cdp::Input,
};
use std::{
    error::Error,
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

const EXPECTED_TITLE: &str = "Hello world from CoCoMiro!";
const HOST: &str = "127.0.0.1";

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

fn wait_for_server(address: &str, timeout: Duration) -> Result<(), Box<dyn Error>> {
    let deadline = Instant::now() + timeout;

    while Instant::now() < deadline {
        if TcpStream::connect(address).is_ok() {
            return Ok(());
        }

        thread::sleep(Duration::from_millis(50));
    }

    Err(format!("timed out waiting for {address}").into())
}

fn chrome_binary() -> Option<&'static str> {
    const MACOS_CHROME: &str = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

    Path::new(MACOS_CHROME).exists().then_some(MACOS_CHROME)
}

fn open_home_page() -> Result<(ChildGuard, Browser, Arc<Tab>), Box<dyn Error>> {
    let port = reserve_port()?;
    let address = format!("{HOST}:{port}");
    let url = format!("http://{address}/");

    let trunk = Command::new("trunk")
        .args(["serve", "--address", HOST, "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let trunk_guard = ChildGuard(trunk);

    wait_for_server(&address, Duration::from_secs(20))?;

    let launch_options = LaunchOptionsBuilder::default()
        .path(chrome_binary().map(Into::into))
        .headless(true)
        .build()
        .map_err(|message| format!("failed to build Chrome launch options: {message}"))?;

    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.navigate_to(&url)?;
    tab.wait_for_element("h1")?;

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

fn assert_dragging_canvas_updates_pan_coordinates(tab: &Tab) -> Result<(), Box<dyn Error>> {
    let canvas = tab.wait_for_element("#infinite-canvas")?;
    let initial_pan_x = attribute_as_f64(&canvas, "data-pan-x")?;
    let initial_pan_y = attribute_as_f64(&canvas, "data-pan-y")?;

    let bounds = canvas.get_box_model()?.margin_viewport();
    let start = Point {
        x: bounds.x + (bounds.width / 2.0),
        y: bounds.y + (bounds.height / 2.0),
    };
    let end = Point {
        x: start.x + 140.0,
        y: start.y + 90.0,
    };

    tab.move_mouse_to_point(start)?;
    dispatch_mouse_event(
        tab,
        Input::DispatchMouseEventTypeOption::MousePressed,
        start,
        Some(Input::MouseButton::Left),
        Some(1),
    )?;

    for step in 1..=6 {
        let progress = step as f64 / 6.0;
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

    thread::sleep(Duration::from_millis(250));

    let updated_canvas = tab.wait_for_element("#infinite-canvas")?;
    let final_pan_x = attribute_as_f64(&updated_canvas, "data-pan-x")?;
    let final_pan_y = attribute_as_f64(&updated_canvas, "data-pan-y")?;

    assert!(final_pan_x - initial_pan_x > 80.0);
    assert!(final_pan_y - initial_pan_y > 50.0);

    Ok(())
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_home -- --ignored`"]
fn home_page_contains_expected_h1_and_supports_dragging() -> Result<(), Box<dyn Error>> {
    let (_trunk_guard, _browser, tab) = open_home_page()?;
    let title = tab.wait_for_element("h1")?;
    let canvas = tab.wait_for_element("#infinite-canvas")?;

    assert_eq!(title.get_inner_text()?.trim(), EXPECTED_TITLE);
    assert!(attribute_as_f64(&canvas, "data-pan-x")?.abs() < 0.01);
    assert!(attribute_as_f64(&canvas, "data-pan-y")?.abs() < 0.01);
    assert!((attribute_as_f64(&canvas, "data-zoom")? - 1.0).abs() < 0.01);

    assert_dragging_canvas_updates_pan_coordinates(tab.as_ref())?;

    Ok(())
}
