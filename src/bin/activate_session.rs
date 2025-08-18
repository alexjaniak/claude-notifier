use claude_notifier::session_store::SessionStore;
use claude_notifier::terminal_detector;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        eprintln!("Usage: {} <session_id>", args[0]);
        eprintln!("\nThis tool activates the terminal window for a given Claude session.");
        eprintln!("\nAvailable sessions:");
        
        let store = SessionStore::new();
        for session_id in store.list_sessions() {
            if let Some(session) = store.get_session(&session_id) {
                eprintln!("  {} - {:?} ({})", 
                    session_id, 
                    session.terminal_info.terminal_app,
                    session.cwd.as_deref().unwrap_or("unknown dir")
                );
            }
        }
        std::process::exit(1);
    }
    
    let session_id = &args[1];
    let store = SessionStore::new();
    
    match store.get_session(session_id) {
        Some(session) => {
            println!("Found session: {}", session_id);
            println!("Terminal: {:?}", session.terminal_info.terminal_app);
            println!("Directory: {:?}", session.cwd);
            
            match terminal_detector::activate_terminal(&session.terminal_info) {
                Ok(_) => {
                    println!("✓ Terminal activated successfully!");
                }
                Err(e) => {
                    eprintln!("✗ Failed to activate terminal: {}", e);
                    std::process::exit(1);
                }
            }
        }
        None => {
            eprintln!("Session '{}' not found", session_id);
            eprintln!("\nAvailable sessions:");
            for sid in store.list_sessions() {
                eprintln!("  {}", sid);
            }
            std::process::exit(1);
        }
    }
}