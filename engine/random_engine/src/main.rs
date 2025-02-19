use chess::{Board, ChessMove, MoveGen};
use std::io::{self, BufRead, Write};

fn main() {
    let mut board = Board::default();

    loop {

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" {
            break;
        } else if input.starts_with("position") {
        
            let fen = input.strip_prefix("position ").unwrap();
            board = Board::from_fen(fen).unwrap();
        } else if input == "go" {
        
            let mut movegen = MoveGen::new_legal(&board);
            if let Some(chess_move) = movegen.next() {
                println!("{}", chess_move.to_string());
                io::stdout().flush().unwrap();
            }
        }
    }
}
