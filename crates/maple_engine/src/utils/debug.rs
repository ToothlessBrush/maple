//! provides some helpful debug functionality beyond the standard library

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::OnceLock;

static PRINTED_MESSAGES: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// provides helpful functions that are useful for debugging
pub struct Debug;

impl Debug {
    fn messages() -> &'static Mutex<HashSet<String>> {
        PRINTED_MESSAGES.get_or_init(|| Mutex::new(HashSet::new()))
    }

    /// only print a message in debug
    pub fn print(message: &str) {
        #[cfg(debug_assertions)]
        {
            println!("{}", message);
        }
    }

    /// will print a string once for use in repeated behavior
    ///
    /// This is useful if you want to print something but dont want your console to be spammed with
    /// the same message over and over again
    ///
    /// # Example
    /// ```rust
    /// use maple::utils::Debug;
    ///
    /// Debug::print_once("this message will only print once");
    /// Debug::print_once("this message will only print once");
    /// ```
    pub fn print_once(message: &str) {
        #[cfg(debug_assertions)]
        {
            if let Ok(mut messages) = Self::messages().lock() {
                if messages.insert(message.to_string()) {
                    println!("{}", message);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_once() {
        Debug::print_once("this message should only print once");
        Debug::print_once("this message should only print once");

        Debug::print_once("this will print after");
    }
}
