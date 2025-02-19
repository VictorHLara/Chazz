use chess::{Board, ChessMove, MoveGen, Piece};
use std::io::{self, BufRead, Write};
use std::str::FromStr;

const MAX_DEPTH: u8 = 4; 


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
    for square in chess::ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(*square) {
            let value = piece_value(piece);
            if board.color_on(*square) == Some(board.side_to_move()) {
                score += value;
            } else {
                score -= value;
            }
        }
    }
    score
}


fn minimax(board: &Board, depth: u8, mut alpha: i32, mut beta: i32, maximizing: bool) -> i32 {
    if depth == 0 {
        return evaluate_board(board);
    }

    let movegen = MoveGen::new_legal(board);
    if maximizing {
        let mut max_eval = -9999;
        for m in movegen {
            let new_board = board.make_move_new(m);
            let eval = minimax(&new_board, depth - 1, alpha, beta, false);
            max_eval = max_eval.max(eval);
            alpha = alpha.max(eval);
            if beta <= alpha {
                break;
            }
        }
        max_eval
    } else {
        let mut min_eval = 9999;
        for m in movegen {
            let new_board = board.make_move_new(m);
            let eval = minimax(&new_board, depth - 1, alpha, beta, true);
            min_eval = min_eval.min(eval);
            beta = beta.min(eval);
            if beta <= alpha {
                break;
            }
        }
        min_eval
    }
}


fn best_move(board: &Board) -> Option<ChessMove> {
    let mut best_move = None;
    let mut best_value = -9999;
    let movegen = MoveGen::new_legal(board);
    
    for m in movegen {
        let new_board = board.make_move_new(m);
        let eval = minimax(&new_board, MAX_DEPTH, -10000, 10000, false);
        
        if eval > best_value {
            best_value = eval;
            best_move = Some(m);
        }
    }
    best_move
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
            if let Some(chess_move) = best_move(&board) {
                println!("{}", chess_move);
                io::stdout().flush().unwrap();
            }
        }
    }
}
