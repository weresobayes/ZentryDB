use zentry_db::{
    db::Ledger, install
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database installation
    install::install()?;

    // Initialize ledger
    let ledger = Ledger::load_from_disk()?;

    ledger.conversion_graphs.iter().for_each(|(uuid, graph)| {
        println!("Conversion Graph: {:?} - {} {}", uuid, graph.graph, graph.rate_since);
    });

    ledger.systems.iter().for_each(|(uuid, system)| {
        println!("System: {:?} - {}", uuid, system.id);
    });

    Ok(())
}
