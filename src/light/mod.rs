mod decoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use glam::Vec3;
use serde::Serialize;

pub use decoder::{DecodeError, Decoder};

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Light {
    pub position: Vec3,
    pub flags: u32,
    pub unknown: u32,
    pub color: Vec3,
}

impl Light {
    pub fn directional_light(&self) -> bool {
        self.flags & 4 != 0
    }

    pub fn point_light(&self) -> bool {
        !(self.directional_light() || self.true_point())
    }

    // TODO: Not really sure what true point is or how it differs from point
    // light. Need to check in game.
    pub fn true_point(&self) -> bool {
        self.flags & 8 != 0
    }

    // TODO: Originally checked `self.flags & 1 != 0` but that seems wrong.
    pub fn shadows_enabled(&self) -> bool {
        true
    }

    // TODO: Check what this means. Need to check in game.
    pub fn is_light(&self) -> bool {
        self.flags & 2 != 0
    }

    // TODO: Check what this means. Need to check in game.
    pub fn is_furn(&self) -> bool {
        self.flags & 16 != 0
    }

    // TODO: Check what this means. Need to check in game.
    pub fn is_base(&self) -> bool {
        self.flags & 32 != 0
    }
}
