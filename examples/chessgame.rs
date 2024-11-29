//! # A simple player vs. computer chess game using rschess
//! The computer plays a random legal move in response to the
//! user's every move.

use rand::{seq::SliceRandom, thread_rng};
use rschess::{Board, Color, DrawType, GameResult};
use std::io::{self, Error as IoError, Write};

fn input(prompt: &str, placeholder: &str, multiline: bool) -> Result<String, IoError> {
    let phlen = placeholder.len();
    print!("{prompt}{}\x1b[{}D", placeholder, phlen);
    let _ = io::stdout().flush();
    let mut resp = String::new();
    io::stdin().read_line(&mut resp)?;
    resp = resp.chars().take(resp.len() - 1).collect();
    let resplen = resp.len();
    if resplen < phlen {
        print!("\x1b[1A\x1b[{}C{}\n", if multiline { resplen } else { prompt.len() + resplen }, " ".repeat(phlen - resplen));
        let _ = io::stdout().flush();
    }
    Ok(resp)
}

fn get_user_starting_color() -> bool {
    loop {
        let user_color_resp = input("Would you like to play white or black (default: white)?\n", r#"_____ ("white" or "black")"#, true)
            .expect("Failed to get user input")
            .trim()
            .to_lowercase();
        match user_color_resp.as_str() {
            "" => return true,
            "w" | "wh" | "whi" | "whit" | "wit" | "white" | "wite" | "wt" | "wht" => return true,
            "b" | "bl" | "bla" | "blac" | "blc" | "blk" | "blak" | "black" => return false,
            _ => println!(r#"Expected "white" or "black", got "{user_color_resp}"{}"#, '\n'),
        }
    }
}

fn get_ascii_or_u() -> bool {
    loop {
        let user_chars_resp = input(
            "Should the pieces be printed with Unicode or ASCII characters (default: Unicode)?\n",
            r#"_______ ("Unicode" or "ASCII")"#,
            true,
        )
        .expect("Failed to get user input")
        .trim()
        .to_lowercase();
        match user_chars_resp.as_str() {
            "" => return false,
            "u" | "un" | "uni" | "unic" | "uc" | "unico" | "unicod" | "ucd" | "uncd" | "unicode" => return false,
            "a" | "as" | "asc" | "ac" | "asci" | "asi" | "aci" | "ascii" | "asii" | "acii" => return true,
            _ => println!(r#"Expected "Unicode" or "ASCII", got "{user_chars_resp}"{}"#, '\n'),
        }
    }
}

fn ask_move(board: &mut Board, ascii: bool) {
    println!("{}\n", board.pretty_print(board.side_to_move(), ascii));
    let legal = board.gen_legal_moves();
    loop {
        let user_move_resp = input(
            "Please enter your move: ",
            &format!(r#"____ (e.g. "{}")"#, board.move_to_san(legal.choose(&mut thread_rng()).unwrap().clone()).unwrap()),
            false,
        )
        .expect("Failed to get user input")
        .trim()
        .to_owned();
        if user_move_resp.is_empty() {
            continue;
        }
        if board.make_move_san(&user_move_resp).is_err() {
            println!("Sorry, this move is illegal. These are your legal moves:");
            let nlegal = legal.len();
            let padlen = nlegal.to_string().len();
            for n in 1..=nlegal {
                println!("{n: >padlen$}. {}", board.move_to_san(legal[n - 1]).unwrap());
            }
            println!();
            continue;
        };
        println!();
        break;
    }
}

fn main() {
    let user_starting_color = get_user_starting_color();
    let ascii = get_ascii_or_u();
    let mut board = Board::default();
    let mut rng = thread_rng();
    println!("\nYou have the {} pieces.\n", if user_starting_color { "white" } else { "black" });
    let mut user_turn = user_starting_color;
    while board.is_ongoing() {
        if user_turn {
            ask_move(&mut board, ascii);
        } else {
            let move_ = board.gen_legal_moves().choose(&mut rng).unwrap().clone();
            let san = board.move_to_san(move_.clone()).unwrap();
            board.make_move(move_).unwrap();
            println!("The computer played {san}.");
        }
        user_turn = !user_turn;
    }
    println!("Game over!");
    println!("{}", board.pretty_print(if user_starting_color { Color::White } else { Color::Black }, ascii));
    let res = board.game_result().unwrap();
    println!(
        "Result: {}",
        match res {
            GameResult::Wins(c, _) => format!("{} won by checkmate", if c.is_white() == user_starting_color { "You" } else { "The computer" }),
            GameResult::Draw(DrawType::Stalemate(c)) => format!("Draw by stalemate ({} stalemated)", if c.is_white() == user_starting_color { "you were" } else { "the computer was" }),
            GameResult::Draw(DrawType::InsufficientMaterial) => "Draw by insufficient checkmating material".to_owned(),
            GameResult::Draw(DrawType::FivefoldRepetition) => "Draw by fivefold repetition".to_owned(),
            GameResult::Draw(DrawType::SeventyFiveMoveRule) => "Draw by the seventy-five-move rule".to_owned(),
            _ => panic!("the universe is malfunctioning"),
        }
    );
    println!(
        "{} {}",
        board.gen_movetext(),
        match res {
            GameResult::Wins(c, _) =>
                if c.is_white() {
                    "1-0"
                } else {
                    "0-1"
                },
            GameResult::Draw(_) => "1/2-1/2",
        }
    );
}
