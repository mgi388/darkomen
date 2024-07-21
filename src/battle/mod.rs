mod decoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::IVec2;
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

impl Obstacle {
    /// Returns the position of the obstacle in the horizontal plane.
    #[inline]
    pub fn position(&self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct ObstacleFlags(u32);

bitflags! {
    impl ObstacleFlags: u32 {
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

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct LineSegment {
    /// The start position of the line segment in the horizontal plane.
    pub start: IVec2,
    /// The end position of the line segment in the horizontal plane.
    pub end: IVec2,
}

impl LineSegment {
    /// Checks if a point is on a line segment.
    fn point_on_line_segment(&self, point: &IVec2) -> bool {
        let crossproduct = (point.y - self.start.y) * (self.end.x - self.start.x)
            - (point.x - self.start.x) * (self.end.y - self.start.y);
        if crossproduct != 0 {
            return false;
        }

        let dotproduct = (point.x - self.start.x) * (self.end.x - self.start.x)
            + (point.y - self.start.y) * (self.end.y - self.start.y);
        if dotproduct < 0 {
            return false;
        }

        let squared_length_line =
            (self.end.x - self.start.x).pow(2) + (self.end.y - self.start.y).pow(2);
        if dotproduct > squared_length_line {
            return false;
        }

        true
    }

    /// Checks if the ray from the point intersects with the line segment.
    fn ray_intersects_segment(&self, point: &IVec2) -> bool {
        // Ensure the point is between the y-coordinates of the line segment's
        // endpoints.
        if point.y < self.start.y.min(self.end.y) || point.y > self.start.y.max(self.end.y) {
            return false;
        }

        // Avoid division by zero for horizontal line segments.
        if self.end.y == self.start.y {
            return false;
        }

        // Calculate the x-coordinate of the intersection.
        let intersection_x = self.start.x
            + (point.y - self.start.y) * (self.end.x - self.start.x) / (self.end.y - self.start.y);

        // The ray intersects if the intersection is to the right of the point.
        intersection_x >= point.x
    }
}

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Region {
    pub name: String,
    pub flags: RegionFlags,
    pub line_segments: Vec<LineSegment>,
}

impl Region {
    /// Returns whether the region is a deployment zone.
    pub fn is_deployment_zone(&self) -> bool {
        self.flags.contains(RegionFlags::IS_PLAYER1_DEPLOYMENT_ZONE)
            || self.flags.contains(RegionFlags::IS_PLAYER2_DEPLOYMENT_ZONE)
    }

    /// Returns whether the given point is contained within the region.
    pub fn contains_point(&self, point: IVec2) -> bool {
        let mut intersections = 0;
        for line in &self.line_segments {
            // Check if point is exactly on the line segment.
            if line.point_on_line_segment(&point) {
                return true;
            }

            // Check for intersections with the ray.
            if line.ray_intersects_segment(&point) {
                intersections += 1;
            }
        }

        // Odd number of intersections means the point is inside.
        intersections % 2 == 1
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct RegionFlags(u32);

bitflags! {
    impl RegionFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const IS_CLOSED = 1 << 1;
        const IS_OPEN = 1 << 2;
        const UNKNOWN_FLAG_2 = 1 << 3;
        const IS_BOUNDARY_REVERSED = 1 << 4;
        const IS_BATTLE_BOUNDARY = 1 << 5;
        const UNKNOWN_FLAG_3 = 1 << 6;
        const IS_BOUNDARY = 1 << 7;
        /// The region is a deployment zone for player 1, i.e. the main player.
        const IS_PLAYER1_DEPLOYMENT_ZONE = 1 << 8;
        /// The region is a deployment zone for player 2, i.e. the enemy.
        const IS_PLAYER2_DEPLOYMENT_ZONE = 1 << 9;
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

impl Node {
    /// Returns the position of the node in the horizontal plane.
    #[inline]
    pub fn position(&self) -> IVec2 {
        IVec2::new(self.x, self.y)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct NodeFlags(u32);

bitflags! {
    impl NodeFlags: u32 {
        const NONE = 0;
        const IS_CENTERED_POINT = 1 << 0;
        const IS_REGIMENT = 1 << 1;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_contains_point() {
        let region = Region {
            line_segments: vec![
                LineSegment {
                    start: IVec2::new(0, 0),
                    end: IVec2::new(10, 0),
                },
                LineSegment {
                    start: IVec2::new(10, 0),
                    end: IVec2::new(10, 10),
                },
                LineSegment {
                    start: IVec2::new(10, 10),
                    end: IVec2::new(0, 10),
                },
                LineSegment {
                    start: IVec2::new(0, 10),
                    end: IVec2::new(0, 0),
                },
            ],
            ..Default::default()
        };

        assert!(region.contains_point(IVec2::new(5, 5)));
        assert!(region.contains_point(IVec2::new(0, 0)));
        assert!(region.contains_point(IVec2::new(10, 0)));
        assert!(region.contains_point(IVec2::new(10, 10)));
        assert!(region.contains_point(IVec2::new(0, 10)));
        assert!(!region.contains_point(IVec2::new(11, 0)));
        assert!(!region.contains_point(IVec2::new(0, 11)));
        assert!(!region.contains_point(IVec2::new(11, 11)));
    }
}
