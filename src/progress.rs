pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

pub struct Progress {
    total: usize,
    current: usize,
    verbosity: Verbosity,
}

impl Progress {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            total: 5,
            current: 0,
            verbosity,
        }
    }

    pub fn next(&mut self, message: &str) {
        self.current += 1;
        if !matches!(self.verbosity, Verbosity::Quiet) {
            println!("[{}/{}] {}", self.current, self.total, message);
        }
    }

    pub fn done(&self, output: impl std::fmt::Display) {
        if !matches!(self.verbosity, Verbosity::Quiet) {
            println!("Successfully completed: {}", output);
        }
    }

    pub fn substep(&mut self, current: usize, total: usize) {
        if !matches!(self.verbosity, Verbosity::Quiet) {
            let human_readable_current = current + 1;
            println!(
                "[{}/{}] Fetching article {}/{}…",
                self.current, self.total, human_readable_current, total
            );
        }
    }

    pub fn verbose(&self, message: &str) {
        if matches!(self.verbosity, Verbosity::Verbose) {
            println!("  {}", message);
        }
    }
}
