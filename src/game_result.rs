use super::Color;
use std::fmt;

/// Represents game results.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum GameResult {
    Wins(Color, WinType),
    Draw(DrawType),
}

impl fmt::Display for GameResult {
    /// Represents the game result as a string (1-0 if white wins, 0-1 if black wins, or 1/2-1/2 in the case of a draw).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Wins(c, _) =>
                    if c.is_white() {
                        "1-0"
                    } else {
                        "0-1"
                    },
                Self::Draw(_) => "1/2-1/2",
            }
        )
    }
}

/// Represents types of wins.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum WinType {
    Checkmate,
    /// Currently, a loss by timeout is also considered a resignation.
    Resignation,
}

/// Represents types of draws.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum DrawType {
    FivefoldRepetition,
    SeventyFiveMoveRule,
    /// Represents a stalemate, with the tuple value being the side in stalemate.
    Stalemate(Color),
    InsufficientMaterial,
    /// Currently, a claimed draw and a draw by timeout vs. insufficient checkmating material are also considered a draw by agreement.
    Agreement,
}
