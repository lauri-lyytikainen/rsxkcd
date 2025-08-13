use colored::Colorize;

pub fn log_info(args: std::fmt::Arguments) {
    print!("{}:    ", "INFO".green());
    println!("{}", args);
}

pub fn log_warning(args: std::fmt::Arguments) {
    print!("{}: ", "WARNING".yellow());
    println!("{}", args);
}

pub fn log_error(args: std::fmt::Arguments) {
    print!("{}:   ", "ERROR".red());
    println!("{}", args);
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logging::log_info(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::logging::log_warning(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logging::log_error(format_args!($($arg)*))
    };
}
