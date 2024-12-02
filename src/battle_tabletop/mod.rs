mod decoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};

/// The scale of the battle tabletop in the game world.
///
/// To get the world coordinates from the battle tabletop coordinates, divide
/// the battle tabletop coordinates by the scale.
pub const SCALE: f32 = 8.;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct BattleTabletop {
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

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Obstacle {
    pub flags: ObstacleFlags,
    /// The position of the obstacle in the horizontal plane.
    pub position: IVec2,
    pub z: i32,
    pub radius: u32,
    pub dir: i32,
}

impl Obstacle {
    /// Returns the position of the obstacle in the horizontal plane, in world
    /// coordinates.
    #[inline]
    pub fn world_position(&self) -> Vec2 {
        Vec2::new(
            self.position.x as f32 / SCALE,
            self.position.y as f32 / SCALE,
        )
    }

    /// Returns the radius of the obstacle in world space.
    #[inline]
    pub fn world_radius(&self) -> f32 {
        self.radius as f32 / SCALE
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
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

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct LineSegment {
    /// The start position of the line segment in the horizontal plane.
    pub start: IVec2,
    /// The end position of the line segment in the horizontal plane.
    pub end: IVec2,
}

impl LineSegment {
    /// Returns the start position of the line segment in world coordinates.
    #[inline]
    pub fn world_start(&self) -> Vec2 {
        Vec2::new(self.start.x as f32 / SCALE, self.start.y as f32 / SCALE)
    }

    /// Returns the end position of the line segment in world coordinates.
    #[inline]
    pub fn world_end(&self) -> Vec2 {
        Vec2::new(self.end.x as f32 / SCALE, self.end.y as f32 / SCALE)
    }

    /// Returns `true` if a point is on a line segment.
    fn is_point_on_line_segment(&self, point: &IVec2) -> bool {
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

    /// Returns `true` if the ray from the point is intersecting with the line
    /// segment.
    fn is_ray_intersecting_segment(&self, point: &IVec2) -> bool {
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
    /// Returns `true` if the region is a deployment zone.
    pub fn is_deployment_zone(&self) -> bool {
        self.flags.contains(RegionFlags::IS_PLAYER1_DEPLOYMENT_ZONE)
            || self.flags.contains(RegionFlags::IS_PLAYER2_DEPLOYMENT_ZONE)
    }

    /// Returns `true` if the given point is contained within the region.
    pub fn is_point_contained(&self, point: IVec2) -> bool {
        let mut intersections = 0;
        for line in &self.line_segments {
            // Check if point is exactly on the line segment.
            if line.is_point_on_line_segment(&point) {
                return true;
            }

            // Check for intersections with the ray.
            if line.is_ray_intersecting_segment(&point) {
                intersections += 1;
            }
        }

        // Odd number of intersections means the point is inside.
        intersections % 2 == 1
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct RegionFlags: u32 {
        const NONE = 0;
        const UNKNOWN_FLAG_1 = 1 << 0;
        const IS_CLOSED = 1 << 1;
        const IS_OPEN = 1 << 2;
        const UNKNOWN_FLAG_2 = 1 << 3;
        /// The region is used for holes in the battle's navmesh.
        const IS_BOUNDARY_REVERSED = 1 << 4;
        /// The region is used for the outer geometry of the battle's navmesh.
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

#[derive(Clone, Debug, Default, Serialize)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
pub struct Node {
    pub flags: NodeFlags,
    /// The position of the node in the horizontal plane.
    pub position: IVec2,
    pub radius: u32,
    /// The rotation of the node as a value between 0 (inclusive) and 512
    /// (exclusive). The rotation is in the range [0, 512) and corresponds to
    /// the angle in degrees. When looking at an aerial view of the map, 0 is
    /// north (up), 128 is east (right), 256 is south (down), and 384 is west
    /// (left).
    ///
    /// The rotation represents a 2D rotation around the horizontal plane.
    ///
    /// Note: The rotation does not seem to be used for player regiments.
    pub rotation: i32,
    pub node_id: u32,
    /// The ID of the regiment the node belongs to. Corresponds to the ID field
    /// of the regiment.
    pub regiment_id: u32,
    pub script_id: u32,
}

impl Node {
    /// Returns `true` if the node is a waypoint.
    #[inline]
    pub fn is_waypoint(&self) -> bool {
        self.flags.contains(NodeFlags::IS_WAYPOINT)
    }

    /// Returns the position of the node in the horizontal plane in world
    /// coordinates.
    #[inline]
    pub fn world_position(&self) -> Vec2 {
        Vec2::new(
            self.position.x as f32 / SCALE,
            self.position.y as f32 / SCALE,
        )
    }

    /// Returns the radius of the node in world space.
    #[inline]
    pub fn world_radius(&self) -> f32 {
        self.radius as f32 / SCALE
    }

    /// Returns the rotation of the node in radians. 0 is north (up), π/2 is
    /// east (right), π is south (down), and 3π/2 is west (left).
    #[inline]
    pub fn rotation_radians(&self) -> f32 {
        (self.rotation as f32 / 512.0) * std::f32::consts::TAU
    }

    /// Returns the rotation of the node in degrees. 0 is north (up), 90 is east
    /// (right), 180 is south (down), and 270 is west (left).
    #[inline]
    pub fn rotation_degrees(&self) -> f32 {
        self.rotation_radians().to_degrees()
    }

    /// Returns `true` if the node belongs to player 1's regiment.
    ///
    /// TODO: Is there a more reliable way to determine this?
    #[inline]
    pub fn is_player1_regiment(&self) -> bool {
        self.regiment_id <= 100
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
    #[cfg_attr(feature = "bevy_reflect", reflect(opaque))]
    #[cfg_attr(feature = "bevy_reflect", reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct NodeFlags: u32 {
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
    fn test_region_is_point_contained() {
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

        assert!(region.is_point_contained(IVec2::new(5, 5)));
        assert!(region.is_point_contained(IVec2::new(0, 0)));
        assert!(region.is_point_contained(IVec2::new(10, 0)));
        assert!(region.is_point_contained(IVec2::new(10, 10)));
        assert!(region.is_point_contained(IVec2::new(0, 10)));
        assert!(!region.is_point_contained(IVec2::new(11, 0)));
        assert!(!region.is_point_contained(IVec2::new(0, 11)));
        assert!(!region.is_point_contained(IVec2::new(11, 11)));
    }

    #[test]
    fn test_node_rotation() {
        let node = Node {
            rotation: 0, // north (up)
            ..Default::default()
        };
        assert_eq!(node.rotation_radians(), 0.);
        assert_eq!(node.rotation_degrees(), 0.);

        let node = Node {
            rotation: 256, // south (down)
            ..Default::default()
        };
        assert_eq!(node.rotation_radians(), std::f32::consts::PI);
        assert_eq!(node.rotation_degrees(), 180.);

        let node = Node {
            rotation: 128, // east (right)
            ..Default::default()
        };
        assert_eq!(node.rotation_radians(), std::f32::consts::PI / 2.);
        assert_eq!(node.rotation_degrees(), 90.);

        let node = Node {
            rotation: 384, // west (left)
            ..Default::default()
        };
        assert_eq!(node.rotation_radians(), std::f32::consts::PI * 1.5);
        assert_eq!(node.rotation_degrees(), 270.);
    }
}
