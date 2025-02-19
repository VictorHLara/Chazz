use chess::{Board, ChessMove, MoveGen, Piece};
use std::io::{self, BufRead, Write};
use std::str::FromStr;


fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 1,
        Piece::Knight | Piece::Bishop => 3,
        Piece::Rook => 5,
        Piece::Queen => 9,
        Piece::King => 1000, 
    }
}


fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;
    for &square in chess::ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(square) {
            let value = piece_value(piece);
            if board.color_on(square) == Some(board.side_to_move()) {
                score += value;
            } else {
                score -= value;
            }
        }
    }
    score
}


fn minimax(board: &Board, depth: u32, maximizing: bool) -> (i32, Option<ChessMove>) {
    if depth == 0 {
        return (evaluate_board(board), None);
    }
    
    let movegen = MoveGen::new_legal(board);
    let mut best_move = None;
    
    if maximizing {
        let mut max_eval = i32::MIN;
        for chess_move in movegen {
            let mut new_board = board.clone();
            new_board = new_board.make_move_new(chess_move);
            let (eval, _) = minimax(&new_board, depth - 1, false);
            if eval > max_eval {
                max_eval = eval;
                best_move = Some(chess_move);
            }
        }
        (max_eval, best_move)
    } else {
        let mut min_eval = i32::MAX;
        for chess_move in movegen {
            let mut new_board = board.clone();
            new_board = new_board.make_move_new(chess_move);
            let (eval, _) = minimax(&new_board, depth - 1, true);
            if eval < min_eval {
                min_eval = eval;
                best_move = Some(chess_move);
            }
        }
        (min_eval, best_move)
    }
}

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
            if let Ok(new_board) = Board::from_str(fen) {
                board = new_board;
            } else {
                eprintln!("Erro: FEN inválida");
            }
        } else if input == "go" {
     
            let (_, best_move) = minimax(&board, 3, true); 
            if let Some(chess_move) = best_move {
                println!("{}", chess_move);
                io::stdout().flush().unwrap();
            }
        }
    }
}
