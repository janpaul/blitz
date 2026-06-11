pub struct Config {
    pub host: String,
    pub port: u16,
    pub udp_port: u16,
    pub journal_path: String,
    pub with_datagram: bool,
}

impl Config {
    pub fn new() -> Self {
        Config {
            host: "127.0.0.1".to_string(),
            journal_path: "blitz-journal.log".to_string(),
            port: 6379,
            udp_port: 6380,
            with_datagram: true,
        }
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    pub fn datagram_address(&self) -> String {
        format!("{}:{}", self.host, self.udp_port)
    }
}
