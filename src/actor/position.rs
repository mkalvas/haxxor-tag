use crate::api::FullResponse;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Pos(pub i16, pub i16);

impl Pos {
    pub fn distance(&self, other: &Self) -> u16 {
        self.0.abs_diff(other.0) + self.1.abs_diff(other.1)
    }

    pub fn successors(&self, game: &FullResponse) -> Vec<(Self, u16)> {
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
