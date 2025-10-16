use frz::{FacetRow, FileRow, SearchData, SearchSelection, SearchUi};
use frz::plugins::builtin::FACETS_MODE;

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
        .with_input_title("workspace-prototype")
        .with_start_mode(FACETS_MODE);
    let outcome = search_ui.run()?;
    println!("Accepted? {}", outcome.accepted);
    match outcome.selection {
        Some(SearchSelection::File(file)) => println!("Selected file: {}", file.path),
        Some(SearchSelection::Facet(facet)) => println!("Selected facet: {}", facet.name),
        Some(SearchSelection::Plugin(plugin)) => println!(
            "Selected plugin result: {} @ {}",
            plugin.mode.as_str(),
            plugin.index
        ),
        None => println!("No selection"),
    }
    Ok(())
}
