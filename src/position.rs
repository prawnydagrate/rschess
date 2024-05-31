use std::collections::HashMap;

use super::{helpers, Move, Occupant, Piece, PieceType, SpecialMoveType};

/// The structure for a chess position
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Position {
    /// The board content; each square is represented by a number 0..64 where a1 is 0 and h8 is 63
    pub content: [Occupant; 64],
    /// The side to move; white is `true` and black is `false`
    pub side: bool,
    /// The indices of rook locations representing castling rights for both sides in the format [K, Q, k, q]
    pub castling_rights: [Option<usize>; 4],
    /// The index of the en passant target square, 0..64
    pub ep_target: Option<usize>,
}

impl Position {
    /// Generates an FEN string representing the board data, active color, castling rights, and en passant target in the position.
    pub fn to_fen(&self) -> String {
        let Position {
            content,
            side,
            castling_rights,
            ep_target,
        } = self;
        let mut rankstrs = Vec::new();
        for rank in content.chunks(8).rev() {
            let mut rankstr = String::new();
            let mut empty = 0;
            for sq in rank {
                match sq {
                    Occupant::Piece(p) => {
                        if empty > 0 {
                            rankstr.push(char::from_digit(empty, 10).unwrap());
                            empty = 0;
                        }
                        rankstr.push((*p).into());
                    }
                    Occupant::Empty => {
                        empty += 1;
                    }
                }
            }
            if empty > 0 {
                rankstr.push(char::from_digit(empty, 10).unwrap());
            }
            rankstrs.push(rankstr);
        }
        let board_data = rankstrs.join("/");
        let active_color = if *side { "w".to_owned() } else { "b".to_owned() };
        let mut castling_availability = String::new();
        let count_rooks = |rng, color| helpers::count_piece(rng, Piece(PieceType::R, color), content);
        let (wk, bk) = (helpers::find_king(true, content), helpers::find_king(false, content));
        if castling_rights[0].is_some() {
            castling_availability.push(if count_rooks(wk + 1..8, true) == 1 {
                'K'
            } else {
                helpers::idx_to_sq(castling_rights[0].unwrap()).0.to_ascii_uppercase()
            });
        }
        if castling_rights[1].is_some() {
            castling_availability.push(if count_rooks(0..wk, true) == 1 {
                'Q'
            } else {
                helpers::idx_to_sq(castling_rights[1].unwrap()).0.to_ascii_uppercase()
            });
        }
        if castling_rights[2].is_some() {
            castling_availability.push(if count_rooks(bk + 1..64, false) == 1 {
                'k'
            } else {
                helpers::idx_to_sq(castling_rights[2].unwrap()).0
            });
        }
        if castling_rights[3].is_some() {
            castling_availability.push(if count_rooks(56..bk, false) == 1 {
                'q'
            } else {
                helpers::idx_to_sq(castling_rights[2].unwrap()).0
            });
        }
        if castling_availability.is_empty() {
            castling_availability.push('-');
        }
        let en_passant_target_square;
        if let Some(target) = ep_target {
            let (f, r) = helpers::idx_to_sq(*target);
            en_passant_target_square = [f.to_string(), r.to_string()].join("");
        } else {
            en_passant_target_square = "-".to_owned();
        }
        [board_data, active_color, castling_availability, en_passant_target_square].join(" ")
    }

