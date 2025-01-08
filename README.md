# winston

![Crates.io](https://img.shields.io/crates/v/winston)
![Rust](https://img.shields.io/badge/rust-%E2%9C%94-brightgreen)

A Winston.js-inspired logging library for Rust.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
winston = "0.1"
```

Alternatively, run:

```bash
cargo add winston
```

## Quick Start

### Using the Global Logger

```rust
use winston::{flush, configure, log, transports::stdout, Logger, LoggerOptions};

fn main() {
    let new_options = LoggerOptions::new()
        .level("debug")
        .add_transport(stdout());

    configure(Some(new_options));

    log!(info, "Hello, world!");
    log!(warn, "Something might be wrong.");

    flush();
}
```

The global logger is an application-wide static reference that provides centralized logging access, requiring configuration only once to add a transport, as it starts without a default one. It eliminates the need to pass logger instances around, with functions like `log()`, `configure()` `close()` and `flush()` operating directly on this global logger. Macros like `log!()` implicitly use it. Since static references don’t automatically call `drop`, `flush()` is necessary to ensure all logs are processed, particularly before the application exits.

### Creating Your Own Logger

```rust
use winston::{
    format::{combine, json, timestamp},
    log,
    transports::{stdout, File},
    Logger,
};

fn main() {
    let logger = Logger::builder()
        .level("debug")
        .add_transport(stdout())
        .add_transport(
            File::builder()
                .filename("app.log")
                .build()
        )
        .format(combine(vec![timestamp(), json()]))
        .build();

    log!(logger, info, "Logging with multiple transports");
}
```

---

### Configuration Options

| **Option**              | **Description**                                                    | **Default Value**                                  |
| ----------------------- | ------------------------------------------------------------------ | -------------------------------------------------- |
| `level`                 | Minimum severity of log messages to be logged                      | `info`                                             |
| `levels`                | Severity levels for log entries.                                   | `{error: 0, warn: 1, info: 2, debug: 3, trace: 4}` |
| `transports`            | Logging destinations (`stdout`, `stderr`, `File`, `Custom`).       | None                                               |
| `format`                | Log message formatting (e.g., `json`, `timestamp`).                | `json`                                             |
| `channel_capacity`      | Maximum size of the log message buffer.                            | `1024`                                             |
| `backpressure_strategy` | Action when buffer is full (`Block`, `DropOldest`, `DropCurrent`). | `Block`                                            |

---

### Logging Basics

#### The `log!` Macro

The `log!` macro simplifies logging by combining log level, message, and optional metadata into a single call:

```rust
log!(level, "Message", key1 = value1, key2 = value2);
```

- **`level`**: Log level (`info`, `warn`, `error`, etc.).
- **`Message`**: A string message.
- **Optional key-value pairs**: Metadata to add context.

Examples:

```rust
log!(info, "App initialized"); // Simple log
log!(warn, "API timeout", endpoint = "/v1/data", retries = 3); // With metadata
```

You can also log directly to a specific logger:

```rust
log!(my_logger, debug, "Custom logger used", module = "auth");
```

#### How It Works

The `log!` macro internally creates a `LogInfo` object and passes it to the global logger or a specified logger. It's equivalent to manually constructing and logging a `LogInfo`:

```rust
let entry = LogInfo::new("info", "App initialized")
    .with_meta("key1", "value1")
    .with_meta("key2", "value2");

logger.log(entry); // or just `log(entry)` for the global logger.
```

## Key Concepts

### Transports

Transports define where log messages are sent. Winston supports:

- **WriterTransport**: A generic transport that can write to any destination implementing the `Write` trait (stdout, stderr, files, network sockets, etc.)
- **File**: Log messages to a file.
- **Custom**: Implement the `Transport` trait to define your own destination.

#### Convenience Functions

Quick transport creation for common use cases:

```rust
use winston::transports::{stdout, stderr};

// Quick stdout/stderr transports
let logger = Logger::builder()
    .add_transport(stdout())    // Same as WriterTransport::new(io::stdout())
    .add_transport(stderr())    // Same as WriterTransport::new(io::stderr())
    .build();
```

Example using different writers:

```rust
use std::io;
use winston::transports::WriterTransport;

// Stdout transport
let stdout_transport = WriterTransport::new(io::stdout());

// Stderr transport
let stderr_transport = WriterTransport::new(io::stderr());

// File transport using Write
let file = std::fs::File::create("app.log").unwrap();
let file_transport = WriterTransport::new(file);
```

### Log Levels

Winston's log levels conform to the severity ordering specified by [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424), ranked in ascending order of importance. Lower numeric values indicate more critical (important) events.

```rust
levels: {
    error: 0,   // Critical errors - issues causing system failure
    warn:  1,   // Warnings - potential problems or recoverable issues
    info:  2,   // Informational - standard operations tracking
    debug: 3,   // Debugging - diagnostic details for troubleshooting
    trace: 4    // Tracing - the most verbose, fine-grained logs
}
```

#### Custom Log Levels

You can define custom log levels while maintaining a numeric severity hierarchy:

```rust
use std::collections::HashMap;
use winston::Logger;

let custom_levels = HashMap::from([
    ("critical", 0),  // Highest severity
    ("high",     1),
    ("medium",   2),
    ("low",      3)   // Lowest severity
]);

let logger = Logger::builder()
    .levels(custom_levels)
    .build();
```

### Log Level

The `level` configuration represents the minimum severity of messages to be logged. For instance, if the level is set to `"info"`, the logger will process only `info`, `warn`, and `error` messages while ignoring less critical levels like `debug` and `trace`.

```rust
let logger = Logger::builder()
    .level("info")  // Logs only info, warn, and error levels
    .build();
```

#### Per-Transport Log Level

Each transport can define its own log level, overriding the logger level. This allows for targeted logging based on the output medium.

```rust
let logger = Logger::builder()
    .level("info")  // Logger default level
    .add_transport(
        stderr()
            .with_level("error")  // stderr only logs error messages
    )
    .add_transport(
        File::builder()
            .filename("app.log")
            .level("debug")  // File logs debug and above
            .build()
    )
    .build();
```

In this example:

- The logger level is set to `info`.
- The stderr transport logs only `error` messages.
- The file transport logs `debug` and higher (i.e., `debug`, `info`, `warn`, and `error`).

### Formats

For advanced formatting, Winston leverages [logform](https://github.com/ifeanyi-ugwu/logform_rs).

```rust
use winston::{Logger, format::{combine, timestamp, json}};

let logger = Logger::builder()
    .format(combine(vec![timestamp(), json()]))
    .build();
```

Each transport can have its own format, which takes precedence over the logger format:

```rust
let logger = Logger::builder()
    .format(json())  // Logger default format
    .add_transport(
        stdout()
            .with_format(combine(vec![timestamp(), colored()]))  // Colorized console output
    )
    .add_transport(
        File::builder()
            .filename("app.log")
            .format(json())  // Structured JSON for file logs
            .build()
    )
    .build();
```

## Advanced Features

### Backpressure Handling

Winston provides three backpressure strategies when the logging channel is full:

- `Block`: Wait until space is available
- `DropOldest`: Remove the oldest log message
- `DropCurrent`: Discard the current log message

```rust
use winston::{Logger, BackpressureStrategy};

let logger = Logger::builder()
    .channel_capacity(100)
    .backpressure_strategy(BackpressureStrategy::DropOldest)
    .build();
```

### Log Querying

Winston supports retrieving log entries from transports.

_To enable querying for a custom transport, override the `query` method in your `Transport` implementation._

```rust
use winston::{Logger, LogQuery};

let query = LogQuery::new()
    .from("2 hours ago")
    .until("now")
    .levels(vec!["error"])
    .limit(10)
    .order("desc")
    .search_term("critical")

let results = logger.query(query);
```

### LogQuery Configuration Options

| **Option**    | **Description**                                                                                                        | **Default Value**                     |
| ------------- | ---------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| `from`        | Start time for the query (supports string formats compatible with [`parse_datetime`](https://docs.rs/parse-datetime/)) | `Utc::now() - Duration::days(1)`      |
| `until`       | End time for the query(supports string formats compatible with [`parse_datetime`](https://docs.rs/parse-datetime/))    | `Utc::now()`                          |
| `limit`       | Maximum number of log entries to retrieve.                                                                             | `50`                                  |
| `start`       | Offset for query results, used for pagination.                                                                         | `0`                                   |
| `order`       | Order of results, either `asc`, `ascending`, `descending` or `desc`.                                                   | `Descending`                          |
| `levels`      | List of log levels to include in the query (e.g., `["error", "info"]`).                                                | `[]` (no filter, includes all levels) |
| `fields`      | List of fields to filter by.                                                                                           | `[]` (no specific fields required)    |
| `search_term` | Text to search for in log messages.                                                                                    | `None` (no search term applied)       |

---

#### Logging Timestamps

**Timestamps are essential for effective log querying.** Log entries must include a `timestamp` field in their metadata (`LogInfo.meta`) for Winston’s querying capabilities to function as expected. The `timestamp` field should be a string compatible with [`dateparser`](https://docs.rs/dateparser/).

Winston's built-in `timestamp` format simplifies this requirement:

```rust
use winston::{Logger, timestamp};

let logger = Logger::new()
    .format(timestamp()) // Adds a timestamp to each log entry
    .build();
```

### Runtime Reconfiguration

Change logging configuration dynamically at runtime:

```rust
use winston::{transports::stdout, Logger, LoggerOptions};

let logger = Logger::default();
logger.configure(
    LoggerOptions::new()
        .level("debug")
        .add_transport(stdout())
);
```

## Performance

- **Configurable Buffering:** Adjust channel capacity to match your application's needs

## Contributing

Contributions are welcome! Please submit issues and pull requests on our GitHub repository.

## License

MIT License
