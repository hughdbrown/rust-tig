use anyhow::Result;
use rust_tig::ui::{self, App, EventHandler};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize panic handler for better error messages
    std::panic::set_hook(Box::new(|panic_info| {
        // Restore terminal before showing panic
        let _ = ui::terminal::restore();
        eprintln!("{}", panic_info);
    }));

    // Run the application
    if let Err(e) = run().await {
        ui::terminal::restore()?;
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

async fn run() -> Result<()> {
    // Initialize terminal
    let mut terminal = ui::terminal::init()?;

    // Create application and event handler
    let mut app = App::new();

    // Initialize the app with the repository
    if let Err(e) = app.init().await {
        ui::terminal::restore()?;
        eprintln!("Failed to initialize: {}", e);
        return Ok(()); // Exit gracefully
    }

    let mut event_handler = EventHandler::new();
    event_handler.start();

    // Main event loop
    while app.is_running() {
        // Update application state
        app.update()?;

        // Render the UI
        terminal.draw(|frame| {
            app.render(frame);
        })?;

        // Handle events
        if let Some(event) = event_handler.next().await {
            app.handle_event(event)?;
        }
    }

    // Restore terminal
    ui::terminal::restore()?;

    Ok(())
}
