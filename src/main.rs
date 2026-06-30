// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Chase Taylor
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use console::Style;
use materia_forge::{installers, logging};
use std::{env, sync::LazyLock};

// Check for Steam Deck
static IS_DECK: LazyLock<bool> = LazyLock::new(|| {
    std::fs::read_to_string("/etc/os-release")
        .map(|s| ["SteamOS", "Bazzite"].iter().any(|id| s.contains(id)))
        .unwrap_or(false)
        || env::args().any(|a| a == "-d" || a == "--deck")
});

fn main() {
    if let Err(e) = logging::init("MateriaForge.log") {
        eprintln!("Failed to create log file! Error: {e}");
        std::process::exit(1);
    }
    log::info!("Starting MateriaForge version {}", VERSION);
    log::info!("Running on Steam Deck: {}", *IS_DECK);

    draw_header();

    if logging::log_and_return(installers::run(*IS_DECK)).is_err() {
        std::process::exit(1);
    }
}

fn draw_header() {
    let title = format!("Welcome to MateriaForge {VERSION}");
    let mut description = vec![
        "This script will:",
        "1. Detect supported FF7/FF8/FF9 Steam/GOG installs (GOG support via Heroic)",
        "2. Apply proton prefix patches for the selected game",
        "3. Install the corresponding mod loader to a folder of your choosing",
        "4. Optionally add a desktop shortcut and Steam shortcut for easy access",
    ];
    let mut footer = [
        "   For support, please open an issue on GitHub, or ask in the #ff7-linux / #ff8-linux channels of the Tsunamods Discord",
        "",
        "   Use arrow keys and Enter to navigate the prompts.",
    ];

    if *IS_DECK {
        description.append(
            &mut [
                "5. Add a custom controller config for Steam Deck, to allow mouse",
                "   control with trackpad without holding down the STEAM button",
            ]
            .to_vec(),
        );
        footer[2] = "   Use D-Pad and A button to navigate the prompts.";
    }

    // Pad description
    let description: Vec<String> = description
        .iter()
        .map(|line| format!("    {line}    "))
        .collect();

    // Define styles
    let border_style = Style::new().cyan(); // Cyan borders
    let title_style = Style::new().bold().cyan(); // Bold cyan title
    let text_style = Style::new().white(); // White text
    let footer_style = Style::new().dim().white(); // Dim white footer

    // Calculate the maximum line width in the description
    let max_description_width = description.iter().map(|line| line.len()).max().unwrap_or(0);

    // Calculate the banner width based on the longest description line
    let banner_width = max_description_width + 4; // 2 spaces padding + 2 border characters

    // Define border characters
    let top_border = format!("┏{}┓", "━".repeat(banner_width - 2));
    let bottom_border = format!("┗{}┛", "━".repeat(banner_width - 2));
    let middle_border = format!("┣{}┫", "━".repeat(banner_width - 2));
    let border_char = "┃";

    // Print the top border
    println!("{}", border_style.apply_to(top_border));

    // Print the title
    println!(
        "{} {:^max_description_width$} {}",
        border_style.apply_to(border_char),
        title_style.apply_to(title),
        border_style.apply_to(border_char)
    );

    // Print the middle border
    println!("{}", border_style.apply_to(&middle_border));

    // Print the description
    for line in description.iter() {
        println!(
            "{} {:<max_description_width$} {}",
            border_style.apply_to(border_char),
            text_style.apply_to(line),
            border_style.apply_to(border_char)
        );
    }

    // Print the bottom border
    println!("{}", border_style.apply_to(middle_border));

    // Wrap the footer to match the width of the longest description line and print it
    for line in footer.iter() {
        let wrapped_line = textwrap::fill(line, max_description_width);
        let lines: Vec<&str> = wrapped_line.lines().collect();
        for wrapped in &lines {
            println!(
                "{} {:<max_description_width$} {}",
                border_style.apply_to(border_char),
                footer_style.apply_to(wrapped),
                border_style.apply_to(border_char),
            );
        }
        if lines.len() > 1 {
            println!(
                "{} {:<max_description_width$} {}",
                border_style.apply_to(border_char),
                "",
                border_style.apply_to(border_char),
            );
        }
    }

    // Print the bottom border
    println!("{}", border_style.apply_to(bottom_border));
}
