//! # A [helpmate](https://en.wikipedia.org/wiki/Helpmate) solver using rschess
//! **NOTE**: rschess prioritizes feature-richness and ease of use over performance,
//! which makes it unsuitable for this purpose.

use rschess::{Board, Fen};
use std::{
    env, fmt,
    io::{self, Error as IoError, Write},
    process,
    time::Instant,
};

fn input(prompt: &str) -> Result<String, IoError> {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut resp = String::new();
    io::stdin().read_line(&mut resp)?;
    Ok(resp)
}

fn error<T: fmt::Display>(e: T) -> ! {
    eprintln!("{e}");
    process::exit(1);
}

fn search(board: Board, maxdepth: usize) -> Vec<Board> {
    if maxdepth == 0 {
        return Vec::new();
    }
    let mut sols = Vec::new();
    for m in board.gen_legal_moves() {
        let mut new_board = board.clone();
        new_board.make_move(m).unwrap();
        if new_board.is_checkmate() && new_board.checkmated_side().unwrap().is_black() {
            sols.push(new_board)
        } else {
            sols.append(&mut search(new_board, maxdepth - 1))
        }
    }
    sols
}

fn main() {
    let maxdepth: usize = if let Some(n) = env::args().nth(1) { n.parse().expect("Invalid max depth") } else { 4 };
    // let fen = Fen::try_from(input("Enter the position FEN: ").expect("Failed to read user input").as_str()).unwrap_or_else(|e| error(e));
    let fen = Fen::try_from("1RrB2b1/8/4n3/2n3p1/2K2b2/1p1rk3/6BR/8 b - - 1 1").unwrap();
    if fen.position().side_to_move().is_white() {
        error("Must be black to move");
    }
    let board = Board::from_fen(fen);
    println!("{board}\nSearching for helpmates (depth = {maxdepth})...");
    let start = Instant::now();
    let sols = search(board, maxdepth);
    let end = Instant::now();
    let dur = end.duration_since(start);
    println!("Finished searching in {dur:?}");
    if sols.is_empty() {
        println!("No solutions found");
    } else {
        println!("Solution(s):");
        for sol in sols {
            println!("{}", sol.gen_movetext());
        }
    }
}
