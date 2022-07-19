mod bp;

/// Command line program options
#[derive(clap_derive::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct ProgramOptions {
    /// Path to Doxygen xml output
    #[clap(value_parser)]
    path: String,
}

fn main() {
    use clap::StructOpt;

    let _options = ProgramOptions::parse();
}

#[cfg(test)]
mod tests {
    use clap::StructOpt;

    const PATH: &str = "sample/xml";

    #[test]
    fn test_program_options() {
        let options = super::ProgramOptions::parse_from(["blueprint", "--path", PATH]);
        assert_eq!(options.path, PATH);
    }
}
