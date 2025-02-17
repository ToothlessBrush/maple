enum Modes {
    Debug,
    Production,
}

pub struct Debug {}

impl Debug {
    pub fn print(message: &str) {
        #[cfg(debug_assertions)]
        {
            println!("{}", message);
        }
    }
}
