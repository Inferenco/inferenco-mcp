use rmcp::schemars;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EchoArgs {
    pub message: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReverseArgs {
    pub text: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DiceArgs {
    #[serde(default = "DiceArgs::default_sides")]
    pub sides: u8,
}

impl DiceArgs {
    const fn default_sides() -> u8 {
        6
    }
}
