use minirust_core::{AppError, Echo, EchoInput};

use crate::cqrs::{Command, CommandHandler};

/// Intent to execute the echo operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoCommand {
    pub message: String,
}

impl Command for EchoCommand {
    type Output = Echo;
    type Error = AppError;
}

/// Result produced by [`EchoCommandHandler`].
pub type EchoCommandResult = Echo;

/// Handles the echo command without depending on HTTP or infrastructure.
#[derive(Debug, Clone, Copy, Default)]
pub struct EchoCommandHandler;

impl CommandHandler<EchoCommand> for EchoCommandHandler {
    fn handle(&self, command: EchoCommand) -> Result<EchoCommandResult, AppError> {
        let input = EchoInput::parse(command.message)?;
        Ok(Echo::new(input.message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handler_executes_a_valid_command() {
        let result = EchoCommandHandler.handle(EchoCommand {
            message: "hello".to_owned(),
        });

        assert_eq!(result.unwrap().echo, "hello");
    }

    #[test]
    fn handler_rejects_an_invalid_command() {
        let result = EchoCommandHandler.handle(EchoCommand {
            message: "   ".to_owned(),
        });

        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
