// Copyright (c) 2024 구FS, all rights reserved. Subject to the MIT licence in `licence.md`.
use unicode_segmentation::UnicodeSegmentation;


/// # Summary
/// Sets up the fern logging framework.
///
/// # Arguments
/// - `logging_level`: minimum logging level to log, discards all logs below this level
/// - `crate_logging_level`: minimum logging level to log for specific crates, discards all logs below this level
/// - `filepath_format`: format of the filepath to write to, formatted with chrono::Utc::now()
pub fn setup_logging(logging_level: log::Level, crate_logging_level: Option<std::collections::HashMap<String, log::Level>>, filepath_format: &'static str) -> ()
{
    let mut console_dispatch: fern::Dispatch;
    let console_formatter: std::sync::Mutex<Formatter> = std::sync::Mutex::new(Formatter::new(logging_level, Output::Console));
    let mut file_dispatch: fern::Dispatch;
    let file_formatter: std::sync::Mutex<Formatter> = std::sync::Mutex::new(Formatter::new(logging_level, Output::File));


    console_dispatch = fern::Dispatch::new()
        .format(move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record| console_formatter.lock().unwrap().format(out, message, record)) // set formatter
        .level(logging_level.to_level_filter()) // set minimum logging level
        .chain(fern::Output::stderr("\n")); // log to console

    file_dispatch = fern::Dispatch::new()
        .format(move |out: fern::FormatCallback, message: &std::fmt::Arguments, record: &log::Record| file_formatter.lock().unwrap().format(out, message, record)) // set formatter
        .level(logging_level.to_level_filter()) // set minimum logging level
        .chain(fern::Output::call(move |record: &log::Record| {
            write_record_to_file(record, filepath_format).unwrap_or_else(|e: std::io::Error| {
                eprintln!(
                    "{} ERROR Writing previous logging message to log file failed with \"{e}\". Unlogged message:\n\"\"\"\n{}\n\"\"\"",
                    chrono::Utc::now().format("[%Y-%m-%dT%H:%M:%S]").to_string(),
                    record.args(),
                );
            });
        })); // log to file

    for (crate_name, crate_logging_level) in crate_logging_level.unwrap_or(std::collections::HashMap::new())
    // set minimum logging level for specific crates
    {
        console_dispatch = console_dispatch.level_for(crate_name.clone(), crate_logging_level.to_level_filter());
        file_dispatch = file_dispatch.level_for(crate_name, crate_logging_level.to_level_filter());
    }


    fern::Dispatch::new()
        .chain(console_dispatch) // log to stdout
        .chain(file_dispatch) // log to file
        .apply() // apply configuration
        .unwrap();

    return;
}


struct Formatter
{
    line_previous_len:       usize,      // line previous' length
    line_previous_timestamp: String,     // line previous' timestamp
    logging_level:           log::Level, // minimum logging level to log
    timestamp_previous:      String,     // timestamp previously used
    output:                  Output,     // where to log to
}

impl Formatter
{
    /// # Summary
    /// Creates a new `Formatter` instance.
    ///
    /// # Arguments
    /// - `logging_level`: minimum logging level to log, discards all logs below this level
    /// - `output`: where to log to
    fn new(logging_level: log::Level, output: Output) -> Self
    {
        return Formatter {
            line_previous_len:       0,
            line_previous_timestamp: String::new(),
            logging_level:           logging_level,
            timestamp_previous:      String::new(),
            output:                  output,
        };
    }

    /// # Summary
    /// Formats log messages to my personal preferences.
    /// - Messages with linebreaks are properly indented.
    /// - Timestamps are only printed if they changed from timestamp of previous line.
    /// Console only:
    /// - "\r" at the beginning of a message overwrites previous line.
    /// - Logging levels are colour-coded.
    fn format(&mut self, out: fern::FormatCallback, message_content: &std::fmt::Arguments, record: &log::Record) -> ()
    {
        const DEBUG_COLOUR: fern::colors::Color = fern::colors::Color::White;
        const ERROR_COLOUR: fern::colors::Color = fern::colors::Color::BrightRed;
        const INFO_COLOUR: fern::colors::Color = fern::colors::Color::Green;
        const WARN_COLOUR: fern::colors::Color = fern::colors::Color::BrightYellow;
        let logging_level_colours: fern::colors::ColoredLevelConfig;
        let mut message = String::new(); // message to log with all formatting like timestamps or space, logging level, message content
        let mut message_content: String = message_content.to_string(); // message content to log, &std::fmt::Arguments -> String
        let overwrite_line_current: bool; // whether to overwrite previous line
        let timestamp: String; // timestamp to use, can be timestamp_current or spaces
        let timestamp_current: String = chrono::Utc::now().format("[%Y-%m-%dT%H:%M:%S]").to_string(); // now


        if message_content.graphemes(true).collect::<Vec<&str>>()[0] == "\r"
        // if message starts with "\r"
        {
            message_content = message_content.graphemes(true).collect::<Vec<&str>>()[1..].concat(); // remove carriage return

            match self.output
            {
                Output::Console =>
                // if console
                {
                    overwrite_line_current = true; //overwrite line previous
                    eprint!("\x1B[A{}\r", " ".repeat(self.line_previous_len).as_str());
                    // move cursor up 1 line, overwrite line previous, move cursor to line beginning
                }
                Output::File =>
                {
                    overwrite_line_current = false; // do not overwrite line previous
                }
            }
        }
        else
        // if message does not start with "\r"
        {
            overwrite_line_current = false; // do not overwrite line previous
        }

        if overwrite_line_current == false
        // if writing in line new:
        {
            self.line_previous_timestamp = self.timestamp_previous.clone(); // line previous' timestamp is timestamp previously used
        }
        if self.line_previous_timestamp == timestamp_current
        // if line previous' timestamp same as timestamp current
        {
            timestamp = format!("{}", " ".repeat(self.line_previous_timestamp.len()).as_str());
            // do not print timestamp, overwrite with spaces
        }
        else
        //usually just timestamp current
        {
            timestamp = timestamp_current.clone();
        }


        message += timestamp.as_str();
        if log::Level::Debug <= self.logging_level
        // if logging level is debug or lower
        {
            message += &format!(" [{}]", record.target()).as_str(); // apprend crate name
        }

        message_content = message_content.replace("\n", format!("\n{}", " ".repeat(message.len() + 7)).as_str()); // after linebreaks indent content, here because timestamp and crate name already accounted for and logging level always same length

        match self.output
        {
            Output::Console =>
            {
                logging_level_colours = fern::colors::ColoredLevelConfig::new() // set colours for different logging levels
                    .error(ERROR_COLOUR)
                    .warn(WARN_COLOUR)
                    .info(INFO_COLOUR)
                    .debug(DEBUG_COLOUR)
                    .trace(DEBUG_COLOUR);

                message += &format!(" {:5}", logging_level_colours.color(record.level())).as_str();
                // append coloured logging level
            }
            Output::File =>
            {
                message += &format!(" {:5}", record.level()).as_str(); // append logging level
            }
        }
        message += &format!(" {}", message_content).as_str(); // append message content
        out.finish(format_args!("{}", message)); // finish message

        self.line_previous_len = message.len(); // line previous' length
        self.timestamp_previous = timestamp_current; // timestamp previously used = timestamp current, not timestamp so proper comparison disregards use of spaces
    }
}


/// # Summary
/// logging outputs
enum Output
{
    Console,
    File,
}


/// # Summary
/// Appends a log record to a file at filepath_format. The file is created if it does not exist.
///
/// # Arguments
/// - `record`: log record to write
/// - `filepath_format`: format of the filepath to write to, formatted with chrono::Utc::now()
///
/// # Returns
/// nothing or `std::io::Error` if the file could not be opened or written to
fn write_record_to_file(record: &log::Record, filepath_format: &str) -> Result<(), std::io::Error>
{
    let mut file: std::fs::File;
    let filepath: String;


    filepath = chrono::Utc::now().format(filepath_format).to_string(); // convert filepath_format to string with current datetime


    std::fs::create_dir_all(std::path::Path::new(&filepath).parent().unwrap_or(std::path::Path::new("")))?; // create necessary parent directories if they do not exist yet, if no parent directory determination possible give "" to not create any directoy
    file = std::fs::OpenOptions::new().create(true).append(true).open(filepath)?; // open file
    std::io::Write::write_all(&mut file, format!("{}\n", record.args()).as_bytes())?; // write log record to file

    return Ok(());
}
