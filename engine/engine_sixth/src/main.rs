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

    // Material e controle do centro
    for square in chess::ALL_SQUARES.iter() {
        if let Some(piece) = board.piece_on(*square) {
            let value = piece_value(piece);
            if board.color_on(*square) == Some(side_to_move) {
                score += value * 100; // Valores mais altos para material (100 = 1 peão)
                
                // Bônus para controle do centro
                if CENTER_SQUARES.contains(square) {
                    score += 20; // 0.2 peões
                }
            } else {
                score -= value * 100;
                if CENTER_SQUARES.contains(square) {
                    score -= 20;
                }
            }
        }
    }

    // Status do jogo
    match board.status() {
        BoardStatus::Checkmate => {
            return -30000; // Valor extremamente negativo para xeque-mate
        }
        BoardStatus::Stalemate => {
            return 0; // Empate
        }
        _ => {
            // Mais fatores de avaliação
            
            // 1. Estar em xeque é ruim
            if board.checkers().popcnt() > 0 {
                score -= 50;
            }
            
            // 2. Mobilidade (quantidade de movimentos legais)
            let mobility = MoveGen::new_legal(board).count() as i32;
            score += mobility * 5; // Valorizar mais a mobilidade
            
            // 3. Valor das peças ameaçadas (alvos não defendidos)
            for square in chess::ALL_SQUARES.iter() {
                if let Some(piece) = board.piece_on(*square) {
                    if board.color_on(*square) != Some(side_to_move) {
                        // Check if any of our pieces can attack this opponent's piece
                        for our_square in chess::ALL_SQUARES.iter() {
                            if board.color_on(*our_square) == Some(side_to_move) {
                                // Check if there's a legal move from our_square to square (capturing)
                                let potential_move = ChessMove::new(*our_square, *square, None);
                                if board.legal(potential_move) {
                                    score += piece_value(piece) * 10; // Bonus for threatening pieces
                                    break;
                                }
                            }
                        }
                    } else {
                        // Check if any opponent pieces can attack our piece
                        for their_square in chess::ALL_SQUARES.iter() {
                            if board.color_on(*their_square).is_some() && 
                               board.color_on(*their_square) != Some(side_to_move) {
                                // Check if there's a legal move from their_square to square (capturing)
                                let potential_move = ChessMove::new(*their_square, *square, None);
                                if board.legal(potential_move) {
                                    score -= piece_value(piece) * 10; // Penalty for pieces under threat
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            
            // 4. Desenvolvimento (peças fora da posição inicial)
            if side_to_move == chess::Color::White {
                if let Some(Piece::King) = board.piece_on(Square::E1) {
                    score += 10; // Bônus leve por manter o rei seguro no início
                }
            } else {
                if let Some(Piece::King) = board.piece_on(Square::E8) {
                    score += 10;
                }
            }
            
            // 5. Posição avançada para peões (promover peões)
            for rank in 0..8 {
                for file in 0..8 {
                    let square = Square::make_square(
                        chess::Rank::from_index(rank),
                        chess::File::from_index(file)
                    );
                    
                    if let Some(Piece::Pawn) = board.piece_on(square) {
                        if board.color_on(square) == Some(side_to_move) {
                            if side_to_move == chess::Color::White {
                                score += (rank as i32) * 5; // Mais avançado, maior o bônus
                            } else {
                                score += (7 - rank as i32) * 5;
                            }
                        }
                    }
                }
            }
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
    initial_depth: i32,
) -> i32 {
    let hash = board.get_hash();
    
    // Verificar fim de jogo ou profundidade máxima
    if depth == 0 || board.status() != BoardStatus::Ongoing {
        return if maximizing { 
            evaluate_board(board)
        } else { 
            -evaluate_board(board)
        };
    }
    
    // Verificação de repetição
    let count = repetitions.get(&hash).cloned().unwrap_or(0);
    
    // Tratamento especial para empates por repetição
    if count >= 2 { // Anteriormente era 3, agora 2 para ser mais conservador
        // Se estiver perto da raiz da árvore de busca e com vantagem material, evite repetições
        if depth >= initial_depth - 2 {
            let eval = evaluate_board(board);
            // Se temos vantagem material, penalizar repetição para evitar empate
            if (maximizing && eval > 100) || (!maximizing && eval < -100) {
                return if maximizing { -5000 } else { 5000 }; // Penalidade pela repetição
            }
        }
        return 0; // Empate por repetição normal
    }
    
    repetitions.insert(hash, count + 1);

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    
    // Ordenação mais sofisticada de movimentos
    moves.sort_by_key(|m| {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(*m);
        
        let mut score = 0;
        
        // 1. Xeque-mate (prioridade máxima)
        if new_board.status() == BoardStatus::Checkmate {
            return 50000;
        }
        
        // 2. Capturas (ordenadas pelo valor da peça capturada - valor da peça que captura)
        if let Some(captured) = board.piece_on(m.get_dest()) {
            let moving_piece = board.piece_on(m.get_source()).unwrap();
            score += piece_value(captured) * 100 - piece_value(moving_piece) * 10;
        }
        
        // 3. Promoções
        if let Some(promotion) = m.get_promotion() {
            score += piece_value(promotion) * 90;
        }
        
        // 4. Xeques
        if new_board.checkers().popcnt() > 0 {
            score += 300;
        }
        
        // 5. Movimentos para o centro
        if CENTER_SQUARES.contains(&m.get_dest()) {
            score += 50;
        }
        
        score
    });
    
    moves.reverse(); // Priorizar maiores valores

    let mut best_eval = if maximizing { -100000 } else { 100000 };

    for chess_move in moves {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(chess_move);

        let eval = alpha_beta(&new_board, depth - 1, alpha, beta, !maximizing, repetitions, initial_depth);

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

    // Restaurar contagem de repetições
    repetitions.insert(hash, count);
    
    best_eval
}


fn best_move(board: &Board) -> Option<ChessMove> {
    let mut best_move = None;
    let mut best_value = -100000;
    let mut repetitions = HashMap::new();
    
    // Profundidade adaptativa com base no estágio do jogo
    let piece_count = chess::ALL_SQUARES.iter()
        .filter(|sq| board.piece_on(**sq).is_some())
        .count();
    
    // Ajustar profundidade com base no número de peças
    // Menos peças = jogo mais avançado = pode buscar mais profundo
    let depth = if piece_count < 10 {
        4 // Fim de jogo
    } else if piece_count < 20 {
        3 // Meio de jogo
    } else {
        3 // Abertura
    };
    
    let moves: Vec<ChessMove> = MoveGen::new_legal(board).collect();
    
    // Verificação especial para xeque-mate em um movimento
    for chess_move in &moves {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(*chess_move);
        
        if new_board.status() == BoardStatus::Checkmate {
            return Some(*chess_move); // Retornar imediatamente se encontrar xeque-mate
        }
    }
    
    // Se não encontramos mate em 1, proceder com a busca normal
    for chess_move in moves {
        let mut new_board = board.clone();
        new_board = new_board.make_move_new(chess_move);
        
        let eval = alpha_beta(&new_board, depth - 1, -100000, 100000, false, &mut repetitions, depth);
        
        // Debug: descomentar para ver avaliações
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
        } else if input == "debug" {
            // Comando adicional para debug
            println!("Avaliação atual: {}", evaluate_board(&board));
            println!("Status: {:?}", board.status());
            println!("Lado a mover: {:?}", board.side_to_move());
            println!("Movimentos legais: {}", MoveGen::new_legal(&board).count());
        }
    }
}

