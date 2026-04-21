#[path = "common/mod.rs"]
mod common;
use common::*;
use headless_chrome::{Tab, browser::tab::{element::Element, point::Point}};

const MIN_EXPECTED_PAN_Y_DELTA: f64 = 50.0;
const MIN_EXPECTED_TOOLBAR_X_DELTA: f64 = 40.0;
const MIN_EXPECTED_TOOLBAR_Y_DELTA: f64 = 30.0;
const TOOLBAR_HANDLE_SELECTOR: &str = "#floating-toolbar-handle";

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

    assert!(
        bounds.height > bounds.width,
        "expected a vertical toolbar (height={:.1} > width={:.1})",
        bounds.height,
        bounds.width
    );
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

fn ready_toolbar_handle(tab: &Tab) -> TestResult<Element<'_>> {
    Ok(tab.wait_for_element(TOOLBAR_HANDLE_SELECTOR)?)
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_basic -- --ignored`"]
fn home_page_supports_dragging_without_header_copy() -> TestResult {
    let session = HomePageSession::launch()?;

    assert_home_page_starts_clean(session.tab())?;
    assert_toolbar_is_visible(session.tab())?;
    assert_dragging_canvas_updates_pan_coordinates(session.tab())?;

    Ok(())
}

#[test]
#[ignore = "opt-in browser E2E; run with `cargo e2e` or `cargo test --test e2e_basic -- --ignored`"]
fn floating_toolbar_can_be_dragged_over_canvas() -> TestResult {
    let session = HomePageSession::launch()?;

    assert_home_page_starts_clean(session.tab())?;
    assert_toolbar_is_visible(session.tab())?;
    assert_dragging_toolbar_repositions_it(session.tab())?;

    Ok(())
}