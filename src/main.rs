mod markup;
mod model;
mod resource;
mod template;

use anyhow::Result;

/// Command line program options
#[derive(clap_derive::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProgramOptions {
    /// Path to Doxygen xml output
    #[clap(value_parser)]
    input_path: String,
    #[clap(value_parser)]
    output_path: String,
}

fn process(options: ProgramOptions) -> Result<()> {
    let input_path = std::path::Path::new(&options.input_path);
    anyhow::ensure!(input_path.exists());
    let output_path = std::path::Path::new(&options.output_path);
    std::fs::create_dir_all(output_path).ok();
    anyhow::ensure!(!output_path.is_file());
    let blueprints = input_path
        .read_dir()?
        .filter_map(|entry| -> Option<markup::Blueprint> {
            match entry.ok()? {
                e if e.file_type().ok()?.is_file() => {
                    let bp = markup::Blueprint::parse_file(&e.path());
                    if let Err(er) = &bp {
                        println!("{}", er)
                    }
                    bp.ok()
                }
                e if e.file_type().ok()?.is_dir() => None,
                _ => unreachable!(),
            }
        });
    let _eng = template::Engine::new()?;
    resource::THEME_DEFAULT.extract(output_path)?;

    model::Model::new(blueprints).store(output_path)?;
    Ok(())
}

fn main() {
    use clap::StructOpt;

    let options = ProgramOptions::parse();
    process(options).ok();
}

#[cfg(test)]
mod tests {
    use clap::StructOpt;

    const INPUT_PATH: &str = "test/sample";
    const OUTPUT_PATH: &str = "target/out";

    #[test]
    #[ignore]
    fn generate() {
        let options = super::ProgramOptions::parse_from(["blueprint", INPUT_PATH, OUTPUT_PATH]);
        super::process(options).unwrap();
    }

    #[test]
    fn options() {
        let options = super::ProgramOptions::parse_from(["blueprint", INPUT_PATH, OUTPUT_PATH]);
        assert_eq!(options.input_path, INPUT_PATH);
    }
}
