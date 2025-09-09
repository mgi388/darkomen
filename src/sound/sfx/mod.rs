mod decoder;
mod lexer;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use rand::{seq::IndexedRandom as _, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use decoder::{DecodeError, Decoder};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Packet {
    /// The name of the packet, e.g., `WaterFallingTears`.
    pub name: String,
    /// A map of SFX IDs to SFX.
    pub sfxs: HashMap<SfxId, Sfx>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Sfx {
    /// The ID of the SFX.
    pub id: SfxId,
    /// The name of the SFX, e.g., `Waterfall`.
    pub name: String,
    /// The priority of the SFX.
    pub priority: u8,
    /// The type of SFX.
    pub typ: SfxType,
    /// The SFX flags.
    pub flags: SfxFlags,
    /// The sounds that make up the SFX.
    pub sounds: Vec<Sound>,
}

impl Sfx {
    /// Returns a random sound from the SFX.
    pub fn random_sound(&self, rng: &mut impl Rng) -> Option<&Sound> {
        self.sounds.choose(rng)
    }
}

/// The ID of a SFX.
///
/// SFX IDs are 0-based, not 1-based, so the first SFX in a packet has an ID of
/// 0, the second SFX has an ID of 1, and so on.
///
/// SFX IDs are unique within a packet.
///
/// SFX IDs are not unique across packets, e.g., SFX ID 0 exists in every
/// packet.
pub type SfxId = u8;

#[repr(u8)]
#[derive(
    Clone, Debug, Default, Deserialize, IntoPrimitive, PartialEq, Serialize, TryFromPrimitive,
)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, PartialEq, Serialize)
)]
pub enum SfxType {
    /// A sound effect that plays one sound and does not loop.
    ///
    /// Only one sound is played for this type of sound effect. If the sound
    /// effect contains more than one sound, the others are ignored.
    ///
    /// Note: In the original game, all sound effects of this type have a single
    /// sound except `SFX_BLADEWINDHIT` from `BATUND.H`, which has 2 sounds but
    /// the second sound is always ignored.
    ///
    /// Used for the "button down" (`SFX_BUTTONDOWN`) sound effect from
    /// `INTAFACE.H` in the original game.
    #[default]
    OneShot = 1,
    /// A sound effect that plays multiple sounds simultaneously without
    /// looping.
    ///
    /// Any sound can have its `loop` set to true, and doing so causes that
    /// sound to be looped indefinitely, even while other sounds are playing.
    ///
    /// Used for the "horn of Urgok" (`SFX_HORNURGOK`) sound effect from
    /// `BATGEN.H` in the original game.
    SimultaneousOneShot = 2,
    /// A sound effect that randomly selects one sound from a list to play,
    /// without looping.
    ///
    /// Used for the "arrows being fired" (`SFX_ARROWS`) and "arrows hitting a
    /// target" (`SFX_ARROWHIT`) sound effects where one sound is randomly
    /// picked from the list. Note: These sound effects both have `!Null` as
    /// their middle sound. This is probably used to reduce the number of sounds
    /// played when a regiment fires a volley of arrows. However, theoretically,
    /// the sound effect could be skipped entirely if the middle sound is picked
    /// for every unit firing an arrow in the regiment / every arrow hitting a
    /// target.
    ///
    /// They all have flags equal to 2 except for `SFX_NEXTPAGE` in `GLUE.H`
    /// which has flags equal to 1. This has two sounds `paper1` and `paper2`
    /// and so it seems like the game is picking between two paper sounds to
    /// make the page turning sound less repetitive. This further supports that
    /// flags equal to 2 means "spatial".
    RandomOneShot = 3,
    /// A sound effect that plays a sequence of sounds one after another,
    /// without looping.
    ///
    /// If any sound in the sequence is set to loop, subsequent sounds will not
    /// play.
    ///
    /// Used for the "steam tank" (`SFX_STEAMWHISTLECOOL`) sound effect from
    /// `BATALL.H` in the original game.
    SequentialNonLooping = 4,
    /// A sound effect that plays a sequence of sounds one after another, with
    /// the sequence looping.
    ///
    /// If any sound in the sequence is set to loop, the entire sound effect
    /// will not play.
    ///
    /// Used for the "fireworks" (`SFX_FIREWORKS`) sound effect from
    /// `FIREWORK.H` in the original game.
    SequentialLooping = 5,
    /// A sound effect that randomly selects one sound from a list to play, with
    /// the selection looping.
    ///
    /// Used for the "birds" (`SFX_TWITTERLOOP`) sound effect from `TWITTER.H`
    /// where one sound is randomly picked from the list, it is played, then the
    /// sound effect loops and randomly picks another sound from the list, and
    /// so on.
    ///
    /// The sound effects that have this type have flags equal to either 0 or 2.
    /// All of those with flags equal to 0 are in `MEET.H`. It seems like flags
    /// equal to 2 could mean "spatial" sound effect, but there are some sounds
    /// that have this flag that don't seem to be affected by the position of
    /// the camera, i.e., they don't seem to be spatial. Changing a flag from 2
    /// to 0 also keeps the sound as spatial, but changing it from 2 to 1 makes
    /// it global, so it seems like flags equal to 1 means "global" and flags
    /// equal to 0 or 2 means "spatial".
    RandomLooping = 6,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct SfxFlags: u8 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const UNKNOWN_FLAG_2 = 1 << 1;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Sound {
    /// The file name of the sound excluding the path and extension, i.e., the
    /// stem of the file name, e.g., `watfal02`.
    pub file_stem: String,
    /// The frequency of the sound.
    pub frequency: u32,
    /// The frequency deviation of the sound.
    pub frequency_deviation: u32,
    /// The volume of the sound from 0 to 255.
    pub volume: u8,
    /// Whether the sound loops.
    pub looped: bool,
    /// The attack of the sound.
    pub attack: i8,
    /// The release of the sound.
    pub release: i8,
}

impl Sound {
    /// Returns a random playback rate for the sound.
    ///
    /// The playback rate is calculated dynamically based on the source audio
    /// file's sample rate, e.g., 44100, and the sound's frequency and frequency
    /// deviation.
    ///
    /// A playback rate of 1.0 means the sound is played at its original
    /// frequency. A playback rate of 2.0 means the sound is played at twice its
    /// original frequency.
    pub fn random_playback_rate(&self, rng: &mut impl Rng, sample_rate: u32) -> f32 {
        let frequency = self.frequency as f32;

        // Calculate the base playback rate from the frequency and sample rate.
        let base_playback_rate = frequency / sample_rate as f32;

        if self.frequency_deviation == 0 {
            return base_playback_rate;
        }

        let random_frequency_deviation = rng.random_range(0..self.frequency_deviation);

        // Adjust the playback rate by the random frequency deviation.
        base_playback_rate * (frequency / (frequency + random_frequency_deviation as f32))
    }

    /// Returns the volume as a linear value from 0.0 to 1.0.
    pub fn linear_volume(&self) -> f32 {
        self.volume as f32 / 255.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn deterministic_rand() -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(42)
    }

    #[test]
    fn test_appear01_playback_rate() {
        let mut rng = deterministic_rand();
        let sound = Sound {
            frequency: 44_100, // 44.1 kHz is from the sound effect packet file
            frequency_deviation: 0,
            ..Default::default()
        };

        let playback_rate = sound.random_playback_rate(&mut rng, 16_000); // 16 kHz is the sample rate of APPEAR01.WAV

        assert_eq!(playback_rate, 2.75625);
    }

    #[test]
    fn test_random_playback_rate() {
        let mut rng = deterministic_rand();
        let sound = Sound {
            frequency: 22_050,
            frequency_deviation: 100,
            ..Default::default()
        };

        let playback_rate = sound.random_playback_rate(&mut rng, 44_100);

        assert!(
            (0.0..=1.0).contains(&playback_rate),
            "Playback rate out of range"
        );
    }
}
