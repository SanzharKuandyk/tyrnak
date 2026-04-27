use tracing::info;

/// Represents the current state of the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Init,
    Running,
    Paused,
    ShuttingDown,
}

/// Application lifecycle state machine.
pub struct AppLifecycle {
    state: AppState,
}

impl AppLifecycle {
    /// Create a new lifecycle in the [`AppState::Init`] state.
    pub fn new() -> Self {
        Self {
            state: AppState::Init,
        }
    }

    /// Returns the current state.
    pub fn state(&self) -> AppState {
        self.state
    }

    /// Transition from [`AppState::Init`] to [`AppState::Running`].
    ///
    /// No-op if not currently in the `Init` state.
    pub fn start(&mut self) {
        if self.state == AppState::Init {
            info!("AppLifecycle: Init -> Running");
            self.state = AppState::Running;
        } else {
            tracing::warn!(
                "AppLifecycle: start() called in {:?} state, ignoring",
                self.state
            );
        }
    }

    /// Transition from [`AppState::Running`] to [`AppState::Paused`].
    ///
    /// No-op if not currently in the `Running` state.
    pub fn pause(&mut self) {
        if self.state == AppState::Running {
            info!("AppLifecycle: Running -> Paused");
            self.state = AppState::Paused;
        } else {
            tracing::warn!(
                "AppLifecycle: pause() called in {:?} state, ignoring",
                self.state
            );
        }
    }

    /// Transition from [`AppState::Paused`] to [`AppState::Running`].
    ///
    /// No-op if not currently in the `Paused` state.
    pub fn resume(&mut self) {
        if self.state == AppState::Paused {
            info!("AppLifecycle: Paused -> Running");
            self.state = AppState::Running;
        } else {
            tracing::warn!(
                "AppLifecycle: resume() called in {:?} state, ignoring",
                self.state
            );
        }
    }

    /// Transition to [`AppState::ShuttingDown`] from any state.
    pub fn shutdown(&mut self) {
        if self.state != AppState::ShuttingDown {
            info!("AppLifecycle: {:?} -> ShuttingDown", self.state);
            self.state = AppState::ShuttingDown;
        } else {
            tracing::warn!("AppLifecycle: shutdown() called while already ShuttingDown, ignoring");
        }
    }

    /// Returns `true` if the current state is [`AppState::Running`].
    pub fn is_running(&self) -> bool {
        self.state == AppState::Running
    }

    /// Returns `true` if the application should quit (state is [`AppState::ShuttingDown`]).
    pub fn should_quit(&self) -> bool {
        self.state == AppState::ShuttingDown
    }
}

impl Default for AppLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_init() {
        let lifecycle = AppLifecycle::new();
        assert_eq!(lifecycle.state(), AppState::Init);
        assert!(!lifecycle.is_running());
        assert!(!lifecycle.should_quit());
    }

    #[test]
    fn init_to_running() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        assert_eq!(lifecycle.state(), AppState::Running);
        assert!(lifecycle.is_running());
    }

    #[test]
    fn running_to_paused() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.pause();
        assert_eq!(lifecycle.state(), AppState::Paused);
        assert!(!lifecycle.is_running());
    }

    #[test]
    fn paused_to_running() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.pause();
        lifecycle.resume();
        assert_eq!(lifecycle.state(), AppState::Running);
        assert!(lifecycle.is_running());
    }

    #[test]
    fn shutdown_from_any_state() {
        // From Init
        let mut lifecycle = AppLifecycle::new();
        lifecycle.shutdown();
        assert_eq!(lifecycle.state(), AppState::ShuttingDown);
        assert!(lifecycle.should_quit());

        // From Running
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.shutdown();
        assert_eq!(lifecycle.state(), AppState::ShuttingDown);

        // From Paused
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.pause();
        lifecycle.shutdown();
        assert_eq!(lifecycle.state(), AppState::ShuttingDown);
    }

    #[test]
    fn start_from_running_is_noop() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.start(); // should be a no-op
        assert_eq!(lifecycle.state(), AppState::Running);
    }

    #[test]
    fn pause_from_init_is_noop() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.pause(); // should be a no-op
        assert_eq!(lifecycle.state(), AppState::Init);
    }

    #[test]
    fn resume_from_running_is_noop() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.resume(); // should be a no-op
        assert_eq!(lifecycle.state(), AppState::Running);
    }

    #[test]
    fn shutdown_twice_is_noop() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.shutdown();
        lifecycle.shutdown(); // should be a no-op
        assert_eq!(lifecycle.state(), AppState::ShuttingDown);
    }

    #[test]
    fn start_from_paused_is_noop() {
        let mut lifecycle = AppLifecycle::new();
        lifecycle.start();
        lifecycle.pause();
        lifecycle.start(); // should be a no-op, only Init -> Running
        assert_eq!(lifecycle.state(), AppState::Paused);
    }
}
