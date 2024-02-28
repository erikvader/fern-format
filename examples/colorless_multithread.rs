use fern_format::Format;

fn main() {
    fern::Dispatch::new()
        .format(Format::new().log_thread_names().callback())
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    let handle1 = std::thread::spawn(|| {
        log::trace!("trace");
        log::debug!("debug");
        log::info!("info");
        log::warn!("warn");
        log::error!("error");
    });

    let handle2 = std::thread::spawn(|| {
        log::trace!("trace");
        log::debug!("debug");
        log::info!("info");
        log::warn!("warn");
        log::error!("error");
    });

    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");

    handle1.join().unwrap();
    handle2.join().unwrap();
}
