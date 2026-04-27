use std::io::{self, BufRead};
use std::thread;

use core_runtime::BrokerTx;

/// Console commands accepted by the headless server shell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsoleCommand {
    Quit,
    Pause,
    Resume,
    Unknown(String),
}

impl ConsoleCommand {
    /// Parse a single console input line.
    pub fn parse(line: &str) -> Option<Self> {
        let normalized = line.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "" => None,
            "quit" | "exit" => Some(Self::Quit),
            "pause" => Some(Self::Pause),
            "resume" | "start" => Some(Self::Resume),
            other => Some(Self::Unknown(other.to_string())),
        }
    }
}

/// Spawn a dedicated stdin reader that forwards parsed commands to the supervisor loop.
pub fn spawn_console_input_thread(command_tx: BrokerTx<ConsoleCommand>) -> thread::JoinHandle<()> {
    thread::Builder::new()
        .name("console-input".to_string())
        .spawn(move || {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                match line {
                    Ok(line) => {
                        if let Some(command) = ConsoleCommand::parse(&line)
                            && command_tx.send(command).is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        })
        .expect("failed to spawn console input thread")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_console_commands() {
        assert_eq!(ConsoleCommand::parse("quit"), Some(ConsoleCommand::Quit));
        assert_eq!(ConsoleCommand::parse("pause"), Some(ConsoleCommand::Pause));
        assert_eq!(
            ConsoleCommand::parse("resume"),
            Some(ConsoleCommand::Resume)
        );
        assert_eq!(
            ConsoleCommand::parse("weird"),
            Some(ConsoleCommand::Unknown("weird".to_string()))
        );
        assert_eq!(ConsoleCommand::parse("   "), None);
    }
}
