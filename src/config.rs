use crate::output::OutputMode;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub no_dna: bool,
    pub output_mode: OutputMode,
    #[allow(dead_code)]
    pub verbose: bool,
}

impl RuntimeConfig {
    pub fn from_cli(output_flag: Option<OutputMode>, verbose_flag: bool) -> Self {
        let no_dna = std::env::var("NO_DNA").is_ok_and(|v| !v.is_empty());

        let output_mode = output_flag.unwrap_or(if no_dna {
            OutputMode::Json
        } else {
            OutputMode::Human
        });

        let verbose = if no_dna { true } else { verbose_flag };

        Self {
            no_dna,
            output_mode,
            verbose,
        }
    }
}
