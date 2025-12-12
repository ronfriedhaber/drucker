use std::process::Command;

use super::command;
use super::content::DruckerContent;
use super::options::DruckerOptions;

/// Handle that stores printer options and can submit multiple jobs.
pub struct Drucker {
    /// CUPS/LP options that control the job.
    pub options: DruckerOptions,
}

impl Drucker {
    /// Create a new [`Drucker`] configured with the provided options.
    pub fn new(options: DruckerOptions) -> Self {
        Self { options }
    }

    /// Execute a print job using `lp` (or `lpr` if configured).
    ///
    /// Returns `Ok(())` on success (exit code 0), otherwise `Err(())`.
    pub fn print(&self, content: DruckerContent) -> Result<(), ()> {
        let cmd = self.build_command(&content)?;
        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .status()
            .map_err(|_| ())?;
        if status.success() { Ok(()) } else { Err(()) }
    }

    /// Build a full shell-safe command line string to submit the job.
    pub(crate) fn build_command(&self, content: &DruckerContent) -> Result<String, ()> {
        command::build_command(&self.options, content)
    }
}
