import pygame
import chess

# Configurações do tabuleiro
WIDTH, HEIGHT = 600, 600
SQ_SIZE = WIDTH // 8
screen = pygame.display.set_mode((WIDTH, HEIGHT))
pygame.display.set_caption("Chazz")

# Cores
WHITE = (238, 238, 210)
BLACK = (27, 30, 35)
RED = (200, 50, 50)

# Configuração do jogo
board = chess.Board()
selected_square = None
running = True

