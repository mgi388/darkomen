mod decoder;
mod encoder;

#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;
use bitflags::bitflags;
use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};

pub use decoder::{DecodeError, Decoder};
pub use encoder::{EncodeError, Encoder};

/// The scale of the battle tabletop in the game world.
///
/// To get the world coordinates from the battle tabletop coordinates, divide
/// the battle tabletop coordinates by the scale.
pub const SCALE: f32 = 8.;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
pub struct BattleTabletop {
    pub width: u32,
    pub height: u32,
    /// The name of the player's army file, without the extension, e.g.,
    /// `b101mrc`.
    pub player_army: String,
    /// The name of the enemy's army file, without the extension, e.g.,
    /// `b101nme`.
    pub enemy_army: String,
    /// The name of the CTL file, without the extension, e.g., `B101`.
    pub ctl: String,
    unknown1: String,
    unknown2: String,
    unknown3: Vec<i32>,
    /// A list of objectives relevant to the battle.
    pub objectives: Vec<Objective>,
    pub obstacles: Vec<Obstacle>,
    obstacles_unknown1: i32,
    pub regions: Vec<Region>,
    pub nodes: Vec<Node>,
}

/// The ID of the critical regiment lose condition objective.
pub const CRITICAL_REGIMENT_LOSE_CONDITION_ID: i32 = 3;

/// The ID of the initial regiment orientation objective.
pub const INITIAL_REGIMENT_ORIENTATION_ID: i32 = 7;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Objective {
    /// The ID of the objective.
    ///
    /// Interesting IDs:
    ///
    /// - 3: Defines critical regiment lose condition. `value1` is the regiment
    ///   ID of the player regiment and `value2` is unknown.
    /// - 7: Defines initial regiment orientation on the battlefield. `value1`
    ///   is the orientation of player regiments and `value2` is the orientation
    ///   of enemy regiments.
    pub id: i32,
    pub value1: i32,
    pub value2: i32,
}

