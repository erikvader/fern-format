use fern_format::Format;

fn main() {
    fern::Dispatch::new()
        .format(Format::new().callback())
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");
}
