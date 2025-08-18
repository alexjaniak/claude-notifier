use claude_notifier::terminal_detector::TerminalInfo;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());
    
    if verbose {
        println!("Detecting terminal information...\n");
    }
    
    let info = TerminalInfo::detect();
    
    println!("Terminal: {:?}", info.terminal_app.as_deref().unwrap_or("Unknown"));
    
    if verbose {
        println!("\n=== Full Detection Results ===");
        println!("Terminal App: {:?}", info.terminal_app);
        println!("Window ID: {:?}", info.window_id);
        println!("Project Dir: {:?}", info.project_dir);
        println!("Parent PID: {:?}", info.parent_pid);
        println!("CWD: {:?}", info.cwd);
        
        println!("\n=== Environment Variables ===");
        println!("TERM_PROGRAM: {:?}", env::var("TERM_PROGRAM").ok());
        println!("CURSOR_TRACE_ID: {:?}", env::var("CURSOR_TRACE_ID").ok());
        println!("GIT_ASKPASS: {:?}", env::var("GIT_ASKPASS").ok());
    }
    
    if args.contains(&"--test-activate".to_string()) {
        println!("\nWaiting 2 seconds then trying to activate terminal...");
        std::thread::sleep(std::time::Duration::from_secs(2));
        
        match claude_notifier::terminal_detector::activate_terminal(&info) {
            Ok(_) => println!("✓ Terminal activated successfully!"),
            Err(e) => println!("✗ Failed to activate terminal: {}", e),
        }
    }
}