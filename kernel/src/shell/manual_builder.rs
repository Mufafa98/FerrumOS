// A manual should provide detailed information about a command
// it should have the folowing format:
// <command_name> - <short_description>
// Description:
// <long_description>
// Usage:
// <usage>
// Args:
// <args>
// Examples:
// <examples>

use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
use crate::drivers::fonts::color::colors;

static MAX_LEN: usize = 50;
fn add_newline_if_needed(description: &str) -> String {
    let mut formatted = String::new();

    // Split by newline explicitly to preserve blank lines.
    for (i, line) in description.split('\n').enumerate() {
        // If it’s just an empty line, preserve it.
        if line.is_empty() {
            formatted.push('\n');
            continue;
        }

        // Otherwise, word-wrap this line:
        let mut count = 0;
        for (j, word) in line.split_whitespace().enumerate() {
            // If adding this word would exceed MAX_LEN, break to next line
            if count + word.len() + if j == 0 { 0 } else { 1 } > MAX_LEN {
                formatted.push('\n');
                formatted.push_str(word);
                count = word.len();
            } else {
                // If not the first word on this line, add a space
                if j > 0 {
                    formatted.push(' ');
                    count += 1; // for the space
                }
                formatted.push_str(word);
                count += word.len();
            }
        }
        // After finishing this line, push a newline
        formatted.push('\n');
    }

    formatted
}

pub(super) struct ManualBuilder {
    name: String,
    short_description: String,
    long_description: String,
    usage: String,
    args: Vec<(String, String)>,     // (arg_name, description)
    examples: Vec<(String, String)>, // (example_name, example_description)
}

impl ManualBuilder {
    pub fn new() -> Self {
        ManualBuilder {
            name: String::new(),
            short_description: String::new(),
            long_description: String::new(),
            usage: String::new(),
            args: Vec::new(),
            examples: Vec::new(),
        }
    }
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }
    pub fn short_description(mut self, description: &str) -> Self {
        self.short_description = description.to_string();
        self
    }
    pub fn long_description(mut self, description: &str) -> Self {
        // self.long_description = add_newline_if_needed(description);
        self.long_description = description.to_string();
        self
    }
    pub fn usage(mut self, usage: &str) -> Self {
        self.usage = usage.to_string();
        self
    }
    pub fn arg(mut self, arg_name: &str, description: &str) -> Self {
        self.args
            .push((arg_name.to_string(), description.to_string()));
        self
    }
    pub fn example(mut self, example_name: &str, example_description: &str) -> Self {
        self.examples
            .push((example_name.to_string(), example_description.to_string()));
        self
    }

    pub fn build_short(&self) -> String {
        format!("{}", self.short_description,)
    }
    pub fn build_long(&self) -> String {
        let description = "Description:\n".bold().fg(colors::LIGHT_BLUE);
        let usage = "Usage:\n".bold().fg(colors::LIGHT_BLUE);
        let args = "Args:\n".bold().fg(colors::LIGHT_BLUE);
        let arg_items = self
            .args
            .iter()
            .map(|(arg_name, desc)| format!("{}: {}", arg_name, desc))
            .collect::<Vec<String>>()
            .join("\n");
        let examples = "Examples:\n".bold().fg(colors::LIGHT_BLUE);
        let example_items = self
            .examples
            .iter()
            .map(|(example_name, example_desc)| format!("{}: {}\n", example_name, example_desc))
            .collect::<Vec<String>>()
            .join("\n");
        let mut long_message = String::new();
        long_message.push_str(&description);
        long_message.push_str("\n");
        long_message.push_str(&self.long_description);
        long_message.push_str("\n\n");
        long_message.push_str(&usage);
        long_message.push_str("\n");
        long_message.push_str(&self.usage);
        long_message.push_str("\n\n");
        if !self.args.is_empty() {
            long_message.push_str(&args);
            long_message.push_str("\n");
            long_message.push_str(&arg_items);
            long_message.push_str("\n\n");
        }
        if !self.examples.is_empty() {
            long_message.push_str(&examples);
            long_message.push_str("\n");
            long_message.push_str(&example_items);
        }
        add_newline_if_needed(&long_message)
    }
    pub fn build_usage(&self) -> String {
        let usage = "Usage:\n".bold().fg(colors::LIGHT_BLUE);
        let mut usage_message = String::new();
        usage_message.push_str(&usage);
        usage_message.push_str("\n");
        usage_message.push_str(&self.usage);
        usage_message.push_str("\n");
        add_newline_if_needed(&usage_message)
    }
}
