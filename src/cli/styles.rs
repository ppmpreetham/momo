use clap::builder::{
    Styles,
    styling::{AnsiColor, Effects},
};

pub const STYLES: Styles = Styles::styled()
    // section titles
    .header(AnsiColor::Blue.on_default().effects(Effects::BOLD))
    // "Usage:"
    .usage(AnsiColor::Magenta.on_default().effects(Effects::BOLD))
    // flags like --help
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    // placeholders like <FILE>
    .placeholder(AnsiColor::BrightBlack.on_default())
    // errors
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    // valid values
    .valid(AnsiColor::Green.on_default())
    // invalid values
    .invalid(AnsiColor::Red.on_default());
