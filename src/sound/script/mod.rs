mod decoder;
mod encoder;
mod lexer;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use indexmap::IndexMap;
use serde::Serialize;
use std::ops::Index;

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Script {
    /// A map of state IDs to number. The purpose of the number is unknown and
    /// does not appear to be required to play a script.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub states: IndexMap<StateId, i32>,
    /// The state to use when the script starts.
    ///
    /// Start state is probably required but provided as an [`Option`] to
    /// allow the decoder to gracefully decode. It's up to callers to decide
    /// how to handle a missing start state.
    pub start_state: Option<StateId>,
    /// The pattern to use when the script starts.
    ///
    /// Start pattern is probably required but provided as an [`Option`] to
    /// allow the decoder to gracefully decode. It's up to callers to decide
    /// how to handle a missing start pattern.
    pub start_pattern: Option<PatternId>,
    /// A map of sample IDs to sample file names.
    ///
    /// The file name is partial is without the path and extension, e.g.
    /// `mDdumchr1a`.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub samples: IndexMap<SampleId, String>,
    /// A map of pattern IDs to patterns.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub patterns: IndexMap<PatternId, Pattern>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Pattern {
    /// A list of sequences to choose from when playing the pattern. Patterns
    /// support multiple sequences to make the music more dynamic. Callers can
    /// choose a sequence at random or based on some other criteria. Randomizing
    /// the sequence affects the list of samples that are played.
    pub sequences: Vec<Sequence>,
    /// A list of state tables to choose from when playing the pattern. Patterns
    /// support multiple state tables to make the music more dynamic. Callers
    /// can choose a state table at random or based on some other criteria.
    /// Randomizing the state table affects the next pattern to play.
    #[cfg_attr(feature = "bevy_reflect", reflect(ignore))]
    pub state_tables: Vec<StateTable>,
}

pub type StateId = String;

pub fn default_state_id() -> StateId {
    "default".to_string()
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct PatternId(String);

impl PatternId {
    pub fn new(value: String) -> Self {
        let mut chars = value.chars();
        let value = match chars.next() {
            None => String::new(),
            Some(f) => f.to_lowercase().collect::<String>() + chars.as_str(),
        };

        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub fn end_pattern_id() -> PatternId {
    PatternId::new("end".to_string())
}

pub type SampleId = String;

/// A sequence of samples to play in order.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Sequence(pub(crate) Vec<SampleId>);

impl Index<usize> for Sequence {
    type Output = SampleId;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<'a> IntoIterator for &'a Sequence {
    type Item = &'a SampleId;
    type IntoIter = std::slice::Iter<'a, SampleId>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Sequence {
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StateTable(pub(crate) IndexMap<StateId, PatternId>);

impl StateTable {
    pub fn get(&self, state: &StateId) -> Option<&PatternId> {
        self.0.get(state)
    }
}
