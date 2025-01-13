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

| **Option**              | **Description**                                                                                        | **Default Value**                                  |
| ----------------------- | ------------------------------------------------------------------------------------------------------ | -------------------------------------------------- |
| `level`                 | Minimum severity of log messages to be logged. Anything equal to or higher in severity will be logged. | `info`                                             |
| `levels`                | Severity levels for log entries.                                                                       | `{error: 0, warn: 1, info: 2, debug: 3, trace: 4}` |
| `transports`            | Logging destinations (`stdout`, `stderr`, `File`, `Custom`).                                           | None                                               |
| `format`                | Log message formatting (e.g., `json`, `timestamp`).                                                    | `json`                                             |
| `channel_capacity`      | Maximum size of the log message buffer.                                                                | `1024`                                             |
| `backpressure_strategy` | Action when buffer is full (`Block`, `DropOldest`, `DropCurrent`).                                     | `Block`                                            |

---

### Logging Basics

#### The `log!` Macro

The simplest way to log messages is using the `log!` macro:

```rust
// Using the global logger
log!(info, "System started");
log!(warn, "Disk space low", usage = 92);

// Using a specific logger
log!(logger, error, "Connection failed", retries = 3, timeout = 120);
```

It takes the following parameters:

- **`level`**: Log level (`info`, `warn`, `error`, etc.).
- **`Message`**: A string message.
- **Optional key-value pairs**: Metadata to add context.

#### Level-specific Methods and Macros

Winston provides macros to create level-specific logging methods and macros:

```rust
use std::collections::HashMap;
use winston::{meta, Logger};

// Define custom logging methods and macros
winston::create_log_methods!(foo, bar, baz, foobar);
winston::create_level_macros!(foo, bar, baz, foobar);

let logger = Logger::builder()
    .add_transport(stdout())
    .levels(HashMap::from([
        ("foo", 0),  // Most severe
        ("bar", 1),
        ("baz", 2)
        ("foobar", 3)   // Least severe
    ]))
    .level("bar")    // Log bar and more severe levels
    .build();

// Usage
logger.foo("Foo-level message", None);
logger.foobar("Foobar-level message with metadata", Some(meta!(key = "value", timestamp = 1234567890)));
foobar!(logger, "Foobar-level macro logging");
```

## Key Concepts

### Transports

Transports define the destinations where log messages are written. Winston includes core transports that leverage Rust's standard I/O capabilities, with additional custom transports possible through community contributions. Each transport implements the `Transport` trait from [winston_transport](https://github.com/ifeanyi-ugwu/winston_transport_rs):

```rust
pub trait Transport: Send + Sync {
    // Required: Handles writing log messages
    fn log(&self, info: LogInfo);

    // Optional: Flushes buffered logs
    fn flush(&self) -> Result<(), String> { Ok(()) }

    // Optional: Gets minimum log level
    fn get_level(&self) -> Option<&String> { None }

    // Optional: Gets format configuration
    fn get_format(&self) -> Option<&Format> { None }

    // Optional: Retrieves matching log entries
    fn query(&self, _options: &LogQuery) -> Result<Vec<LogInfo>, String> { Ok(Vec::new()) }
}
```

#### Built-in Transports

Winston provides two core transports:

##### WriterTransport

A generic transport that writes to any destination implementing Rust's `Write` trait:

```rust
use std::io::{self, Write};
use winston::transports::WriterTransport;

// Write to stdout
let stdout_transport = WriterTransport::new(io::stdout())
    .with_level("info");

// Write to a file
let file = std::fs::File::create("app.log").unwrap();
let file_transport = WriterTransport::new(file)
    .with_format(json());

// Write to a network socket
let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
let network_transport = WriterTransport::new(stream);
```

There are quick `WriterTransport` creation for common use cases:

```rust
use winston::transports::{stdout, stderr};

// Quick stdout/stderr transports
let logger = Logger::builder()
    .add_transport(stdout())    // Same as WriterTransport::new(io::stdout())
    .add_transport(stderr())    // Same as WriterTransport::new(io::stderr())
    .build();
```

##### File Transport

Specialized file transport with querying capabilities for log retrieval.

#### Creating Custom Transports

To define a custom transport, implement the `Transport` trait and define the `log` method:

```rust
use winston::{log, LogInfo, Transport};

struct MyCustomTransport;

impl Transport for MyCustomTransport {
    fn log(&self, info: LogInfo) {
        println!("Custom transport: {}", info.message);
    }
}

fn main() {
    let custom_transport = MyCustomTransport;

    let logger = Logger::builder()
        .add_transport(custom_transport)
        .build();

    log!(info, "This uses a custom transport!");
}
```

#### Multiple Transports

You can use multiple transports simultaneously, even of the same type. Each transport can have its own configuration:

```rust
use winston::{log, Logger, format::{json, simple}, transports::{stdout, WriterTransport}};
use std::fs::File;

let logger = Logger::builder()
    // Log all info and above to stdout with simple formatting
    .add_transport(
        stdout()
            .with_level("info")
            .with_format(simple())
    )
    // Log all error to file with JSON formatting
    .add_transport(
        WriterTransport::new(File::create("app.log").unwrap())
            .with_level("error")
            .with_format(json())
    )
    .build();

// Usage
log!(error, "Appears in file only");
log!(info, "Appears in both stdout and file");
```

### Logging Levels

Winston's logging levels conform to the severity ordering specified by [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424), ranked in ascending order of importance. Lower numeric values indicate more critical (important) events.

```rust
levels: {
    error: 0,   // Critical errors - issues causing system failure
    warn:  1,   // Warnings - potential problems or recoverable issues
    info:  2,   // Informational - standard operations tracking
    debug: 3,   // Debugging - diagnostic details for troubleshooting
    trace: 4    // Tracing - the most verbose, fine-grained logs
}
```

#### Custom Logging Levels

In addition to the predefined `rust`, `syslog`, and `cli` levels available in winston, you can also choose to define your own:

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
