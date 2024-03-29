use fern_format::Format;
use fern_format::Stream;

fn main() {
    fern::Dispatch::new()
        .format(Format::new().color_if_supported(Stream::Stdout).callback())
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    log::trace!("trace");
    log::debug!("debug");
    log::info!("info");
    log::warn!("warn");
    log::error!("error");
}
