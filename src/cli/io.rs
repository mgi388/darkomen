use std::io::{Read as _, Write as _};

pub fn read_input_to_string(input: &str) -> anyhow::Result<String> {
    if input == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(std::fs::read_to_string(input)?)
    }
}

pub fn write_output_string(output: Option<&str>, s: &str) -> anyhow::Result<()> {
    match output {
        None | Some("-") => {
            let stdout = std::io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(s.as_bytes())?;
        }
        Some(path) => std::fs::write(path, s)?,
    }
    Ok(())
}
