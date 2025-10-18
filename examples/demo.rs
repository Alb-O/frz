use frz::{AttributeRow, FileRow, SearchData, SearchSelection, SearchUi, UiConfig};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let attributes = vec![
        AttributeRow::new("frontend", 3),
        AttributeRow::new("backend", 2),
    ];
    let files = vec![
        FileRow::new("src/main.rs", ["frontend"]),
        FileRow::new("src/lib.rs", ["backend"]),
    ];

    let data = SearchData::new()
        .with_context("example/repo")
        .with_initial_query("")
        .with_attributes(attributes)
        .with_files(files);

    // Minimal search UI configuration with prompt
    let search_ui = SearchUi::new(data)
        .with_ui_config(UiConfig::tags_and_files())
        .with_input_title("workspace-prototype")
        .with_start_mode(frz::extensions::builtin::attributes::mode());
    let outcome = search_ui.run()?;
    println!("Accepted? {}", outcome.accepted);
    match outcome.selection {
        Some(SearchSelection::File(file)) => println!("Selected file: {}", file.path),
        Some(SearchSelection::Attribute(attribute)) => {
            println!("Selected attribute: {}", attribute.name)
        }
        Some(SearchSelection::Extension(plugin)) => println!(
            "Selected extension result: {} @ {}",
            plugin.mode.id(),
            plugin.index
        ),
        None => println!("No selection"),
    }
    Ok(())
}
