use crate::board::Board;
use crate::engine::{pick_move, evaluate_breakdown, AiConfig};
use crate::piece::PieceType;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct SquarePiece {
    piece_type: String,
    color: String,
}

#[derive(Serialize)]
struct MoveJson {
    from: [usize; 2],
    to: [usize; 2],
    promotion: Option<String>,
}

#[derive(Serialize)]
struct BoardState {
    squares: Vec<Vec<Option<SquarePiece>>>,
    current_turn: String,
    game_over: bool,
    result: Option<String>,
    is_in_check: bool,
    legal_moves: Vec<MoveJson>,
    captured_white: Vec<String>,
    captured_black: Vec<String>,
    last_move: Option<[[usize; 2]; 2]>,
}

#[derive(Serialize)]
struct MoveResult {
    #[serde(flatten)]
    board_state: Option<BoardState>,
    error: Option<String>,
}

#[derive(Serialize)]
struct SquareMoveJson {
    to: [usize; 2],
    promotion: Option<String>,
}

#[derive(Serialize)]
struct EvalBreakdownJson {
    mate: f64,
    material: f64,
    centre: f64,
    passed_pawns: f64,
    draw_penalty: f64,
    total: f64,
}

fn piece_type_to_string(pt: PieceType) -> String {
    match pt {
        PieceType::King => "King".to_string(),
        PieceType::Queen => "Queen".to_string(),
        PieceType::Rook => "Rook".to_string(),
        PieceType::Bishop => "Bishop".to_string(),
        PieceType::Knight => "Knight".to_string(),
        PieceType::Pawn => "Pawn".to_string(),
    }
}

fn color_to_string(c: crate::piece::Color) -> String {
    match c {
        crate::piece::Color::White => "White".to_string(),
        crate::piece::Color::Black => "Black".to_string(),
    }
}

fn string_to_piece_type(s: &str) -> Option<PieceType> {
    match s {
        "Queen" => Some(PieceType::Queen),
        "Rook" => Some(PieceType::Rook),
        "Bishop" => Some(PieceType::Bishop),
        "Knight" => Some(PieceType::Knight),
        _ => None,
    }
}

fn build_board_state(board: &Board) -> BoardState {
    let squares: Vec<Vec<Option<SquarePiece>>> = (0..8)
        .map(|r| {
            (0..8)
                .map(|c| {
                    board.squares[r][c].map(|p| SquarePiece {
                        piece_type: piece_type_to_string(p.piece_type),
                        color: color_to_string(p.color),
                    })
                })
                .collect()
        })
        .collect();

    let legal_moves: Vec<MoveJson> = board
        .generate_legal_moves(board.current_turn)
        .iter()
        .map(|m| MoveJson {
            from: [m.from.0, m.from.1],
            to: [m.to.0, m.to.1],
            promotion: m.promotion.map(piece_type_to_string),
        })
        .collect();

    BoardState {
        squares,
        current_turn: color_to_string(board.current_turn),
        game_over: board.game_over,
        result: board.result.clone(),
        is_in_check: board.is_in_check(board.current_turn),
        legal_moves,
        captured_white: board.captured_white.iter().map(|pt| piece_type_to_string(*pt)).collect(),
        captured_black: board.captured_black.iter().map(|pt| piece_type_to_string(*pt)).collect(),
        last_move: board.last_move.map(|((fr, fc), (tr, tc))| [[fr, fc], [tr, tc]]),
    }
}

