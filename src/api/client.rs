use core::fmt;
use serde::Deserialize;
use std::time::Duration;

use crate::server::url;

use super::json;

pub struct ApiClient {
    url: String,
    client: reqwest::Client,
}

impl ApiClient {
    /// Make a call to any endpoint and parse a json response
    pub async fn call<T: for<'de> Deserialize<'de>>(&self, url: &str) -> anyhow::Result<T> {
        Ok(self.client.get(url).send().await?.json::<T>().await?)
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
    pub async fn register(&self) -> anyhow::Result<json::FullResponse> {
        self.call(&format!("{}/register", self.url)).await
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
    pub async fn mv(&self, id: u16, dir: MoveDir) -> anyhow::Result<json::PartialResponse> {
        self.call(&format!("{}/move{dir}/{id}", self.url)).await
    }

    /// If you want to get an update on what's going on in the world, but don't want
    /// to lose the sweet spot you have claimed, you can do that by looking. To
    /// look, make an HTTP request to the following url:
    ///     `http://xortag.apphb.com/look/{your_player_id}`
    ///
    /// As with moving, make sure to supply your user id. Also, in response to your
    /// request you'll receive back an updated JSON object.
    pub async fn look(&self, id: u16) -> anyhow::Result<json::PartialResponse> {
        self.call(&format!("{}/look/{id}", self.url)).await
    }

    /// Attempt to quit from the game. If this call succeeds, the server will remove
    /// the player and return the final state that the player would have seen.
    pub async fn quit(&self, id: u16) -> anyhow::Result<json::PartialResponse> {
        self.call(&format!("{}/quit/{id}", self.url)).await
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self {
            url: url(),
            client: reqwest::Client::builder()
                .timeout(Duration::from_millis(750))
                .build()
                .expect("hardcoded config for builder should not panic"),
        }
    }
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
