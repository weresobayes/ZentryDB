use chrono::Utc;
use zentry_db::{
    db::Ledger,
    System, ConversionGraph,
    install,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database installation
    install::install()?;

    // Initialize ledger
    let mut ledger = Ledger::load_from_disk()?;

    // Create currency systems
    let idr_system = System {
        id: "IDR".to_string(),
        description: "Indonesian Rupiah".to_string(),
    };
    ledger.create_system(idr_system)?;

    let usd_system = System {
        id: "USD".to_string(),
        description: "United States Dollar".to_string(),
    };
    ledger.create_system(usd_system)?;

    // Create conversion graph
    let idr_to_usd = ConversionGraph {
        graph: "USD -> IDR".to_string(),
        rate: 14000.0,
        rate_since: Utc::now(),
    };
    ledger.create_conversion_graph(idr_to_usd)?;

    ledger.conversion_graphs.iter().for_each(|(uuid, graph)| {
        println!("Conversion Graph: {:?} - {} {}", uuid, graph.graph, graph.rate_since);
    });

    ledger.systems.iter().for_each(|(uuid, system)| {
        println!("System: {:?} - {}", uuid, system.id);
    });

    ledger.persist_indexes()?;

    Ok(())
}
