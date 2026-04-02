use headless_chrome::{Browser, LaunchOptionsBuilder};
use std::{
    error::Error,
    net::{TcpListener, TcpStream},
    path::Path,
    process::{Child, Command, Stdio},
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

        thread::sleep(Duration::from_millis(200));
    }

    Err(format!("timed out waiting for {address}").into())
}

fn chrome_binary() -> Option<&'static str> {
    const MACOS_CHROME: &str = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";

    Path::new(MACOS_CHROME).exists().then_some(MACOS_CHROME)
}

#[test]
fn home_page_contains_expected_h1() -> Result<(), Box<dyn Error>> {
    let port = reserve_port()?;
    let address = format!("{HOST}:{port}");
    let url = format!("http://{address}/");

    let trunk = Command::new("trunk")
        .args(["serve", "--address", HOST, "--port", &port.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let _trunk_guard = ChildGuard(trunk);

    wait_for_server(&address, Duration::from_secs(20))?;

    let launch_options = LaunchOptionsBuilder::default()
        .path(chrome_binary().map(Into::into))
        .headless(true)
        .build()
        .map_err(|message| format!("failed to build Chrome launch options: {message}"))?;

    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;

    tab.navigate_to(&url)?;
    let title = tab.wait_for_element("h1")?;

    assert_eq!(title.get_inner_text()?, EXPECTED_TITLE);

    Ok(())
}
