mod decoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Blueprint {
    pub width: u32,
    pub height: u32,
    /// The name of the player's army file, without the extension. E.g.
    /// `b101mrc`.
    pub player_army: String,
    /// The name of the enemy's army file, without the extension. E.g.
    /// `b101nme`.
    pub enemy_army: String,
    /// The name of the CTL file, without the extension. E.g. `B101`.
    pub ctl: String,
    pub objectives: Vec<Objective>,
    pub obstacles: Vec<Obstacle>,
    pub regions: Vec<Region>,
    pub nodes: Vec<Node>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Objective {
    pub typ: i32,
    pub val1: i32,
    pub val2: i32,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Obstacle {
    pub flags: ObstacleFlags,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub radius: u32,
    pub dir: i32,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect_value(Debug, Deserialize, Hash, PartialEq, Serialize))]
    pub struct ObstacleFlags: u32 {
        const NONE = 0;
        const IS_ENABLED = 1 << 0;
        const BLOCKS_MOVEMENT = 1 << 1;
        const BLOCKS_PROJECTILES = 1 << 2;
        const UNKNOWN_FLAG_1 = 1 << 3;
        const UNKNOWN_FLAG_2 = 1 << 4;
        const UNKNOWN_FLAG_3 = 1 << 5;
        const UNKNOWN_FLAG_4 = 1 << 6;
        const UNKNOWN_FLAG_5 = 1 << 7;
        const UNKNOWN_FLAG_6 = 1 << 8;
        const UNKNOWN_FLAG_7 = 1 << 9;
        const UNKNOWN_FLAG_8 = 1 << 10;
        const UNKNOWN_FLAG_9 = 1 << 11;
        const UNKNOWN_FLAG_10 = 1 << 12;
        const UNKNOWN_FLAG_11 = 1 << 13;
        const UNKNOWN_FLAG_12 = 1 << 14;
        const UNKNOWN_FLAG_13 = 1 << 15;
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct LineSegment {
    pub start_x: i32,
    pub start_y: i32,
    pub end_x: i32,
    pub end_y: i32,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Region {
    pub name: String,
    pub flags: RegionFlags,
    pub line_segments: Vec<LineSegment>,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect_value(Debug, Deserialize, Hash, PartialEq, Serialize))]
    pub struct RegionFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const IS_CLOSED = 1 << 1;
        const IS_OPEN = 1 << 2;
        const UNKNOWN_FLAG_2 = 1 << 3;
        const IS_BOUNDARY_REVERSED = 1 << 4;
        const IS_BATTLE_BOUNDARY = 1 << 5;
        const UNKNOWN_FLAG_3 = 1 << 6;
        const IS_BOUNDARY = 1 << 7;
        const IS_PLAYER1_DEPLOY_AREA = 1 << 8;
        const IS_PLAYER2_DEPLOY_AREA = 1 << 9;
        const IS_VISIBLE_AREA = 1 << 10;
        const UNKNOWN_FLAG_4 = 1 << 11;
        const UNKNOWN_FLAG_5 = 1 << 12;
        const UNKNOWN_FLAG_6 = 1 << 13;
        const UNKNOWN_FLAG_7 = 1 << 14;
        const UNKNOWN_FLAG_8 = 1 << 15;
    }
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Node {
    pub flags: NodeFlags,
    pub x: i32,
    pub y: i32,
    pub radius: u32,
    pub direction: i32,
    pub node_id: u32,
    pub uuid: u32,
    pub script_id: u32,
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect_value(Debug, Deserialize, Hash, PartialEq, Serialize))]
    pub struct NodeFlags: u32 {
        const NONE = 0;
        const IS_CENTERED_POINT = 1 << 0;
        const IS_UNIT = 1 << 1;
        const IS_WAYPOINT = 1 << 2;
        const UNKNOWN_FLAG_1 = 1 << 3;
        const UNKNOWN_FLAG_2 = 1 << 4;
        const UNKNOWN_FLAG_3 = 1 << 5;
        const UNKNOWN_FLAG_4 = 1 << 6;
        const UNKNOWN_FLAG_5 = 1 << 7;
        const UNKNOWN_FLAG_6 = 1 << 8;
        const UNKNOWN_FLAG_7 = 1 << 9;
        const UNKNOWN_FLAG_8 = 1 << 10;
        const UNKNOWN_FLAG_9 = 1 << 11;
        const UNKNOWN_FLAG_10 = 1 << 12;
        const UNKNOWN_FLAG_11 = 1 << 13;
        const UNKNOWN_FLAG_12 = 1 << 14;
        const UNKNOWN_FLAG_13 = 1 << 15;
    }
}
