pub mod human;
pub mod json;
pub mod markdown;
pub mod plain;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum OutputMode {
    Json,
    Plain,
    Markdown,
    Human,
}

/// Anything that can be rendered to each output mode.
pub trait Renderable: Serialize {
    fn render_human(&self) -> String;
    fn render_plain(&self) -> String;
    fn render_markdown(&self) -> String;

    fn render_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    fn render(&self, mode: OutputMode) -> String {
        match mode {
            OutputMode::Json => self.render_json(),
            OutputMode::Plain => self.render_plain(),
            OutputMode::Markdown => self.render_markdown(),
            OutputMode::Human => self.render_human(),
        }
    }
}

/// Render and print to stdout.
pub fn print_output(item: &impl Renderable, mode: OutputMode) {
    println!("{}", item.render(mode));
}
