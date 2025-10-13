use frz::{FacetRow, FileRow, SearchData, SearchMode, SearchSelection, SearchUi, UiConfig};

fn main() -> anyhow::Result<()> {
    // Build sample data
    let facets = vec![FacetRow::new("frontend", 3), FacetRow::new("backend", 2)];
    let files = vec![
        FileRow::new("src/main.rs", ["frontend"]),
        FileRow::new("src/lib.rs", ["backend"]),
    ];

    let data = SearchData::new()
        .with_context("example/repo")
        .with_initial_query("")
        .with_facets(facets)
        .with_files(files);

    // Minimal search UI configuration with prompt
    let search_ui = SearchUi::new(data)
        .with_ui_config(UiConfig::tags_and_files())
        .with_input_title("workspace-prototype")
        .with_start_mode(SearchMode::Facets);
    let outcome = search_ui.run()?;
    println!("Accepted? {}", outcome.accepted);
    match outcome.selection {
        Some(SearchSelection::File(file)) => println!("Selected file: {}", file.path),
        Some(SearchSelection::Facet(facet)) => println!("Selected facet: {}", facet.name),
        None => println!("No selection"),
    }
    Ok(())
}
