pub fn emit(shell: Option<&str>) -> anyhow::Result<()> {
    match shell {
        Some("bash") => {
            print!("{BASH}");
            Ok(())
        }
        Some("zsh") => {
            print!("{ZSH}");
            Ok(())
        }
        Some("fish") => {
            print!("{FISH}");
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown shell '{other}' — try bash, zsh or fish"),
        None => anyhow::bail!("usage: podctl completion <bash|zsh|fish>"),
    }
}

const BASH: &str = include_str!("../../dist/completion/podctl.bash");
const ZSH: &str = include_str!("../../dist/completion/_podctl.zsh");
const FISH: &str = include_str!("../../dist/completion/podctl.fish");
