use core::fmt;
use serde::{Deserialize, Serialize};

pub const URL: &str = "http://localhost:3000";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct FullState {
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
    pub inner: PartialState,
}

/// Partial state to deserialize on updates. See `RegisterResult` for the full
/// struct that includes this one as well as the stable fields such as `id`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PartialState {
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

impl PartialState {
    pub fn occupied(&self, x: i16, y: i16) -> bool {
        self.x == x && self.y == y || self.players.iter().any(|p| p.x == x && p.y == y)
    }
}

/// Make a call to a player action endpoint
pub async fn call<T: for<'de> Deserialize<'de>>(url: &str) -> anyhow::Result<T> {
    Ok(reqwest::get(url).await?.json::<T>().await?)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PlayerLocation {
    pub is_it: bool,
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
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
            s if s == "up" => Self::Up,
            s if s == "down" => Self::Down,
            s if s == "left" => Self::Left,
            s if s == "right" => Self::Right,
            _ => Self::None,
        }
    }
}

impl fmt::Display for MoveDir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
            Self::None => "look",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i16, pub i16);

impl Pos {
    pub fn distance(&self, other: &Self) -> u16 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }

    pub fn successors(&self, game: &PartialState) -> Vec<(Self, u16)> {
        let &Self(x, y) = self;
        let mut successors = Vec::new();

        if !game.occupied(x + 1, y) {
            successors.push((Self(x + 1, y), 1));
        }

        if !game.occupied(x - 1, y) {
            successors.push((Self(x - 1, y), 1));
        }

        if !game.occupied(x, y + 1) {
            successors.push((Self(x, y + 1), 1));
        }

        if !game.occupied(x, y - 1) {
            successors.push((Self(x, y - 1), 1));
        }

        successors
    }
}

/// This is the first step you'll need to do.
///
/// When you register the game will create your player, assign you an id, pick a
/// name for you and put your player on the map.
///
/// When you register you'll get back a JSON object `Res` that
/// will let you know what your id is and where your player is.
///
/// To register you need to make an HTTP request to the following url:
///     `http://xortag.apphb.com/register`
pub async fn register() -> anyhow::Result<FullState> {
    call(&format!("{URL}/register")).await
}

/// Once you are registered you can start moving your player around. This is the
/// heart of tag.
///
/// If you are "it", try to move towards other players and tag them.
///
/// If you aren't "it", try to run away from the player who is.
///
/// If you move to the same space where another player is and one of you is it,
/// that counts as a tag. If neither of you are it, you won't go anywhere. No
/// piggybacking here.
pub async fn mv(id: u16, dir: MoveDir) -> anyhow::Result<PartialState> {
    call(&format!("{URL}/move{dir}/{id}")).await
}

/// If you want to get an update on what's going on in the world, but don't want
/// to lose the sweet spot you have claimed, you can do that by looking. To
/// look, make an HTTP request to the following url:
///     `http://xortag.apphb.com/look/{your_player_id}`
///
/// As with moving, make sure to supply your user id. Also, in response to your
/// request you'll receive back an updated JSON object.
pub async fn look(id: u16) -> anyhow::Result<PartialState> {
    call(&format!("{URL}/look/{id}")).await
}

/// Attempt to quit from the game. If this call succeeds, the server will remove
/// the player and return the final state that the player would have seen.
pub async fn quit(id: u16) -> anyhow::Result<PartialState> {
    call(&format!("{URL}/quit/{id}")).await
}
