use super::{helpers, InvalidUciError, PieceType};
use std::fmt;

/// The structure for a chess move, in the format (_source square_, _destination square_, _castling/promotion/en passant_)
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct Move(pub(crate) usize, pub(crate) usize, pub(crate) Option<SpecialMoveType>);

impl Move {
    /// Returns the source square of the move in the format (_file_, _rank_).
    pub fn from_square(&self) -> (char, char) {
        helpers::idx_to_sq(self.0)
    }

    /// Returns the destination square of the move in the format (_file_, _rank_).
    pub fn to_square(&self) -> (char, char) {
        helpers::idx_to_sq(self.1)
    }

    /// Returns the type of special move (castling/promotion/en passant) if this move is a special move (otherwise `None`).
    pub fn special_move_type(&self) -> Option<SpecialMoveType> {
        self.2
    }

    /// Creates a `Move` object from its UCI representation.
    pub fn from_uci(uci: &str) -> Result<Self, InvalidUciError> {
        let uci_len = uci.len();
        if ![4, 5].contains(&uci_len) {
            return Err(InvalidUciError::Length);
        }
        let from_square = (uci.chars().next().unwrap(), uci.chars().nth(1).unwrap());
        let to_square = (uci.chars().nth(2).unwrap(), uci.chars().nth(3).unwrap());
        let promotion = uci.chars().nth(4);
        if !(('a'..='h').contains(&from_square.0) && ('1'..='8').contains(&from_square.1)) {
            return Err(InvalidUciError::InvalidSquareName(from_square.0, from_square.1));
        }
        if !(('a'..='h').contains(&to_square.0) && ('1'..='8').contains(&to_square.1)) {
            return Err(InvalidUciError::InvalidSquareName(to_square.0, to_square.1));
        }
        let (src, dest) = (helpers::sq_to_idx(from_square.0, from_square.1), helpers::sq_to_idx(to_square.0, to_square.1));
        let promotion = match promotion {
            Some(p) => Some({
                let pt = PieceType::try_from(p).map_err(|_| InvalidUciError::InvalidPieceType(p))?;
                if pt == PieceType::K {
                    return Err(InvalidUciError::InvalidPieceType(p));
                } else {
                    pt
                }
            }),
            _ => None,
        };
        Ok(Self(
            src,
            dest,
            match promotion {
                Some(p) => Some(SpecialMoveType::Promotion(p)),
                _ => Some(SpecialMoveType::Unclear),
            },
        ))
    }

    /// Returns the UCI representation of the move.
    pub fn to_uci(&self) -> String {
        let ((srcf, srcr), (destf, destr)) = (helpers::idx_to_sq(self.0), helpers::idx_to_sq(self.1));
        format!(
            "{srcf}{srcr}{destf}{destr}{}",
            match self.2 {
                Some(SpecialMoveType::Promotion(pt)) => char::from(pt).to_ascii_lowercase().to_string(),
                _ => String::new(),
            }
        )
    }
}

impl fmt::Display for Move {
    /// Converts the move to a UCI string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_uci())
    }
}

/// Represents types of special moves (castling/promotion/en passant).
#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub enum SpecialMoveType {
    CastlingKingside,
    CastlingQueenside,
    /// Represents a promotion, with the tuple value being the type of piece that the pawn promotes to.
    Promotion(PieceType),
    EnPassant,
    Unclear,
}
