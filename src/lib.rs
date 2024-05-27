//! rschess is yet another chess library for Rust, with the aim to be as feature-rich as possible. It is still IN DEVELOPMENT, and NOT FIT FOR USE.

/// The structure for a rschess chessboard
#[derive(Debug)]
pub struct Board {
    /// The board content; each square is represented by a number 0..64 where a1 is 0 and h8 is 63
    content: [Occupant; 64],
    /// The side to move; white is `true` and black is `false`
    side_to_move: bool,
    /// The castling rights for both sides in the format [K, Q, k, q]
    castling_rights: [bool; 4],
    /// The index of the en passant target square, 0..64
    en_passant_target: Option<usize>,
    /// The number of halfmoves since the last pawn push or capture
    halfmove_clock: usize,
    /// The current fullmove number
    fullmove_number: usize,
}

impl Board {
    /// Attempts to construct a `Board` from a standard FEN string, returning an error if the FEN is invalid.
    /// **Shredder-FEN is not supported.**
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut content = [Occupant::Empty; 64];
        let fields: Vec<_> = fen.split(' ').collect();
        let nfields = fields.len();
        if nfields != 6 {
            return Err(format!("Invalid FEN: expected six space-separated FEN fields, got {nfields}"));
        }
        let ranks: Vec<_> = fields[0].split('/').collect();
        let nranks = ranks.len();
        if nranks != 8 {
            return Err(format!("Invalid FEN: expected eight ranks of pieces separated by forward-slashes, got {nranks}"));
        }
        let mut wk_seen = false;
        let mut wk_pos = 0;
        let mut bk_seen = false;
        let mut bk_pos = 0;
        let mut ptr: usize = 63;
        let mut rankn = 8;
        for rank in ranks {
            let mut rank_filled = 0;
            for piece_char in rank.chars().rev() {
                if rank_filled == 8 {
                    return Err(format!("Invalid FEN: rank {rankn} cannot have pieces beyond the h file (8 squares already occupied)"));
                }
                if piece_char.is_ascii_digit() {
                    let empty_space = piece_char.to_digit(10).unwrap() as usize;
                    if !(1..=8).contains(&empty_space) {
                        return Err(format!("Invalid FEN: {empty_space} is not a valid character for board data, digits must be in the range 1..=8"));
                    }
                    if empty_space > 8 - rank_filled {
                        return Err(format!(
                            "Invalid FEN: rank {rankn} only has 8 squares, {rank_filled} of which is/are occupied. {empty_space} more squares of empty space cannot be accomodated"
                        ));
                    }
                    rank_filled += empty_space;
                    ptr = ptr.saturating_sub(empty_space);
                } else {
                    content[ptr] = match piece_char.try_into() {
                        Ok(piece) => {
                            match piece {
                                Piece(PieceType::K, true) => {
                                    if wk_seen {
                                        return Err("Invalid FEN: white cannot have more than one king".to_owned());
                                    }
                                    wk_seen = true;
                                    wk_pos = ptr;
                                }
                                Piece(PieceType::K, false) => {
                                    if bk_seen {
                                        return Err("Invalid FEN: black cannot have more than one king".to_owned());
                                    }
                                    bk_seen = true;
                                    bk_pos = ptr;
                                }
                                _ => (),
                            }
                            Occupant::Piece(piece)
                        }
                        Err(e) => return Err(format!("Invalid FEN: {e}")),
                    };
                    rank_filled += 1;
                    ptr = ptr.saturating_sub(1);
                }
            }
            if rank_filled != 8 {
                return Err(format!("Invalid FEN: rank {rankn} does not have data for 8 squares"));
            }
            rankn -= 1;
        }
        if !(wk_seen && bk_seen) {
            return Err("Invalid FEN: a valid chess position must have one white king and one black king".to_owned());
        }
        let turn = fields[1];
        let side_to_move;
        match turn {
            "w" => side_to_move = true,
            "b" => side_to_move = false,
            _ => return Err(format!("Invalid FEN: Expected second field (side to move) to be 'w' or 'b', got '{turn}'")),
        }
        let castling = fields[2];
        let len_castling = castling.len();
        if !((1..=4).contains(&len_castling)) {
            return Err(format!(
                "Invalid FEN: Expected third field (castling rights) to be 1 to 4 characters long, got {len_castling} characters"
            ));
        }
        let mut castling_rights = [false; 4];
        if castling != "-" {
            for ch in castling.chars() {
                match ch {
                    'K' => {
                        if wk_pos > 6 {
                            return Err("Invalid FEN: White king must be from a1 to g1 to have kingside castling rights".to_owned());
                        }
                        if castling_rights[0] {
                            return Err("Invalid FEN: Found more than one occurrence of 'K' in third field (castling rights)".to_owned());
                        }
                        castling_rights[0] = true;
                    }
                    'Q' => {
                        if !(1..=7).contains(&wk_pos) {
                            return Err("Invalid FEN: White king must be from b1 to h1 to have queenside castling rights".to_owned());
                        }
                        if castling_rights[1] {
                            return Err("Invalid FEN: Found more than one occurrence of 'Q' in third field (castling rights)".to_owned());
                        }
                        castling_rights[1] = true;
                    }
                    'k' => {
                        if !(56..=62).contains(&bk_pos) {
                            return Err("Invalid FEN: Black king must be from a8 to g8 to have kingside castling rights".to_owned());
                        }
                        if castling_rights[2] {
                            return Err("Invalid FEN: Found more than one occurrence of 'k' in third field (castling rights)".to_owned());
                        }
                        castling_rights[2] = true;
                    }
                    'q' => {
                        if !(57..64).contains(&bk_pos) {
                            return Err("Invalid FEN: Black king must be from b8 to h8 to have queenside castling rights".to_owned());
                        }
                        if castling_rights[3] {
                            return Err("Invalid FEN: Found more than one occurrence of 'q' in third field (castling rights)".to_owned());
                        }
                        castling_rights[3] = true;
                    }
                    _ => return Err(format!("Invalid FEN: Expected third field (castling rights) to contain '-' or a subset of 'KQkq', found '{ch}'")),
                }
            }
        }
        fn count_rooks<R>(rng: R, color: bool, content: &[Occupant]) -> usize
        where
            R: std::ops::RangeBounds<usize> + Iterator<Item = usize>,
        {
            let rook = Occupant::Piece(Piece(PieceType::R, color));
            rng.fold(0, |acc, sq| if content[sq] == rook { acc + 1 } else { acc })
        }
        if castling_rights[0] && count_rooks(wk_pos + 1..=7, true, &content) != 1 {
            return Err("Invalid FEN: White must have exactly one king's rook to have kingside castling rights".to_owned());
        }
        if castling_rights[1] && count_rooks(0..wk_pos, true, &content) != 1 {
            return Err("Invalid FEN: White must have exactly one queen's rook to have queenside castling rights".to_owned());
        }
        if castling_rights[2] && count_rooks(bk_pos + 1..64, false, &content) != 1 {
            return Err("Invalid FEN: Black must have exactly one king's rook to have kingside castling rights".to_owned());
        }
        if castling_rights[3] && count_rooks(56..bk_pos, false, &content) != 1 {
            return Err("Invalid FEN: Black must have exactly one queen's rook to have queenside castling rights".to_owned());
        }
        let ep = fields[3];
        let len_ep = ep.len();
        if !((1..=2).contains(&len_ep)) {
            return Err(format!(
                "Invalid FEN: Expected fourth field (en passant target square) to be 1 to 2 characters long, got {len_ep} characters"
            ));
        }
        let mut en_passant_target = None;
        if ep != "-" {
            let err = Err(format!(
                "Invalid FEN: Expected fourth field (en passant target square) to be '-' or a valid en passant target square name, '{ep}' is not a valid en passant target square name"
            ));
            if len_ep != 2 {
                return err;
            }
            let file = ep.chars().next().unwrap();
            let rank = ep.chars().nth(1).unwrap();
            if !(('a'..='h').contains(&file) && ['3', '6'].contains(&rank)) {
                return err;
            }
            en_passant_target = Some(Self::sq_to_idx(file, rank));
        }
        let halfmoves = fields[4];
        let halfmove_clock: usize = halfmoves
            .parse()
            .map_err(|_| format!("Invalid FEN: Expected fifth field (halfmove clock) to be a whole number, got '{halfmoves}'"))?;
        if halfmove_clock > 150 {
            return Err(format!(
                "Invalid FEN: Fifth field (halfmove clock) cannot contain a value greater than 150 (the seventy-five-move rule forces a draw when this reaches 150), got {halfmove_clock}"
            ));
        }
        let fullmoves = fields[5];
        let fullmove_number: usize = fullmoves
            .parse()
            .map_err(|_| format!("Invalid FEN: Expected sixth field (fullmove number) to be a natural number, got '{fullmoves}'"))?;
        if fullmove_number < 1 {
            return Err(format!(
                "Invalid FEN: Sixth field (fullmove number) cannot contain a value less than 1 (it starts at 1 and increments after Black's move), got {fullmove_number}"
            ));
        }
        Ok(Self {
            content,
            side_to_move,
            castling_rights,
            en_passant_target,
            halfmove_clock,
            fullmove_number,
        })
    }

    /// Returns the representation of the board state in standard FEN.
    pub fn to_fen(&self) -> String {
        let mut rankstrs = Vec::new();
        for rank in self.content.chunks(8).rev() {
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
        let active_color = if self.side_to_move { "w".to_owned() } else { "b".to_owned() };
        let mut castling_availability = String::new();
        if self.castling_rights[0] {
            castling_availability.push('K');
        }
        if self.castling_rights[1] {
            castling_availability.push('Q');
        }
        if self.castling_rights[2] {
            castling_availability.push('k');
        }
        if self.castling_rights[3] {
            castling_availability.push('q');
        }
        if castling_availability.is_empty() {
            castling_availability.push('-');
        }
        let en_passant_target_square;
        if let Some(target) = self.en_passant_target {
            let (f, r) = Self::idx_to_sq(target);
            en_passant_target_square = [f.to_string(), r.to_string()].join("");
        } else {
            en_passant_target_square = "-".to_owned();
        }
        [
            board_data,
            active_color,
            castling_availability,
            en_passant_target_square,
            self.halfmove_clock.to_string(),
            self.fullmove_number.to_string(),
        ]
        .join(" ")
    }

    fn sq_to_idx(file: char, rank: char) -> usize {
        (rank.to_digit(10).unwrap() as usize - 1) * 8 + (file as usize - 97)
    }

    fn idx_to_sq(idx: usize) -> (char, char) {
        ((idx % 8 + 97) as u8 as char, char::from_digit((idx / 8 + 1) as u32, 10).unwrap())
    }

    /// Generates pseudolegal moves based on the board data, castling rights, available en passant target, and the side to move.
    fn gen_pseudolegal_moves(content: &[Occupant; 64], castling_rights: &[bool; 4], ep_target: Option<usize>, side: bool) -> Vec<Move> {
        let castling_idx_offset = if side { 0 } else { 2 };
        for (i, sq) in content.into_iter().enumerate() {
            if let Occupant::Piece(piece) = sq {
                if piece.1 != side {
                    continue;
                }
                match piece.0 {
                    PieceType::K => {
                        let mut possible_dests = Vec::new();
                        if i + 1 % 8 != 0 {
                            possible_dests.push(i + 1);
                        }
                        if i % 8 != 0 {
                            possible_dests.push(i - 1);
                        }
                        if i < 56 {
                            possible_dests.push(i + 8);
                        }
                        if i > 7 {
                            possible_dests.push(i - 8);
                        }
                        if !Self::king_capture_possible(content) {
                            if castling_rights[castling_idx_offset] {
                                todo!()
                            }
                            if castling_rights[castling_idx_offset + 1] {
                                todo!()
                            }
                        }
                        possible_dests = possible_dests
                            .into_iter()
                            .filter(|&dest| match content[dest] {
                                Occupant::Piece(p) => p.1 != side,
                                _ => true,
                            })
                            .collect();
                    }
                    PieceType::Q => todo!(),
                    PieceType::B => todo!(),
                    PieceType::N => todo!(),
                    PieceType::R => todo!(),
                    PieceType::P => todo!(),
                }
            }
        }
        todo!()
    }

    fn king_capture_possible(content: &[Occupant; 64]) -> bool {
        todo!()
    }
}

