use std::fs::File;
use std::io::{self, Write};
use std::process::Command;

fn main() -> io::Result<()> {
    let git_rev_parse_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()?
        .stdout;

    let git_show_output = Command::new("git")
        .args(["show", "-s", "--format=%cI", "HEAD"])
        .output()?
        .stdout;

    let mut file = File::create("../../assets/versions/git.txt")?;

    file.write_all(&git_rev_parse_output)?;
    file.write_all(&git_show_output)?;

    Ok(())
}
