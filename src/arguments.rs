use clap::{Args, ColorChoice, Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(about, color = ColorChoice::Auto, name = "JetBrains Toolbox Context Menu Generator", version, propagate_version = true, author)]
pub struct JetbrainsToolBoxContextArguments {
    #[command(subcommand)]
    pub subcommands: Option<SubCommands>,
    #[arg(long, short)]
    /// This will suppress all output except for errors and formatted responses.
    /// This is useful for scripting.
    pub quiet: bool,

    #[arg(long, short)]
    /// This will output more information.
    /// This is useful for debugging.
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub enum SubCommands {
    #[command(about = "Scan a directory for JetBrains Toolbox installations")]
    /// Scans a directory for JetBrains Toolbox installations and adds context menu entries for them.
    Scan,
    #[command(about = "Removes the context menu entries for JetBrains Toolbox installations")]
    /// Removes the context menu entries for JetBrains Toolbox installations.
    Uninstall,
    // TODO: Add update feature...
    //    #[command(
    //        about = "Downloads the latest version of JetBrains Toolbox Context Menu from github"
    //    )]
    //    /// Downloads the latest version of JetBrains Toolbox Context Menu from GitHub.
    //    Update(UpdateArguments),
}

#[derive(Debug, Clone, Args)]
pub struct InstallArguments {
    #[cfg_attr(
        target_os = "windows",
        arg(
            long,
            short,
            default_value = "C:\\Program Files",
            help = "The path to the directory to install the context menu entries."
        )
    )]
    #[cfg_attr(
        target_os = "macos",
        arg(
            long,
            short,
            default_value = "/Applications",
            help = "The path to the directory to install the context menu entries."
        )
    )]
    #[cfg_attr(
        target_os = "linux",
        arg(
            long,
            short,
            default_value = "/usr/bin",
            help = "The path to the directory to install the context menu entries."
        )
    )]
    /// The path to the directory to install the context menu entries.
    pub path: String,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArguments {
    #[arg(long, short, conflicts_with = "dry_run")]
    /// This will force the download of the latest version, even if the version is the same as the current version.
    pub force: bool,

    #[arg(long)]
    /// This will not actually download the latest version, but it will show the version number, changelog, and direct download link.
    /// This is useful for debugging and testing.
    pub dry_run: bool,

    #[arg(long, requires = "dry_run")]
    /// This will output all releases, not just the latest one.
    /// This is useful for getting a specific release.
    pub all: bool,

    #[arg(long, short, visible_alias = "format", value_enum, default_value_t = FormatOptions::PlainText)]
    /// The format of the output.
    /// This is useful for scripting.
    pub style: FormatOptions,
}
#[derive(Debug, Clone, ValueEnum)]
pub enum FormatOptions {
    Json,
    Yaml,
    PlainText,
}
