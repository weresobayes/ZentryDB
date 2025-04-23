use chrono::Utc;
use colored::*;
use rustyline::DefaultEditor;

use tabled::Table;
use zentry_db::{
    db::Ledger,
    install,
    model::{System, ConversionGraph},
    interface::cli::ConversionGraphRow,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Database installation
    install::install()?;

    // Initialize ledger
    let mut ledger = Ledger::load_from_disk()?;

    let mut r1 = DefaultEditor::new()?;

    println!();
    println!();

    println!();
    println!();

    println!(
    "{}",
    r#"
                xX+.   x:  .+  ;;                                                         
                    ;X$$X++: ;+: ;+                                                        
            .             +;:+$$;..;x+:;                      .                           
                    +XXx+++Xx.  :X+:  :x++                                                
                        +;:+X::x  ;x+.   .;:                                             
                    +xxxx;:::+:.+x;;:  ;xx:+;::                                 ...        
                    +;::.::;+;..+:.     :+x+;:                                           
    .       .     ;+x:: :::;;x$X+xX+:::::     ;;:;               .  .    .         .    . 
        . .    ;+x+. ;.;;:;+ .;;;x$$$XXXXX+. .+x     .      . .     ...                 
    ...         +; +;: .::;:x        ..     ;+xX+               .   .            . .....  
        ... ...    ;+;.;. ;;.+           .   +;                          .   .. .          
    .   .        :+x+;:.:.;.:x.     ..      ;        .   ;+:   ;;;                . ..... 
    ........... :+ ++::: ::..:+;                  ++ .+x+;;xxx+;:;++++:    ..... ....... 
    ..   . . ....   ;x;;::..;;  .;+:    ...    ; :++;;::    .   ..::;;;::.      ... ....: 
    ...:........ ... ;X+:;:.::+;.  :;;.      ;+++;.: ..  . :... .:.    ;+x+;;.    ..... . 
    ::.:.:. ::.:    ..;;x;;::::.:;:  .;;. ;xx..;:;;:.:::::::::::..:..:.. :;;:   . ....... 
    :.:.:::   ;: : .:; :Xx;;+;;. :;;.  :;. :  :x;  ;;;;;+xxx++x+x+;;.::;.:.:++++. :::...: 
    .::.    ;;:: :::;    Xxx;;x;:::;:.. .+ ;::    +;;+x+x+;::.:;++x+++::.....;; ;.:....:: 
    ::::::    ;+ .;;    ..     +X;:;;;;;++: +    ;+++++:           ;++++:::.::+;:  ..::.. 
    :...:;. .::;; ;.   :         +;+++;     .:.  ;+++.   .....:...   ;x;;::..::x;+.:..::: 
    :::::.;   ::+;::  .;         .xx;       .;  x$$x:.::.:...........  x+;..;::+;  .:.:.. 
    .:::;:;;+   .;;;  .:     . .  X.        : +X+      ... ....::::::: :+;;:.::++;.:...:. 
    :::     .;+:   :++ :  ... .  ;;      .:;++.      ....;+;::;;....::. x;:.:::x;+ :::::. 
    ;:       .:;++xxxXX$XX;    .+.   ::+xx++:;.   .Xxxxxx++++++xx;:.::.x;:::::;+::::::::. 
    ::::;+;.     :    . :;+xx+xxxXXXx++:        x$Xx++++;+;;;;;:++xx++x;.:;:;;+x:.::::::: 
    :::::;;++x+;+;       .    +  :::::;;+x$$$XX+;::::..:;;+++x+++++;:...::;;++x:.:::::::: 
    ;;;;;;:;;;++++         .xXx+            x+x+xx+;;;;::      ..   .:::++;++;..::::::::: 
    ;:::;;;;+++;++:+xxXxXX$X++++xXxx;;++xx; Xxxxxx++xX+++;+;;:;::::;;;++++;++:;;;;;;;:::: 
    ;;;:::::;:::;:;;;;;;;;;;::;;+;++xxx+++x;;::.:::;:;+++xxxxxxxxxXxxxx+++;;::;:;.::::::: 
    ;;;:::::;:::;:;;;;;;;;;;::;;+;++xxx+++x;;::.:::;:;+++xxxxxxxxxXxxxx+++;;::;:;.::::::: 
    ;:       .:;++xxxXX$XX;    .+.   ::+xx++:;.   .Xxxxxx++++++xx;:.::.x;:::::;+::::::::. 
        ... ...    ;+;.;. ;;.+           .   +;                          .   .. .          
                          ____           __             ___  ___ 
                         /_  / ___ ___  / /_______ __  / _ \/ _ )
                          / /_/ -_) _ \/ __/ __/ // / / // / _  |
                         /___/\__/_//_/\__/_/  \_, / /____/____/ 
                                              /___/              
    "#
    .cyan()
    );
                                                                                       

    println!();
    println!();

    println!("{}", "Entering Zentry Zone. Type 'help' for commands, 'exit' to quit".bold());

    println!();
    println!();

    loop {
        let readline = r1.readline("zentry> ");
        match readline {
            Ok(line) => {
                let input = line.trim();
                if input == "exit" {
                    break;
                } else if input == "help" {
                    println!("{}", "Commands:".cyan().bold());
                    println!("{}", "  system <id> <desc>                                        - Create a currency system".cyan());
                    println!("{}", "  conv <system1> <relation> <system2> <rate> [<rate since>] - Add a conversion graph".cyan());
                    println!("{}", "  show systems                                              - List all systems".cyan());
                    println!("{}", "  show conversions                                          - List all conversion graphs".cyan());
                    println!("{}", "  exit                                                      - Quit".cyan());
                } else if let Some(rest) = input.strip_prefix("system ") {
                    let mut parts = rest.splitn(2, ' ');

                    if  let (Some(id), Some(desc)) = (parts.next(), parts.next()) {
                        let system = System {
                            id: id.to_string(),
                            description: desc.to_string(),
                        };
                        match ledger.create_system(system) {
                            Ok(_) => println!("System created successfully"),
                            Err(e) => println!("Error creating system: {}", e),
                        }
                        continue;
                    } else {
                        println!("Invalid command format. Type 'help' for list of commands");
                    }
                } else if let Some(rest) = input.strip_prefix("conv ") {
                    let mut parts = rest.split_whitespace();

                    let system1 = parts.next();
                    let relation = parts.next();
                    let system2 = parts.next();
                    let rate_str = parts.next();
                    let rate_since_str = parts.next();

                    match (system1, relation, system2, rate_str) {
                        (Some(system1), Some(relation), Some(system2), Some(rate_str)) => {
                            let rate = match rate_str.parse() {
                                Ok(rate) => rate,
                                Err(_) => {
                                    println!("Invalid `rate` format. Use a number");
                                    continue;
                                }
                            };

                            let rate_since = if let Some(since_str) = rate_since_str {
                                match since_str.parse() {
                                    Ok(since) => since,
                                    Err(_) => {
                                        println!("Invalid `rate_since` format. Use RFC3339 format (e.g., 2023-04-23T12:00:00Z)");
                                        continue;
                                    }
                                }
                            } else {
                                Utc::now()
                            };

                            let conversion_graph = ConversionGraph {
                                graph: format!("{} {} {}", system1, relation, system2),
                                rate,
                                rate_since,
                            };

                            match ledger.create_conversion_graph(conversion_graph) {
                                Ok(_) => println!("Conversion graph added successfully"),
                                Err(e) => {
                                    println!("Error adding conversion graph");
                                    println!("  {}", e);
                                },
                            }
                            continue;
                        }
                        _ => {
                            println!("Invalid command format. Type 'help' for list of commands");
                            continue;
                        }
                    }
                } else if let Some(rest) = input.strip_prefix("show ") {
                    match rest.trim() {
                        "systems" => {
                            println!("{}", "Function is in development".cyan());
                            continue;
                        }
                        "conversions" => {
                            let conversion_graphs = ledger.conversion_graphs.clone();
                            let rows: Vec<ConversionGraphRow> = conversion_graphs.iter().map(|(_, conversion_graph)| ConversionGraphRow {
                                graph: conversion_graph.graph.clone(),
                                rate: conversion_graph.rate,
                                rate_since: conversion_graph.rate_since,
                            }).collect();

                            let table = Table::new(rows);
                            println!("{}", table);
                            continue;
                        }
                        _ => {
                            println!("Invalid command format. Type 'help' for list of commands");
                            continue;
                        }
                    }
                } else {
                    println!("Unknown command: '{}'. Type 'help' for list of commands", input);
                }
            },
            Err(e) => {
                println!("Error: {}", e);
                break;
            }
        }
    }

    ledger.persist_indexes()?;
    Ok(())
}