    /// Pretty-prints the position to a string, from the perspective of the side `perspective`.
    pub fn pretty_print(&self, perspective: bool) -> String {
        let mut string = String::new();
        let codepoints = HashMap::from([
            (PieceType::K, 0x2654),
            (PieceType::Q, 0x2655),
            (PieceType::R, 0x2656),
            (PieceType::B, 0x2657),
            (PieceType::N, 0x2658),
            (PieceType::P, 0x2659),
        ]);
        if perspective {
            for (ranki, rank) in self.content.chunks(8).rev().enumerate() {
                string += &format!("{} |", 8 - ranki);
                for (sqi, occupant) in rank.iter().enumerate() {
                    string += &format!(
                        " {} ",
                        match occupant {
                            Occupant::Piece(Piece(t, c)) => char::from_u32((codepoints.get(t).unwrap() + if *c { 0 } else { 6 }) as u32).unwrap(),
                            Occupant::Empty => ' ',
                        }
                    );
                    if sqi != 7 {
                        string.push('|');
                    }
                }
                string.push('\n');
                string += &"⎯".repeat(33);
                string.push('\n');
            }
            string += "  | a | b | c | d | e | f | g | h";
        } else {
            for (ranki, rank) in self.content.chunks(8).enumerate() {
                string += &format!("{} |", ranki + 1);
                for (sqi, occupant) in rank.iter().rev().enumerate() {
                    string += &format!(
                        " {} ",
                        match occupant {
                            Occupant::Piece(Piece(t, c)) => char::from_u32((codepoints.get(t).unwrap() + if *c { 0 } else { 6 }) as u32).unwrap(),
                            Occupant::Empty => ' ',
                        }
                    );
                    if sqi != 7 {
                        string.push('|');
                    }
                }
                string.push('\n');
                string += &"⎯".repeat(33);
                string.push('\n');
            }
            string += "  | h | g | f | e | d | c | d | a";
        }
        string
    }

    /// Generates the legal moves in the position, assuming the game is ongoing.
    pub fn gen_non_illegal_moves(&self) -> Vec<Move> {
        let Position { content, side, castling_rights, .. } = self;
        self.gen_pseudolegal_moves()
            .into_iter()
            .filter(|move_| {
                if let Move(src, dest, Some(SpecialMoveType::CastlingKingside | SpecialMoveType::CastlingQueenside)) = move_ {
                    for sq in *std::cmp::min(src, dest)..=*std::cmp::max(src, dest) {
                        if self.controls_square(sq, !side) {
                            return false;
                        }
                    }
                    return true;
                }
                !helpers::king_capture_pseudolegal(&helpers::change_content(content, move_, castling_rights), !side)
            })
            .collect()
    }

    /// Checks whether the game is drawn by stalemate. Use [`Board::stalemated_side`] to know which side is in stalemate.
    pub fn is_stalemate(&self) -> bool {
        !self.is_check() && self.gen_non_illegal_moves().is_empty()
    }

    /// Checks whether any side is in check (a checkmate is also considered a check). Use [`Board::checked_side`] to know which side is in check.
    pub fn is_check(&self) -> bool {
        self.checked_side().is_some()
    }

    /// Checks whether any side is in checkmate. Use [`Board::checkmated_side`] to know which side is in checkmate.
    pub fn is_checkmate(&self) -> bool {
        self.is_check() && self.gen_non_illegal_moves().is_empty()
    }

    /// Returns an optional boolean representing the side in stalemate (`None` if neither side is in stalemate).
    pub fn stalemated_side(&self) -> Option<bool> {
        if self.is_stalemate() {
            Some(self.side)
        } else {
            None
        }
    }

    /// Returns an optional boolean representing the side in check (`None` if neither side is in check).
    pub fn checked_side(&self) -> Option<bool> {
        if helpers::king_capture_pseudolegal(&self.content, false) {
            Some(true)
        } else if helpers::king_capture_pseudolegal(&self.content, true) {
            Some(false)
        } else {
            None
        }
    }

    /// Returns an optional boolean representing the side in checkmate (`None` if neither side is in checkmate).
    pub fn checkmated_side(&self) -> Option<bool> {
        if self.is_checkmate() {
            Some(self.side)
        } else {
            None
        }
    }

