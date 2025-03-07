import pygame
import chess
import subprocess
import time
from configs import SQ_SIZE,WHITE, BLACK, RED, screen, WIDTH, HEIGHT, board, running, selected_square
import os

# Inicializa o Pygame
pygame.init()
pygame.mixer.init()
pygame.mixer.music.load(os.path.join("../songs", "jazzy.mp3"))
pygame.mixer.music.play(-1)
pygame.mixer_music.set_volume(0.1)

# Carregar imagens das peças
def load_images():
    pieces = ['r', 'n', 'b', 'q', 'k', 'p', 'R', 'N', 'B', 'Q', 'K', 'P']
    images = {}
    for piece in pieces:
        images[piece] = pygame.transform.scale(
            pygame.image.load(f"../pieces/{piece}.svg"), (SQ_SIZE, SQ_SIZE)
        )
    return images

images = load_images()

# Função para desenhar o tabuleiro
def draw_board():
    for row in range(8):
        for col in range(8):
            color = WHITE if (row + col) % 2 == 0 else BLACK
            pygame.draw.rect(screen, color, (col * SQ_SIZE, row * SQ_SIZE, SQ_SIZE, SQ_SIZE))

# Função para desenhar as peças
def draw_pieces(board):
    for row in range(8):
        for col in range(8):
            piece = board.piece_at(chess.square(col, 7 - row))  # Converte coordenadas para FEN
            if piece:
                screen.blit(images[piece.symbol()], (col * SQ_SIZE, row * SQ_SIZE))

# Converte coordenadas do clique para a posição no tabuleiro
def get_square_from_mouse(pos):
    x, y = pos
    col = x // SQ_SIZE
    row = 7 - (y // SQ_SIZE)  # Inverte eixo Y para FEN
    return chess.square(col, row)

# Exibir tela de Game Over
def show_game_over(text):
    font = pygame.font.Font(None, 60)
    text_surface = font.render(text, True, RED)
    text_rect = text_surface.get_rect(center=(WIDTH // 2, HEIGHT // 2))
    screen.blit(text_surface, text_rect)
    pygame.display.flip()
    time.sleep(6)

# Inicializa a engine personalizada
engine_path = "../engine/engine_sixth/target/release/engine_sixth" 
engine = subprocess.Popen(
    [engine_path],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    text=True
)

# Função para obter movimento da IA
def get_ai_move(board):
    fen = board.fen()
    engine.stdin.write(f"position {fen}\n")  
    engine.stdin.write("go\n")  
    engine.stdin.flush()

    move = engine.stdout.readline().strip()  
    return move if move else None


while running:
    draw_board()
    draw_pieces(board)
    pygame.display.flip()

    
    if board.is_game_over():
        if board.is_checkmate():
            if board.turn == chess.BLACK:
                result_text = "Xeque-mate!"  
            else :
                result_text = "Vitória da IA!"
        elif board.is_stalemate():
            result_text = "Empate por afogamento!"
        elif board.is_insufficient_material():
            result_text = "Empate por material insuficiente!"
        elif board.is_seventyfive_moves():
            result_text = "Empate por 75 movimentos!"
        elif board.is_fivefold_repetition():
            result_text = "Empate por repetição!"
        else:
            result_text = "Empate!"
        
        show_game_over(result_text)
        running = False
        break

    for event in pygame.event.get():
        if event.type == pygame.QUIT:
            running = False

        elif event.type == pygame.MOUSEBUTTONDOWN and board.turn == chess.WHITE:
            square = get_square_from_mouse(pygame.mouse.get_pos())

            if board.piece_at(square) and board.piece_at(square).color == chess.WHITE:
                selected_square = square
            elif selected_square is not None:
                if board.piece_at(selected_square).piece_type == chess.PAWN and chess.square_rank(square) in [0, 7]:
                    move = chess.Move(selected_square, square, promotion=chess.QUEEN)
                else:
                    move = chess.Move(selected_square, square)
                
                if move in board.legal_moves:
                    board.push(move)
                    selected_square = None
                    
                    # Redesenha o tabuleiro para mostrar o movimento do jogador
                    draw_board()
                    draw_pieces(board)
                    pygame.display.flip()
                    
                    # Adiciona um delay antes do movimento da IA
                    time.sleep(0.4) 
                    
                    if not board.is_game_over():                    
                        ai_move = get_ai_move(board)
                        if ai_move:
                            board.push(chess.Move.from_uci(ai_move))
                        else:
                            print("Erro ao obter movimento da IA!")


engine.stdin.write("quit\n")
engine.stdin.flush()
engine.terminate()

pygame.quit()
