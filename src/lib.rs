use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub const URL: &str = "http://localhost:3000";

/// Partial state to deserialize on updates. See `RegisterResult` for the full
/// struct that includes this one as well as the stable fields such as `id`.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CommandResult {
    /// Let's you know if you are it or not.
    ///
    /// True means you're it.
    ///
    /// False means run for your life.
    pub is_it: bool,

    /// An array of other players that are close enough for you to see.
    ///
    /// Each player has an X position, a Y position and whether or not they are
    /// it. If they aren't it and you are, get 'em! If they are it, run for it.
    pub players: Vec<PlayerLocation>,

    /// The X (horizontal) position of your player.
    ///
    /// The left-most column on the map is position `0`.
    ///
    /// The right-most column is `mapWidth - 1`.
    pub x: i16,

    /// The Y (vertical) position of your player.
    ///
    /// The top row of the map is position `0`.
    ///
    /// The bottom row is <span class="code">mapHeight - 1</span>.
    pub y: i16,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterResult {
    /// The id for your player.
    ///
    /// You'll use it to make all your other requests to the game. That's how
    /// the game knows it's you, rather than that shady looking guy over there
    /// in the corner.
    pub id: u16,
    /// How many tiles wide the map is.
    ///
    /// Default 30
    pub map_height: i16,

    /// How many tiles high the map is.
    ///
    /// Default 50
    pub map_width: i16,

    /// Your player's name.
    ///
    /// Everyone's got to have a name.
    pub name: String,

    /// Include the sub-struct that we care about during updates.
    #[serde(flatten)]
    pub inner: CommandResult,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerLocation {
    pub is_it: bool,
    pub x: i16,
    pub y: i16,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum MoveDir {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl From<&String> for MoveDir {
    fn from(value: &String) -> Self {
        match value {
            s if s == "up" => MoveDir::Up,
            s if s == "down" => MoveDir::Down,
            s if s == "left" => MoveDir::Left,
            s if s == "right" => MoveDir::Right,
            _ => MoveDir::None,
        }
    }
}

impl Display for MoveDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MoveDir::Up => "up",
            MoveDir::Down => "down",
            MoveDir::Left => "left",
            MoveDir::Right => "right",
            MoveDir::None => "look",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i16, pub i16);

impl Pos {
    pub fn distance(&self, other: &Pos) -> u16 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }

    pub fn successors(&self) -> Vec<(Pos, u16)> {
        let &Pos(x, y) = self;
        vec![
            (Pos(x + 1, y), 1),
            (Pos(x - 1, y), 1),
            (Pos(x, y + 1), 1),
            (Pos(x, y - 1), 1),
        ]
    }
}
