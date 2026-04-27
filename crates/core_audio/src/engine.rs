/// Opaque handle returned by [`AudioEngine::play_sound`].
pub type SoundId = u32;

/// Trait abstracting over audio backends.
///
/// Implementors must be [`Send`] so the engine can live on a dedicated thread.
pub trait AudioEngine: Send {
    /// Play a one-shot sound effect and return a handle to it.
    fn play_sound(&mut self, path: &str) -> SoundId;

    /// Stop a currently-playing sound effect.
    fn stop_sound(&mut self, id: SoundId);

    /// Adjust the volume of an individual sound effect (`0.0` – `1.0`).
    fn set_sound_volume(&mut self, id: SoundId, volume: f32);

    /// Start playing background music (replaces any current track).
    fn play_music(&mut self, path: &str);

    /// Stop the current background music.
    fn stop_music(&mut self);

    /// Set the music volume (`0.0` – `1.0`).
    fn set_music_volume(&mut self, volume: f32);

    /// Set the master volume that scales all output (`0.0` – `1.0`).
    fn set_master_volume(&mut self, volume: f32);

    /// Set the listener position for spatial audio.
    fn set_listener_position(&mut self, position: glam::Vec3);

    /// Called once per frame to drive internal updates.
    fn update(&mut self);
}
