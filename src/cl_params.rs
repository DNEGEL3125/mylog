use gumdrop::Options;

// Defines options that can be parsed from the command line.
//
// `derive(Options)` will generate an implementation of the trait `Options`.
// Each field must either have a `Default` implementation or an inline
// default value provided.
//
// (`Debug` is derived here only for demonstration purposes.)
#[derive(Debug, Options)]
pub struct MyOptions {
    #[options(help = "Write log message to log file")]
    pub write: Option<String>,

    #[options(help = "Read log messages in log file")]
    pub read: bool,

    #[options(help = "print help message")]
    pub help: bool,

    #[options(help = "Do not print any messages")]
    pub quiet: bool,

    #[options(help = "Enable colorful output")]
    pub colorful: bool,

    #[options(help = "The date of the logs you want to read")]
    pub date: Option<String>,
}
