use std::error::Error;

use chrono::{Local, SecondsFormat};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::Handle;

pub fn init() -> Result<Handle, impl Error> {
    let encoder = Box::new(PatternEncoder::new(
        "{d:<35} [thread \"{T}\" id {({I}]):<6} {l:<5} {t} >> {m}{n}",
    ));

    let stdout = ConsoleAppender::builder().encoder(encoder.clone()).build();
    let file_name = format!(
        "logs/logfile_{}.log",
        Local::now()
            .to_rfc3339_opts(SecondsFormat::Nanos, true)
            .replace(":", "-"),
    );
    let file = FileAppender::builder()
        .encoder(encoder)
        .build(file_name)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .build(
            Root::builder()
                .appenders(["stdout", "file"])
                .build(LevelFilter::Debug),
        )
        .expect("wrong logger configuration");
    log4rs::init_config(config)
}
