//! A Rust chess library with the aim to be as feature-rich as possible
//!
//! Examples are available on the [GitHub repository page](https://github.com/Python3-8/rschess).

mod board;
pub mod errors;
mod fen;
mod game_result;
mod helpers;
#[cfg(feature = "img")]
pub mod img;
mod move_;
#[cfg(feature = "pgn")]
pub mod pgn;
mod piece;
mod position;

pub use board::*;
pub(crate) use errors::*;
pub use fen::Fen;
pub use game_result::*;
pub use move_::*;
pub use piece::*;
pub use position::*;
use std::{fmt, ops::Not};

/// Converts a square index (`0..64`) to a square name, returning an error if the square index is invalid.
pub fn idx_to_sq(idx: usize) -> Result<(char, char), InvalidSquareIndexError> {
    if !(0..64).contains(&idx) {
        return Err(InvalidSquareIndexError(idx));
    }
    Ok(helpers::idx_to_sq(idx))
}

/// Converts a square name to a square index, returning an error if the square name is invalid.
pub fn sq_to_idx(file: char, rank: char) -> Result<usize, InvalidSquareNameError> {
    if !(('a'..'h').contains(&file) && ('1'..'8').contains(&rank)) {
        return Err(InvalidSquareNameError(file, rank));
    }
    Ok(helpers::sq_to_idx(file, rank))
}

/// Represents a side/color.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Checks if the color is white.
    pub fn is_white(&self) -> bool {
        matches!(self, Self::White)
    }

    /// Checks if the color is black.
    pub fn is_black(&self) -> bool {
        matches!(self, Self::Black)
    }
}

impl TryFrom<&str> for Color {
    type Error = InvalidColorCharacterError;

    /// Attempts to convert a color character in a string slice to a `Color` ("w" is white, and "b" is black).
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        match string {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => Err(InvalidColorCharacterError(string.to_string())),
        }
    }
}

impl From<Color> for char {
    /// Converts a `Color` to a color character (white is 'w', and black is 'b').
    fn from(c: Color) -> char {
        match c {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}

impl Not for Color {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[cfg(test)]
mod test;
