use crate::engine::{AudioEngine, SoundId};

/// A no-op audio engine that logs every call via [`tracing::debug!`].
///
/// Useful for headless servers, tests, and CI where no audio device is present.
pub struct StubAudioEngine {
    next_id: SoundId,
    master_volume: f32,
    music_volume: f32,
}

impl StubAudioEngine {
    /// Create a new stub engine with default volumes of `1.0`.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            master_volume: 1.0,
            music_volume: 1.0,
        }
    }
}

impl Default for StubAudioEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioEngine for StubAudioEngine {
    fn play_sound(&mut self, path: &str) -> SoundId {
        let id = self.next_id;
        self.next_id += 1;
        tracing::debug!(path, id, "stub: play_sound");
        id
    }

    fn stop_sound(&mut self, id: SoundId) {
        tracing::debug!(id, "stub: stop_sound");
    }

    fn set_sound_volume(&mut self, id: SoundId, volume: f32) {
        tracing::debug!(id, volume, "stub: set_sound_volume");
    }

    fn play_music(&mut self, path: &str) {
        tracing::debug!(path, "stub: play_music");
    }

    fn stop_music(&mut self) {
        tracing::debug!("stub: stop_music");
    }

    fn set_music_volume(&mut self, volume: f32) {
        self.music_volume = volume;
        tracing::debug!(volume, "stub: set_music_volume");
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume;
        tracing::debug!(volume, "stub: set_master_volume");
    }

    fn set_listener_position(&mut self, position: glam::Vec3) {
        tracing::debug!(?position, "stub: set_listener_position");
    }

    fn update(&mut self) {
        tracing::debug!("stub: update");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_stub() {
        let engine = StubAudioEngine::new();
        assert_eq!(engine.next_id, 1);
        assert_eq!(engine.master_volume, 1.0);
        assert_eq!(engine.music_volume, 1.0);
    }

    #[test]
    fn play_sound_returns_incrementing_ids() {
        let mut engine = StubAudioEngine::new();
        let id1 = engine.play_sound("sfx/hit.ogg");
        let id2 = engine.play_sound("sfx/miss.ogg");
        let id3 = engine.play_sound("sfx/crit.ogg");
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn all_methods_callable_without_panic() {
        let mut engine = StubAudioEngine::new();
        let id = engine.play_sound("sfx/test.ogg");
        engine.stop_sound(id);
        engine.set_sound_volume(id, 0.5);
        engine.play_music("music/battle.ogg");
        engine.stop_music();
        engine.set_music_volume(0.8);
        engine.set_master_volume(0.9);
        engine.set_listener_position(glam::Vec3::new(1.0, 2.0, 3.0));
        engine.update();
    }
}