#[wasm_bindgen]
pub struct Game {
    board: Board,
    ai_config: AiConfig,
    last_evals: u64,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        Game {
            board: Board::new(),
            ai_config: AiConfig::new(),
            last_evals: 0,
        }
    }

    pub fn set_module(&mut self, name: &str, enabled: bool) {
        match name {
            "mate" => self.ai_config.mate_module = enabled,
            "material" => self.ai_config.material_module = enabled,
            "centre" => self.ai_config.centre_module = enabled,
            "passed_pawns" => self.ai_config.passed_pawn_module = enabled,
            "draw_penalty" => self.ai_config.draw_penalty_module = enabled,
            _ => {}
        }
    }

    pub fn set_depth(&mut self, depth: u32) {
        self.ai_config.depth = depth.clamp(1, 3);
    }

    pub fn get_board_state(&self) -> JsValue {
        let state = build_board_state(&self.board);
        serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
    }

    pub fn make_move(
        &mut self,
        from_row: usize,
        from_col: usize,
        to_row: usize,
        to_col: usize,
        promotion: Option<String>,
    ) -> JsValue {
        if self.board.game_over {
            let err = MoveResult {
                board_state: None,
                error: Some("Game is already over".to_string()),
            };
            return serde_wasm_bindgen::to_value(&err).unwrap_or(JsValue::NULL);
        }

        let promo_pt = promotion.as_deref().and_then(string_to_piece_type);

        let legal_moves = self.board.generate_legal_moves(self.board.current_turn);
        let matching_move = legal_moves.iter().find(|m| {
            m.from == (from_row, from_col) && m.to == (to_row, to_col) && m.promotion == promo_pt
        });

        match matching_move {
            Some(m) => {
                let m = m.clone();
                self.board.apply_move(&m);
                let state = build_board_state(&self.board);
                serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
            }
            None => {
                let err = MoveResult {
                    board_state: None,
                    error: Some("Illegal move".to_string()),
                };
                serde_wasm_bindgen::to_value(&err).unwrap_or(JsValue::NULL)
            }
        }
    }

    pub fn make_ai_move(&mut self) -> JsValue {
        if self.board.game_over {
            let state = build_board_state(&self.board);
            return serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL);
        }

        match pick_move(&self.board, &self.ai_config) {
            Some(result) => {
                self.last_evals = result.evals;
                self.board.apply_move(&result.mv);
                let state = build_board_state(&self.board);
                serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
            }
            None => {
                let state = build_board_state(&self.board);
                serde_wasm_bindgen::to_value(&state).unwrap_or(JsValue::NULL)
            }
        }
    }

    pub fn get_hint(&self, depth: u32) -> JsValue {
        let mut hint_config = self.ai_config.clone();
        hint_config.depth = depth.clamp(1, 3);
        match pick_move(&self.board, &hint_config) {
            Some(result) => {
                let hint = MoveJson {
                    from: [result.mv.from.0, result.mv.from.1],
                    to: [result.mv.to.0, result.mv.to.1],
                    promotion: result.mv.promotion.map(piece_type_to_string),
                };
                serde_wasm_bindgen::to_value(&hint).unwrap_or(JsValue::NULL)
            }
            None => JsValue::NULL,
        }
    }

    pub fn get_legal_moves_for_square(&self, row: usize, col: usize) -> JsValue {
        let legal_moves = self.board.generate_legal_moves(self.board.current_turn);
        let square_moves: Vec<SquareMoveJson> = legal_moves
            .iter()
            .filter(|m| m.from == (row, col))
            .map(|m| SquareMoveJson {
                to: [m.to.0, m.to.1],
                promotion: m.promotion.map(piece_type_to_string),
            })
            .collect();

        serde_wasm_bindgen::to_value(&square_moves).unwrap_or(JsValue::NULL)
    }

    pub fn get_last_evals(&self) -> u64 {
        self.last_evals
    }

    pub fn get_eval_breakdown(&self) -> JsValue {
        let breakdown = evaluate_breakdown(&self.board, self.board.current_turn, &self.ai_config);
        let json = EvalBreakdownJson {
            mate: breakdown.mate,
            material: breakdown.material,
            centre: breakdown.centre,
            passed_pawns: breakdown.passed_pawns,
            draw_penalty: breakdown.draw_penalty,
            total: breakdown.total,
        };
        serde_wasm_bindgen::to_value(&json).unwrap_or(JsValue::NULL)
    }
}
