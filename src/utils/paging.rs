use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

pub fn run_pager(text: &str, pager: &str, no_less_options: bool) -> Result<()> {
    let mut cmd = Command::new(pager);

    if pager == "less" && !no_less_options {
        cmd.arg(
            // Handle ANSI color sequences
            "-R",
        );

        cmd.arg(
            // Quit if input is smaller than the screen's size
            "-F",
        );
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run pager: {pager}"))?;

    let mut stdin = child
        .stdin
        .as_ref()
        .context("Failed to get STDIN pipe from pager")?;

    write!(stdin, "{text}").context("Failed to write data to the pager's STDIN pipe")?;

    let exit = child
        .wait()
        .with_context(|| format!("Pager command '{pager}' failed"))?;

    if !exit.success() {
        bail!("Pager command '{pager}' returned a non-zero exit code");
    }

    Ok(())
}
