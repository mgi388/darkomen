mod decoder;
mod lexer;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use rand::{seq::SliceRandom as _, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use decoder::{DecodeError, Decoder};

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Packet {
    /// The name of the packet, e.g. `WaterFallingTears`.
    pub name: String,
    /// A map of SFX IDs to SFX.
    pub sfxs: HashMap<SfxId, Sfx>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Sfx {
    /// The ID of the SFX.
    pub id: SfxId,
    /// The name of the SFX, e.g. `Waterfall`.
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
/// SFX IDs are not unique across packets, e.g. SFX ID 0 exists in every packet.
pub type SfxId = u8;

#[repr(u8)]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub enum SfxType {
    #[default]
    One,
    Two,
    Three,
    Four,
    Five,
    /// Type 6 seems to be used for SFX that have multiple sounds which are
    /// randomly picked from.
    ///
    /// TODO: For cases where type is 6, none of the individual sounds are
    /// looped but the SFX as a whole is looped. So, it's possible that "loop
    /// SFX" is managed in flags. Type 6 has flags either 0 or 2. All of those
    /// with flags 0 are in MEET.H.
    Six,
}

impl From<SfxType> for u8 {
    fn from(sfx_type: SfxType) -> Self {
        sfx_type as u8
    }
}

impl TryFrom<u8> for SfxType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(SfxType::One),
            2 => Ok(SfxType::Two),
            3 => Ok(SfxType::Three),
            4 => Ok(SfxType::Four),
            5 => Ok(SfxType::Five),
            6 => Ok(SfxType::Six),
            _ => Err(()),
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct SfxFlags: u8 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const UNKNOWN_FLAG_2 = 1 << 1;
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Sound {
    /// The file name of the sound excluding the path and extension, i.e. the
    /// stem of the file name, e.g. `watfal02`.
    pub file_stem: String,
    /// The frequency of the sound.
    pub frequency: u32,
    /// The frequency deviation of the sound.
    pub frequency_deviation: u32,
    /// The volume of the sound.
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
    /// file's sample rate, e.g. 44100, and the sound's frequency and frequency
    /// deviation.
    ///
    /// A playback rate of 1.0 means the sound is played at its original
    /// frequency. A playback rate of 2.0 means the sound is played at twice its
    /// original frequency.
    pub fn random_playback_rate(&self, rng: &mut impl Rng, sample_rate: u32) -> f64 {
        // Calculate the base playback rate from the frequency and sample rate.
        let base_playback_rate = self.frequency as f64 / sample_rate as f64;

        if self.frequency_deviation == 0 {
            return base_playback_rate;
        }

        let random_frequency_deviation = rng.gen_range(0..self.frequency_deviation);

        // Adjust the playback rate by the random frequency deviation.
        base_playback_rate
            * (self.frequency as f64 / (self.frequency as f64 + random_frequency_deviation as f64))
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
