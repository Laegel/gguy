#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Profile,
    Info,
    Warn,
    Error,
}

pub type LogEntry = (LogLevel, String);

pub trait Logger: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}
