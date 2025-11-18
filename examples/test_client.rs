use inferenco_mcp::server::{DiceArgs, EchoArgs, ReverseArgs, ToolService};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let service = ToolService::new();

    println!("Available tools:");
    for tool in service.available_tools() {
        let description = tool.description.as_deref().unwrap_or("No description");
        println!(" - {}: {}", tool.name, description);
    }

    // Call the echo tool directly with Parameters<EchoArgs>
    let echo = service
        .echo(Parameters(EchoArgs {
            message: "Hello from the Inferenco MCP example!".into(),
        }))
        .await?;
    println!("echo -> {:?}", echo.content);

    // Call the increment tool to demonstrate stateful behavior
    let increment = service.increment().await?;
    println!("increment -> {:?}", increment.content);

    // Reverse a string
    let reversed = service
        .reverse_text(Parameters(ReverseArgs {
            text: "Inferenco MCP".into(),
        }))
        .await?;
    println!("reverse_text -> {:?}", reversed.content);

    // Get the current UTC time
    let clock = service.current_time().await?;
    println!("current_time -> {:?}", clock.content);

    // Roll a 20-sided die
    let roll = service
        .roll_dice(Parameters(DiceArgs { sides: 20 }))
        .await?;
    println!("roll_dice -> {:?}", roll.content);

    Ok(())
}
