use fern_format::Format;
use fern_format::Stream;

fn main() {
    fern::Dispatch::new()
        .format(
            Format::new()
                .color_if_supported(Stream::Stdout)
                .uniquely_color_threads()
                .callback(),
        )
        .chain(std::io::stdout())
        .apply()
        .unwrap();

    std::thread::scope(|s| {
        for i in 0..29 {
            std::thread::Builder::new()
                .name(i.to_string())
                .spawn_scoped(s, || {
                    log::trace!("trace");
                    log::debug!("debug");
                    log::info!("info");
                    log::warn!("warn");
                    log::error!("error");
                })
                .unwrap();
        }
    });
}
