// Argument parser for command-line arguments

use std::collections::HashMap;
use anyhow::Result;

pub struct ArgumentParser;

impl ArgumentParser {
    pub fn parse(args: &[String]) -> Result<HashMap<String, String>> {
        let mut arguments = HashMap::new();

        for argument in args {
            if let Some(idx) = argument.find(':') {
                let key = argument[..idx].to_string();
                let value = argument[idx + 1..].to_string();
                arguments.insert(key, value);
            } else {
                arguments.insert(argument.to_string(), String::new());
            }
        }

        Ok(arguments)
    }
}
