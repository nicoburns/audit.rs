/// Initialize logging.
fn init_logging() {
    use dioxus::logger::tracing::Level;

    // use debug level for debug builds, warn level for production builds.
    #[cfg(debug_assertions)]
    let level = Level::DEBUG;
    #[cfg(not(debug_assertions))]
    let level = Level::WARN;

    dioxus::logger::init(level).expect("failed to initialize logger");
}

fn main() {
    init_logging();
    dioxus::launch(diff_rs::App);
}