impl Default for Board {
    /// Constructs a `Board` with the starting position for a chess game.
    fn default() -> Self {
        Self::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Occupant {
    Piece(Piece),
    Empty,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
struct Piece(PieceType, bool);

impl TryFrom<char> for Piece {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if !value.is_ascii_alphanumeric() {
            return Err(format!("Invalid piece character: '{value}' is not ASCII alphanumeric"));
        }
        let color = value.is_uppercase();
        Ok(Self(
            match value.to_ascii_lowercase() {
                'k' => PieceType::K,
                'q' => PieceType::Q,
                'b' => PieceType::B,
                'n' => PieceType::N,
                'r' => PieceType::R,
                'p' => PieceType::P,
                _ => return Err(format!("Invalid piece character: '{value}' does not correspond to any chess piece")),
            },
            color,
        ))
    }
}

impl From<Piece> for char {
    fn from(piece: Piece) -> char {
        let ch = match piece.0 {
            PieceType::K => 'k',
            PieceType::Q => 'q',
            PieceType::B => 'b',
            PieceType::N => 'n',
            PieceType::R => 'r',
            PieceType::P => 'p',
        };
        if piece.1 {
            ch.to_ascii_uppercase()
        } else {
            ch
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum PieceType {
    K,
    Q,
    B,
    N,
    R,
    P,
}

/// The structure for a chess move, in the format (<source square>, <destination square>)
pub struct Move(usize, usize);

#[cfg(test)]
mod tests {
    use super::Board;

    #[test]
    fn default_board() {
        println!("{:?}", Board::default());
    }

    #[test]
    fn valid_fen() {
        Board::from_fen("6k1/8/6K1/6P1/8/8/8/8 w - - 0 1").unwrap();
        Board::from_fen("k5rb/8/8/4P3/3p4/8/8/K5BR w Kk - 0 1").unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_fen() {
        Board::from_fen("what").unwrap();
        Board::from_fen("blafsd o fs o sdo d").unwrap();
        Board::from_fen("rnbkkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RKBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQQBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8p/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppxpp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP-RNBQKBNR w KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR lol KQkq - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b ros - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KKqk - 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq C6 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq c5 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq brr 0 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0.1 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 151 1").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 150 0").unwrap();
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 150 bro").unwrap();
        Board::from_fen("k5rb/8/8/4P3/3p4/8/8/K5BR w KQkq - 0 1").unwrap();
    }

    #[test]
    fn idx_sq_conversion() {
        assert_eq!(Board::sq_to_idx('f', '5'), 37);
        assert_eq!(Board::idx_to_sq(37), ('f', '5'));
        assert_eq!(Board::sq_to_idx('g', '2'), 14);
        assert_eq!(Board::idx_to_sq(14), ('g', '2'));
        assert_eq!(Board::sq_to_idx('c', '6'), 42);
        assert_eq!(Board::idx_to_sq(42), ('c', '6'));
    }

    #[test]
    fn board_to_fen() {
        assert_eq!(Board::from_fen("6k1/8/6K1/6P1/8/8/8/8 w - - 0 1").unwrap().to_fen(), "6k1/8/6K1/6P1/8/8/8/8 w - - 0 1");
        assert_eq!(Board::from_fen("k5rb/8/8/4P3/3p4/8/8/K5BR w Kk - 0 1").unwrap().to_fen(), "k5rb/8/8/4P3/3p4/8/8/K5BR w Kk - 0 1");
        assert_eq!(Board::default().to_fen(), Board::default().to_fen());
    }
}
