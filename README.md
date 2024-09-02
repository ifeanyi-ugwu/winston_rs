# winston

![Crates.io](https://img.shields.io/crates/v/winston)
![Rust](https://img.shields.io/badge/rust-%E2%9C%94-brightgreen)

A customizable, multithreaded logging library for Rust, inspired by the flexibility of Winston.js. This logger supports custom log levels, multiple transports, real-time filtering, and format customization.

## Features

- **Custom Log Levels:** Define your own log levels or use the default ones.
- **Multiple Transports:** Log to multiple destinations (e.g., files, console, etc.).
- **Real-Time Filtering:** Filter logs based on log levels and conditions.
- **Custom Formats:** Define how log entries are formatted.(implemented in [logform](https://github.com/ifeanyi-ugwu/logform_rs))
- **Multithreading:** Logs are processed in a separate thread to avoid blocking your application.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
winston = "0.1"
```

or with

```bash
cargo add winston
```

## Usage

### 1. Basic Setup

Start by configuring the default logger:

```rust
use winston::{LoggerOptions, Logger, log_info, log_warn, log_error, configure};

fn main() {
    let new_options = LoggerOptions::new()
        .level("debug").add_transport(Console::new(None));

    configure(Some(new_options));

    log_info!("This is an info message.");
    log_warn!("This is a warning.");
    log_error!("This is an error.");

    Logger::shutdown();
}
```

### 2. Custom Logger Configuration

You can configure the logger with custom log levels, transports, and formats:

```rust
use winston::{transports::Console, Logger, LoggerOptions, log_warn, format::json};

fn main() {
    let options = LoggerOptions::new()
        .level("info")
        .format(json())
        .add_transport(Console::new(None));

    let logger = Logger::new(Some(options)); // or with Logger::builder().level("info").format(json()).add_transport(Console::new(None)).build();

    logger.warn("Custom logger warning!");
}
```

### 3. Querying Logs

You can query the logs that are in queryable transports like files:

```rust
use winston::{Logger, LogQuery, format};

fn main() {
    let logger = Logger::builder()
        .add_transport(
            transports::File::builder()
                .filename(temp_path.clone())
                .build(),
        )
        .format(format::combine(vec![format::timestamp(), format::json()]))
        .build();

    // Log some messages
    logger.info("Test message 1");
    logger.error("Test error message");
    logger.warn("Test warning");

    // For testing purpose, Sleep for a short duration to ensure logs are flushed to the file so the query will retrieve them
    // logging messages and immediately querying logs is highly unlikely
    std::thread::sleep(std::time::Duration::from_secs(1));

    let query = LogQuery::new()
        .levels(vec!["error"]);

    let results = logger.query(&query).unwrap();

    for entry in results {
        println!("{:?}", entry);
    }
}
```

### 4. Shutting Down the Logger

Ensure all log entries are processed before your application exits(this is only necessary for the default logger since statics do not call drop):

```rust
use winston::Logger;

fn main() {
    // Your code here

    // Gracefully shut down the logger
    Logger::shutdown();
}
```

### 5. Global Logger

A global logger is provided for convenience. You can log messages using macros:

```rust
use winston::{log_info, log_warn, log_error};

fn main() {
    log_info!("Global info log");
    log_warn!("Global warning log");
    log_error!("Global error log");
}
```

### 6. Changing Configuration at Runtime

You can reconfigure the logger during runtime:

```rust
use winston::{Logger, LoggerOptions};

fn main() {
    let logger = Logger::new(None);

    let new_options = LoggerOptions::new()
        .level("debug").add_transport(Console::new(None));

    logger.configure(Some(new_options));

    logger.debug("This is a debug message after reconfiguration.");
}
```

## Contributing

Feel free to contribute to this project by submitting issues or pull requests.

## License

This project is licensed under the MIT License.
