use std::io::Result;

fn main() -> Result<()> {
    lalrpop::process_root()?;
    Ok(())
}