pub struct Config {
    pub host: String,
    pub port: u16,
    pub journal_path: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            journal_path: "blitz-journal.log".to_string(),
            port: 6379,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}