impl Objective {
    /// Returns the rotation in radians. 0 is north (up), π/2 is east (right), π
    /// is south (down), and 3π/2 is west (left).
    #[inline]
    pub fn rotation_radians(value: i32) -> f32 {
        (value as f32 / 512.0) * std::f32::consts::TAU
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Deserialize, Serialize)
)]
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
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct ObstacleFlags: u32 {
        const NONE = 0;
        const ACTIVE = 1 << 0;
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
pub struct Region {
    pub display_name: String,
    /// The original game writes over the existing display name with the new
    /// path but the old bytes are not cleared first. This field is used to
    /// store the residual bytes, if there are any. If it's `None` then there
    /// are no residual bytes / all bytes are zero after the null-terminated
    /// string. If it's `Some`, then it contains the residual bytes, up to, but
    /// not including, the last nul-terminated string.
    display_name_residual_bytes: Option<Vec<u8>>,
    pub flags: RegionFlags,
    /// The position of the region in the horizontal plane.
    pub position: IVec2,
    pub line_segments: Vec<LineSegment>,
}

impl Region {
    /// Returns `true` if the region is a deployment zone.
    pub fn is_deployment_zone(&self) -> bool {
        self.flags.contains(RegionFlags::PLAYER1_DEPLOYMENT_ZONE)
            || self.flags.contains(RegionFlags::PLAYER2_DEPLOYMENT_ZONE)
    }

    /// Returns `true` if the region is a player 1 deployment zone.
    pub fn is_player1_deployment_zone(&self) -> bool {
        self.flags.contains(RegionFlags::PLAYER1_DEPLOYMENT_ZONE)
    }

    /// Returns `true` if the region is a player 2 deployment zone.
    pub fn is_player2_deployment_zone(&self) -> bool {
        self.flags.contains(RegionFlags::PLAYER2_DEPLOYMENT_ZONE)
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

    /// Returns the position of the region in the horizontal plane, in world
    /// coordinates.
    #[inline]
    pub fn world_position(&self) -> Vec2 {
        Vec2::new(
            self.position.x as f32 / SCALE,
            self.position.y as f32 / SCALE,
        )
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct RegionFlags: u32 {
        const NONE = 0;
        const ACTIVE = 1 << 0;
        const CLOSED = 1 << 1;
        const OPEN = 1 << 2;
        const UNKNOWN_FLAG_2 = 1 << 3;
        /// The region is used for holes in the battle's navmesh.
        const BOUNDARY_REVERSED = 1 << 4;
        /// The region is used for the outer geometry of the battle's navmesh.
        const BATTLE_BOUNDARY = 1 << 5;
        const UNKNOWN_FLAG_3 = 1 << 6;
        const BOUNDARY = 1 << 7;
        /// The region is a deployment zone for player 1, i.e., the main player.
        const PLAYER1_DEPLOYMENT_ZONE = 1 << 8;
        /// The region is a deployment zone for player 2, i.e., the enemy.
        const PLAYER2_DEPLOYMENT_ZONE = 1 << 9;
        const VISIBLE_AREA = 1 << 10;
        const UNKNOWN_FLAG_4 = 1 << 11;
        const UNKNOWN_FLAG_5 = 1 << 12;
        const UNKNOWN_FLAG_6 = 1 << 13;
        const UNKNOWN_FLAG_7 = 1 << 14;
        const UNKNOWN_FLAG_8 = 1 << 15;
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(
    feature = "bevy_reflect",
    derive(Reflect),
    reflect(Debug, Default, Deserialize, Serialize)
)]
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
        self.flags.contains(NodeFlags::WAYPOINT)
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
    #[cfg_attr(feature = "bevy_reflect", derive(Reflect), reflect(opaque), reflect(Debug, Default, Deserialize, Hash, PartialEq, Serialize))]
    pub struct NodeFlags: u32 {
        const NONE = 0;
        const ACTIVE = 1 << 0;
        const REGIMENT = 1 << 1;
        const WAYPOINT = 1 << 2;
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
    use std::{
        ffi::{OsStr, OsString},
        fs::File,
        path::{Path, PathBuf},
    };

    use image::{DynamicImage, Rgba};
    use imageproc::{drawing::draw_hollow_rect_mut, rect::Rect};
    use pretty_assertions::assert_eq;

    use crate::project::{self, Project};

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

    fn roundtrip_test(original_bytes: &[u8], b: &BattleTabletop) {
        let mut encoded_bytes = Vec::new();
        Encoder::new(&mut encoded_bytes).encode(b).unwrap();

        let original_bytes = original_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        let encoded_bytes = encoded_bytes
            .chunks(16)
            .map(|chunk| {
                chunk
                    .iter()
                    .map(|b| format!("{b:02X}"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join("\n");

        assert_eq!(original_bytes, encoded_bytes);
    }

    #[test]
    fn test_decode_b1_01() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
            "1PBAT",
            "B1_01",
            "B1_01.BTB",
        ]
        .iter()
        .collect();

        let original_bytes = std::fs::read(d.clone()).unwrap();
        let file = File::open(d).unwrap();
        let b = Decoder::new(file).decode().unwrap();

        assert_eq!(b.width, 1440);
        assert_eq!(b.height, 1600);
        assert_eq!(b.player_army, "B101mrc");
        assert_eq!(b.enemy_army, "B101nme");
        assert_eq!(b.ctl, "B101");

        const EPSILON: f32 = 0.0001;

        assert!(b.obstacles[0]
            .world_position()
            .abs_diff_eq(Vec2::new(138.625, 47.5), EPSILON));
        assert!((b.obstacles[0].world_radius() - 7.875).abs() < EPSILON);
        assert!(b.obstacles[5]
            .world_position()
            .abs_diff_eq(Vec2::new(-0.75, 161.0), EPSILON));

        // Night Goblins#1
        assert!(b.nodes[0]
            .world_position()
            .abs_diff_eq(Vec2::new(151.25, 119.625), EPSILON));
        assert!((b.nodes[0].world_radius() - 6.0).abs() < EPSILON);
        assert!((b.nodes[0].rotation_degrees() - 182.10938).abs() < EPSILON);
        assert_eq!(b.nodes[0].regiment_id, 131);

        roundtrip_test(&original_bytes, &b);
    }

    #[test]
    fn test_decode_all() {
        let d: PathBuf = [
            std::env::var("DARKOMEN_PATH").unwrap().as_str(),
            "DARKOMEN",
            "GAMEDATA",
        ]
        .iter()
        .collect();

        let root_output_dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "decoded", "btbs"]
            .iter()
            .collect();

        std::fs::create_dir_all(&root_output_dir).unwrap();

        fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&Path)) {
            println!("Reading dir {:?}", dir.display());

            let mut paths = std::fs::read_dir(dir)
                .unwrap()
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, std::io::Error>>()
                .unwrap();

            paths.sort();

            for path in paths {
                if path.is_dir() {
                    visit_dirs(&path, cb);
                } else {
                    cb(&path);
                }
            }
        }

        visit_dirs(&d, &mut |path| {
            let Some(ext) = path.extension() else {
                return;
            };
            if ext.to_string_lossy().to_uppercase() != "BTB" {
                return;
            }

            println!("Decoding {:?}", path.file_name().unwrap());

            let parent_dir = path
                .components()
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .skip(1) // skip the file name
                .take_while(|c| c.as_os_str() != "DARKOMEN")
                .collect::<Vec<_>>()
                .iter()
                .rev()
                .collect::<PathBuf>();
            let output_dir = root_output_dir.join(parent_dir);
            std::fs::create_dir_all(&output_dir).unwrap();

            let original_bytes = std::fs::read(path).unwrap();
            let file = File::open(path).unwrap();
            let b = Decoder::new(file).decode().unwrap();

            // The width and height should be multiples of 8.
            assert_eq!(b.width % 8, 0);
            assert_eq!(b.height % 8, 0);

            roundtrip_test(&original_bytes, &b);

            let project_file = File::open(path.with_extension("PRJ"));
            if project_file.is_ok() {
                let p = project::Decoder::new(project_file.unwrap())
                    .decode()
                    .unwrap();

                // The scaled down dimensions should always be smaller than the
                // project dimensions.
                assert!(b.width / 8 <= p.attributes.width);
                assert!(b.height / 8 <= p.attributes.height);

                // Overlay the battle tabletop on the heightmap image.
                let img = overlay_battle_tabletop_on_terrain(&p, &b);
                img.save(
                    output_dir
                        .join(path.file_stem().unwrap())
                        .with_extension("overlay.png"),
                )
                .unwrap();
            }

            // Every 1-player battle tabletop should at least have the following
            // objectives.
            for id in [
                1,
                CRITICAL_REGIMENT_LOSE_CONDITION_ID,
                4,
                INITIAL_REGIMENT_ORIENTATION_ID,
                26,
            ] {
                // Skip if file name is TMPBAT.BTB.
                if path.file_name().unwrap() == "TMPBAT.BTB" {
                    continue;
                }
                // Skip the multiplayer files.
                if path.file_name().unwrap().to_str().unwrap().starts_with('M') {
                    continue;
                }
                // Skip the tutorial file.
                if path.file_name().unwrap() == "SPARE9.BTB" {
                    continue;
                }

                assert!(
                    b.objectives.iter().any(|obj| obj.id == id),
                    "Battle tabletop {:?} is missing required objective ID: {}",
                    path.file_name().unwrap(),
                    id
                );
            }

            // Every multiplayer battle tabletop should have the following
            // objectives.
            for id in [1, 4, INITIAL_REGIMENT_ORIENTATION_ID, 26] {
                // Skip non-multiplayer files.
                if !path.file_name().unwrap().to_str().unwrap().starts_with('M') {
                    continue;
                }

                assert!(
                    b.objectives.iter().any(|obj| obj.id == id),
                    "Battle tabletop {:?} is missing required objective ID: {}",
                    path.file_name().unwrap(),
                    id
                );
            }

            for o in &b.obstacles {
                // Should either block movement or projectiles.
                assert!(
                    o.flags.contains(ObstacleFlags::BLOCKS_MOVEMENT)
                        || o.flags.contains(ObstacleFlags::BLOCKS_PROJECTILES)
                );
                // All obstacles should be active.
                assert!(o.flags.contains(ObstacleFlags::ACTIVE));
            }

            for region in &b.regions {
                // All regions should be active.
                assert!(region.flags.contains(RegionFlags::ACTIVE));
            }

            for node in &b.nodes {
                // All nodes should be active.
                assert!(node.flags.contains(NodeFlags::ACTIVE));

                // All regiment nodes should have a regiment ID except for
                // "TMPBAT.BTB". "TMPBAT.BTB" is the only file that has a
                // regiment node with a regiment ID of 0. It was probably a
                // temporary file.
                if node.flags.contains(NodeFlags::REGIMENT)
                    && path.file_name().unwrap() != "TMPBAT.BTB"
                {
                    assert!(node.regiment_id > 0);
                }

                // All waypoint nodes should have a route ID.
                if node.flags.contains(NodeFlags::WAYPOINT) {
                    assert!(node.node_id > 0);
                }
            }

            let output_path = append_ext("ron", output_dir.join(path.file_name().unwrap()));
            let mut output_file = File::create(output_path).unwrap();
            ron::ser::to_writer_pretty(&mut output_file, &b, Default::default()).unwrap();
        });
    }

    /// Note: We know the battle tabletop always fits within the project
    /// dimensions so we don't need to expand the base image.
    fn overlay_battle_tabletop_on_terrain(p: &Project, b: &BattleTabletop) -> DynamicImage {
        // Doesn't matter which heightmap we use, they all have the same
        // dimensions, but the furniture one has the most detail.
        let img = p.terrain.furniture_heightmap_image();
        let mut img_buffer = img.to_rgba8();

        // The image is quite dark, so invert colors just for ease of viewing.
        for pixel in img_buffer.pixels_mut() {
            let (r, g, b, a) = (255 - pixel[0], 255 - pixel[1], 255 - pixel[2], pixel[3]); // invert RGB, keep alpha the same
            *pixel = Rgba([r, g, b, a]);
        }

        // Pin the rectangle to the top right which is the terrain origin.
        let start_x = img_buffer.width() as i32 - (b.width / 8) as i32;
        let start_y = 0; // top edge, so y is 0

        // Draw a hollow rectangle on the base image to show the battle tabletop
        // dimensions.
        let rect = Rect::at(start_x, start_y).of_size(b.width / 8, b.height / 8);
        draw_hollow_rect_mut(&mut img_buffer, rect, Rgba([255, 0, 0, 255]));

        // Now rotate the image 180 degrees to make the origin at the bottom
        // left which matches the in-game aeiral map view.
        let img_buffer = image::imageops::rotate180(&img_buffer);

        DynamicImage::ImageRgba8(img_buffer)
    }

    fn append_ext(ext: impl AsRef<OsStr>, path: PathBuf) -> PathBuf {
        let mut os_string: OsString = path.into();
        os_string.push(".");
        os_string.push(ext.as_ref());
        os_string.into()
    }
}