    /// Generates the pseudolegal moves in the position.
    pub fn gen_pseudolegal_moves(&self) -> Vec<Move> {
        let Self {
            content,
            castling_rights,
            ep_target,
            side,
        } = self;
        let mut pseudolegal_moves = Vec::new();
        for (i, sq) in content.iter().enumerate() {
            if let Occupant::Piece(piece) = sq {
                if piece.1 != *side {
                    continue;
                }
                match piece.0 {
                    PieceType::K => {
                        let mut possible_dests = Vec::new();
                        for axis in [1, 8, 7, 9] {
                            if helpers::long_range_can_move(i, axis as isize) {
                                possible_dests.push(i + axis);
                            }
                            if helpers::long_range_can_move(i, -(axis as isize)) {
                                possible_dests.push(i - axis);
                            }
                        }
                        possible_dests.retain(|&dest| match content[dest] {
                            Occupant::Piece(Piece(_, color)) => color != *side,
                            _ => true,
                        });
                        pseudolegal_moves.extend(possible_dests.into_iter().map(|d| Move(i, d, None)));
                        let castling_rights_idx_offset = if *side { 0 } else { 2 };
                        let (oo_sq, ooo_sq) = if *side { (6, 2) } else { (62, 58) };
                        let (kingside, queenside) = (castling_rights[castling_rights_idx_offset], castling_rights[castling_rights_idx_offset + 1]);
                        if let Some(r) = kingside {
                            match helpers::count_pieces(i + 1..=oo_sq, content) {
                                0 => pseudolegal_moves.push(Move(i, oo_sq, Some(SpecialMoveType::CastlingKingside))),
                                1 => {
                                    if helpers::find_all_pieces(i + 1..=oo_sq, content)[0] == r {
                                        pseudolegal_moves.push(Move(i, oo_sq, Some(SpecialMoveType::CastlingKingside)))
                                    }
                                }
                                _ => (),
                            }
                        }
                        if let Some(r) = queenside {
                            match helpers::count_pieces(ooo_sq..i, content) {
                                0 => pseudolegal_moves.push(Move(i, ooo_sq, Some(SpecialMoveType::CastlingQueenside))),
                                1 => {
                                    if helpers::find_all_pieces(ooo_sq..i, content)[0] == r {
                                        pseudolegal_moves.push(Move(i, ooo_sq, Some(SpecialMoveType::CastlingQueenside)))
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                    PieceType::N => {
                        let b_r_axes = [(7, [-1, 8]), (9, [8, 1]), (-7, [1, -8]), (-9, [-8, -1])];
                        let mut dest_squares = Vec::new();
                        for (b_axis, r_axes) in b_r_axes {
                            if !helpers::long_range_can_move(i, b_axis) {
                                continue;
                            }
                            let b_dest = i as isize + b_axis;
                            for r_axis in r_axes {
                                if !helpers::long_range_can_move(b_dest as usize, r_axis) {
                                    continue;
                                }
                                dest_squares.push((b_dest + r_axis) as usize);
                            }
                        }
                        pseudolegal_moves.extend(
                            dest_squares
                                .into_iter()
                                .filter(|&dest| match content[dest] {
                                    Occupant::Piece(Piece(_, color)) => color != *side,
                                    _ => true,
                                })
                                .map(|dest| Move(i, dest, None)),
                        )
                    }
                    PieceType::P => {
                        let mut possible_dests = Vec::new();
                        if *side {
                            if let Occupant::Empty = content[i + 8] {
                                possible_dests.push((i + 8, false));
                                if (8..16).contains(&i) && content[i + 16] == Occupant::Empty {
                                    possible_dests.push((i + 16, false))
                                }
                            }
                            if helpers::long_range_can_move(i, 7) {
                                if let Occupant::Piece(Piece(_, color)) = content[i + 7] {
                                    if !color {
                                        possible_dests.push((i + 7, false));
                                    }
                                } else if ep_target.is_some() && ep_target.unwrap() == i + 7 {
                                    possible_dests.push((i + 7, true));
                                }
                            }
                            if helpers::long_range_can_move(i, 9) {
                                if let Occupant::Piece(Piece(_, color)) = content[i + 9] {
                                    if !color {
                                        possible_dests.push((i + 9, false));
                                    }
                                } else if ep_target.is_some() && ep_target.unwrap() == i + 9 {
                                    possible_dests.push((i + 9, true));
                                }
                            }
                        } else {
                            if let Occupant::Empty = content[i - 8] {
                                possible_dests.push((i - 8, false));
                                if (48..56).contains(&i) && content[i - 16] == Occupant::Empty {
                                    possible_dests.push((i - 16, false))
                                }
                            }
                            if helpers::long_range_can_move(i, -9) {
                                if let Occupant::Piece(Piece(_, color)) = content[i - 9] {
                                    if color {
                                        possible_dests.push((i - 9, false));
                                    }
                                } else if ep_target.is_some() && ep_target.unwrap() == i - 9 {
                                    possible_dests.push((i - 9, true));
                                }
                            }
                            if helpers::long_range_can_move(i, -7) {
                                if let Occupant::Piece(Piece(_, color)) = content[i - 7] {
                                    if color {
                                        possible_dests.push((i - 7, false));
                                    }
                                } else if ep_target.is_some() && ep_target.unwrap() == i - 7 {
                                    possible_dests.push((i - 7, true));
                                }
                            }
                        }
                        pseudolegal_moves.extend(possible_dests.into_iter().flat_map(|(dest, ep)| {
                            if (0..8).contains(&dest) || (56..64).contains(&dest) {
                                [PieceType::Q, PieceType::R, PieceType::B, PieceType::N]
                                    .into_iter()
                                    .map(|p| Move(i, dest, Some(SpecialMoveType::Promotion(p))))
                                    .collect()
                            } else {
                                vec![Move(i, dest, if ep { Some(SpecialMoveType::EnPassant) } else { None })]
                            }
                        }));
                    }
                    long_range_type => pseudolegal_moves.append(&mut self.gen_long_range_piece_pseudolegal_moves(i, long_range_type)),
                }
            }
        }
        pseudolegal_moves
    }

    /// Generates pseudolegal moves for a long-range piece.
    pub fn gen_long_range_piece_pseudolegal_moves(&self, sq: usize, piece_type: PieceType) -> Vec<Move> {
        let Self { content, side, .. } = self;
        let axes = match piece_type {
            PieceType::Q => vec![1, 8, 7, 9],
            PieceType::R => vec![1, 8],
            PieceType::B => vec![7, 9],
            _ => panic!("not a long-range piece"),
        };
        let mut dest_squares = Vec::new();
        for axis in axes {
            'axis: for axis_direction in [-axis, axis] {
                let mut current_sq = sq as isize;
                while helpers::long_range_can_move(current_sq as usize, axis_direction) {
                    let mut skip = false;
                    current_sq += axis_direction;
                    if let Occupant::Piece(Piece(_, color)) = content[current_sq as usize] {
                        if color == *side {
                            continue 'axis;
                        } else {
                            skip = true;
                        }
                    }
                    dest_squares.push(current_sq as usize);
                    if skip {
                        continue 'axis;
                    }
                }
            }
        }
        dest_squares.into_iter().map(|dest| Move(sq, dest, None)).collect()
    }

    /// Checks whether the given side controls a specified square in this position.
    pub fn controls_square(&self, sq: usize, side: bool) -> bool {
        let Self {
            mut content,
            castling_rights,
            ep_target,
            ..
        } = self.clone();
        content[sq] = Occupant::Piece(Piece(PieceType::P, !side));
        Self {
            content,
            side,
            castling_rights,
            ep_target,
        }
        .gen_pseudolegal_moves()
        .into_iter()
        .any(|Move(_, dest, _)| dest == sq)
    }

    /// Counts the material on the board. This function is used by [`Position::is_insufficient_material`] to determine whether there is insufficient checkmating material.
    pub fn count_material(&self) -> Vec<Material> {
        let mut material = Vec::new();
        for sq in 0..64 {
            if let Occupant::Piece(Piece(piece_type, _)) = self.content[sq] {
                match piece_type {
                    PieceType::K => (),
                    PieceType::N => material.push(Material::Knight),
                    PieceType::B => material.push(Material::Bishop(helpers::color_complex_of(sq))),
                    _ => material.push(Material::Other),
                }
            }
        }
        material
    }

    pub fn is_insufficient_material(&self) -> bool {
        let copy1 = self.count_material();
        let (mut copy2, copy3, mut copy4) = (copy1.clone(), copy1.clone(), copy1.clone());
        if copy1.is_empty() {
            return true;
        }
        for (i, m) in copy2.iter().enumerate() {
            if let Material::Knight = m {
                copy2.remove(i);
                break;
            }
        }
        if copy2.is_empty() {
            return true;
        }
        let mut b_complex = None;
        for m in copy3.iter() {
            if let Material::Bishop(complex) = m {
                b_complex = Some(complex);
                break;
            }
        }
        if let Some(complex) = b_complex {
            copy4.retain(|m| m != &Material::Bishop(*complex));
            if copy4.is_empty() {
                return true;
            }
        }
        false
    }
}

/// Represents a piece of material.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Material {
    Knight,
    Bishop(bool),
    Other,
}