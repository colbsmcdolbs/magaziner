pub struct Progress {
    total: usize,
    current: usize,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            total: 5,
            current: 0,
        }
    }

    pub fn next(&mut self, message: &str) {
        self.current += 1;
        println!("[{}/{}] {}", self.current, self.total, message);
    }

    pub fn done(&self, output: impl std::fmt::Display) {
        println!("Successfully completed: {}", output);
    }

    pub fn substep(&mut self, current: usize, total: usize) {
        let human_readable_current = current + 1;
        println!(
            "[{}/{}] Fetching article {}/{}…",
            self.current, self.total, human_readable_current, total
        );
    }
}
