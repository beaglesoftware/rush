mod shell;

use dirs::config_dir;
use miette::Result;

fn main() -> Result<()> {
    let version = "0.1.0";
    let mut confdir = config_dir().unwrap();
    confdir.push("Rush");
    let config_dir = confdir;
    println!("Rush v{}", version);
    shell::run_shell(config_dir)
}
