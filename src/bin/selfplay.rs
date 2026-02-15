use chess::board::Board;
use chess::engine::{pick_move, AiConfig};
use chess::piece::Color;

fn main() {
    let mut config = AiConfig::new();
    config.depth = 2;
    config.auto_deepen = false;

    let mut board = Board::new();
    let mut move_count = 0;

    while !board.game_over && move_count < 60 {
        if let Some(result) = pick_move(&board, &config) {
            board.apply_move(&result.mv);
            move_count += 1;
        } else {
            break;
        }
    }

    let result = board.result.as_deref().unwrap_or("ongoing");
    eprintln!("Game over after {move_count} moves: {result}");
}
