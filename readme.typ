#import "@preview/wrap-it:0.1.0": wrap-content  // https://github.com/ntjess/wrap-it/blob/main/docs/manual.pdf
#import "./doc_templates/src/note.typ": *
#import "./doc_templates/src/style.typ": set_style


#show: doc => set_style(
    topic: "setup_logging",
    author: "구FS",
    language: "EN",
    doc
)
#set text(size: 3.5mm)


#align(center, text(size: 2em, weight: "bold")[setup_logging])
#line(length: 100%, stroke: 0.3mm)
\
\
= Introduction

This setup function provides a clean logging setup for the `log` crate using `fern`.

- Messages with linebreaks are properly indented.
- Timestamps are only printed if they changed from timestamp of previous line.
Console only:
- "\\r" at the beginning of a message overwrites previous line.
- Logging levels are colour-coded.

= Table of Contents

#outline()

#pagebreak(weak: true)

= Installation

+ `cargo add setup_logging --git https://github.com/9-FS/setup_logging`
+ In your `Cargo.toml` entry, replace "version" with "tag" to select the version you want to use.

#info()[Cargo does not support automatic versioning for GitHub dependencies. Manual updates are required in the `Cargo.toml` file using "tag".]

= Usage

Call `setup_logging` with the desired minimum log level and log filepath format according to the `chrono` crate. After that, `log` can be used as usual.

== Example

```rust
setup_logging::setup_logging(log::Level::Info, "./log/%Y-%m-%d.log");

log::debug!("debug message"); // not printed
log::info!("info message"); // printed
log::warn!("warn message"); // printed without timestamp
log::error!("error message"); // printed without timestamp
```

== Example

```rust
setup_logging::setup_logging(log::Level::Debug, "./log/%Y-%m-%dT%H_%M.log");

log::debug!("debug message"); // printed
log::info!("info message"); // printed without timestamp
log::warn!("warn message"); // printed without timestamp
log::error!("error message"); // printed without timestamp
```