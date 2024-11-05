use super::{Color, InvalidPieceCharacterError};
use std::{collections::HashMap, fmt};

/// Represents a piece in the format (_piece type_, _color_).
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Piece(pub(crate) PieceType, pub(crate) Color);

impl Piece {
    /// Returns the type of piece.
    pub fn piece_type(&self) -> PieceType {
        self.0
    }

    /// Returns the color of the piece.
    pub fn color(&self) -> Color {
        self.1
    }
}

impl TryFrom<char> for Piece {
    type Error = InvalidPieceCharacterError;

    /// Attempts to convert a piece character to a `Piece`.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(Self(PieceType::try_from(value)?, if value.is_ascii_uppercase() { Color::White } else { Color::Black }))
    }
}

impl From<Piece> for char {
    /// Converts a `Piece` to a piece character.
    fn from(piece: Piece) -> char {
        let ch = piece.0.into();
        match piece.1 {
            Color::White => ch,
            Color::Black => ch.to_ascii_lowercase(),
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let codepoints = HashMap::from([
            (PieceType::K, 0x2654),
            (PieceType::Q, 0x2655),
            (PieceType::R, 0x2656),
            (PieceType::B, 0x2657),
            (PieceType::N, 0x2658),
            (PieceType::P, 0x2659),
        ]);
        let Self(t, c) = self;
        write!(f, "{}", char::from_u32((codepoints.get(t).unwrap() + if c.is_white() { 0 } else { 6 }) as u32).unwrap())
    }
}

/// Represents types of pieces.
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum PieceType {
    K,
    Q,
    B,
    N,
    R,
    P,
}

impl TryFrom<char> for PieceType {
    type Error = InvalidPieceCharacterError;

    /// Attempts to convert a piece character to a `PieceType`.
    fn try_from(value: char) -> Result<Self, Self::Error> {
        if !value.is_ascii_alphanumeric() {
            return Err(InvalidPieceCharacterError(value));
        }
        Ok(match value.to_ascii_lowercase() {
            'k' => Self::K,
            'q' => Self::Q,
            'b' => Self::B,
            'n' => Self::N,
            'r' => Self::R,
            'p' => Self::P,
            _ => return Err(InvalidPieceCharacterError(value)),
        })
    }
}

impl From<PieceType> for char {
    /// Converts a `PieceType` to a piece character (uppercase).
    fn from(piece_type: PieceType) -> char {
        match piece_type {
            PieceType::K => 'K',
            PieceType::Q => 'Q',
            PieceType::B => 'B',
            PieceType::N => 'N',
            PieceType::R => 'R',
            PieceType::P => 'P',
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", char::from(*self))
    }
}
