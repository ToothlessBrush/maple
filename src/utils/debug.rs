/// provides helpful functions that are useful for debugging
pub struct Debug;

impl Debug {
    /// only print a message in debug
    pub fn print(message: &str) {
        #[cfg(debug_assertions)]
        {
            println!("{}", message);
        }
    }
}
