use chess::{Board, BoardStatus, ChessMove, MoveGen, Piece, Square};
use std::io::{self, BufRead, Write};
use std::str::FromStr;
use std::collections::HashMap;

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 1,
        Piece::Knight | Piece::Bishop => 3,
        Piece::Rook => 5,
        Piece::Queen => 9,
        Piece::King => 1000,
    }
}


const CENTER_SQUARES: [Square; 4] = [Square::D4, Square::D5, Square::E4, Square::E5];


fn evaluate_board(board: &Board) -> i32 {
    let mut score = 0;
    let side_to_move = board.side_to_move();

    // Avaliar material e controle do centro
    for square in chess::ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(*square) {
            let value = piece_value(piece);
            if board.color_on(*square) == Some(side_to_move) {
                score += value;
                // Bônus para controle do centro
                if CENTER_SQUARES.contains(square) {
                    score += 2;
                }
            } else {
                score -= value;
                if CENTER_SQUARES.contains(square) {
                    score -= 2;
                }
            }
        }
    }

    // Verificar status do jogo
    match board.status() {
        BoardStatus::Checkmate => {
            // IMPORTANTE: Se estamos em xeque-mate, quem tem que mover perdeu
            return -20000; // Valor negativo muito alto, pois xeque-mate é ruim para quem deve mover
        }
        BoardStatus::Stalemate => {
            return 0; // Empate
        }
        _ => {
            // Bônus para xeque
            if board.checkers().popcnt() > 0 {
                score -= 30; // Estar em xeque é ruim para o lado que vai mover
            }
            
            // Bônus para mobilidade
            let mobility = MoveGen::new_legal(board).count() as i32;
            score += mobility / 10; // Mais movimentos é melhor para quem vai mover
        }
    }

    score
}

fn alpha_beta(
    board: &Board,
    depth: i32,
    mut alpha: i32,
    mut beta: i32,
    maximizing: bool,
    repetitions: &mut HashMap<u64, i32>,
) -> i32 {
    // Se chegamos à profundidade máxima ou fim de jogo, avaliar a posição
    if depth == 0 || board.status() != BoardStatus::Ongoing {
        return if maximizing { evaluate_board(board) } else { -evaluate_board(board) };
    }

    let hash = board.get_hash();
    let count = repetitions.get(&hash).cloned().unwrap_or(0);
    if count >= 3 {
        return 0; // Empate por repetição
    }
    repetitions.insert(hash, count + 1);

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    
    // Ordenar movimentos (capturas, xeques e xeque-mates primeiro)
    moves.sort_by_key(|m| {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(*m);
        
        let mut score = 0;
        
        // Verificar se captura alguma peça
        if let Some(captured) = board.piece_on(m.get_dest()) {
            score += piece_value(captured) * 10;
        }
        
        // Verificar se é uma promoção
        if m.get_promotion().is_some() {
            score += 800;
        }
        
        // Verificar se é xeque
        if new_board.checkers().popcnt() > 0 {
            score += 100;
        }
        
        // Verificar se é xeque-mate (maior prioridade)
        if new_board.status() == BoardStatus::Checkmate {
            score += 20000;
        }
        
        score
    });
    moves.reverse(); // Prioriza maiores valores primeiro

    let mut best_eval = if maximizing { -100000 } else { 100000 };

    for chess_move in moves {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(chess_move);

        let eval = alpha_beta(&new_board, depth - 1, alpha, beta, !maximizing, repetitions);

        if maximizing {
            best_eval = best_eval.max(eval);
            alpha = alpha.max(eval);
        } else {
            best_eval = best_eval.min(eval);
            beta = beta.min(eval);
        }

        if beta <= alpha {
            break; // Poda alfa-beta
        }
    }

    repetitions.insert(hash, count); // Reverte a contagem
    best_eval
}



fn best_move(board: &Board) -> Option<ChessMove> {
    let mut best_move = None;
    let mut best_value = -100000;
    let mut repetitions = HashMap::new();
    
    let movegen = MoveGen::new_legal(board);
    let moves: Vec<ChessMove> = movegen.collect();
    
    // Ajuste a profundidade conforme necessário
    let depth = 3;
    
    for chess_move in moves {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(chess_move);
        
        // IMPORTANTE: Nós somos o jogador maximizante no nível raiz,
        // mas após nosso movimento, é o oponente que joga (minimizante)
        let eval = alpha_beta(&new_board, depth, -100000, 100000, false, &mut repetitions);
        
        // Debugging: imprimir valores para verificar
        // eprintln!("Move: {}, Eval: {}", chess_move, eval);
        
        if eval > best_value {
            best_value = eval;
            best_move = Some(chess_move);
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

