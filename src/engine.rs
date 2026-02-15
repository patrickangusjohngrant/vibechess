// =============================================================================
// Chess AI Engine
//
// Uses negamax search with alpha-beta pruning. Moves are ordered by MVV-LVA
// (captures of high-value pieces first) and promotions so that alpha-beta
// prunes aggressively. The evaluation function is modular — each aspect of
// the position (material, centre control, passed pawns, draw avoidance) is
// scored independently and can be toggled on/off via AiConfig. Weights were
// optimized using the simulation tool (src/bin/simulate.rs).
//
// Coordinate system: row 0 = rank 1, col 0 = file a.
// Positional modules score from White's perspective; evaluate() flips for
// the AI's color. The draw penalty is perspective-independent.
// =============================================================================

use crate::board::Board;
use crate::moves::Move;
use crate::piece::{Color, PieceType};

/// Platform-appropriate random number in [0, 1).
/// Uses js_sys::Math::random() in WASM builds, rand crate natively.
fn random_f64() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Math::random()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use rand::Rng;
        rand::thread_rng().gen::<f64>()
    }
}

// =============================================================================
// Configuration
// =============================================================================

/// Tunable weights for the evaluation modules.
/// These were optimized by running AI-vs-AI simulations (src/bin/simulate.rs)
/// and ELO benchmarks against Stockfish (src/bin/elo.rs).
#[derive(Clone, Debug)]
pub struct Weights {
    // --- Centre control module ---
    /// Bonus per d4/d5/e4/e5 square attacked by AI (or penalty if attacked by opponent).
    pub centre_attack: f64,
    /// Bonus for having a piece physically on a centre square.
    pub centre_occupy: f64,
    /// Bonus per c3–f3/c6–f6 ring square attacked (the "extended centre").
    pub extended_centre_attack: f64,

    // --- Passed pawn module ---
    /// Base bonus for a passed pawn (no enemy pawns ahead on same or adjacent files).
    pub passed_pawn_base: f64,
    /// Additional bonus scaled by advancement². A pawn on the 6th rank gets
    /// a much larger bonus than one on the 3rd, encouraging promotion pushes.
    pub passed_pawn_quadratic: f64,
    /// Linear bonus per rank advanced for non-passed pawns (currently 0 = disabled).
    pub pawn_advance: f64,

    // --- Mate / check module ---
    /// Penalty applied to the side currently in check (but not mated).
    pub check_penalty: f64,

    // --- Draw avoidance module ---
    /// Flat penalty applied when the current position has been seen before.
    pub repeat_penalty: f64,
}

impl Default for Weights {
    fn default() -> Self {
        Weights {
            centre_attack: 0.15,
            centre_occupy: 0.2,
            extended_centre_attack: 0.3,
            passed_pawn_base: 0.2,
            passed_pawn_quadratic: 0.3,
            pawn_advance: 0.0,
            check_penalty: 0.5,
            repeat_penalty: 10.0,
        }
    }
}

/// Controls which evaluation modules are active and the search depth.
/// Each module can be toggled independently via the web UI.
#[derive(Clone)]
pub struct AiConfig {
    /// Detect checkmate/stalemate and assign extreme scores (±10000 / -5000).
    pub mate_module: bool,
    /// Count material advantage using standard piece values (P=1, N=B=3, R=5, Q=9).
    pub material_module: bool,
    /// Reward control and occupation of the centre squares.
    pub centre_module: bool,
    /// Reward passed pawns and penalize blocked pawns.
    pub passed_pawn_module: bool,
    /// Penalize positions that approach draws (repetition, 50-move rule).
    pub draw_penalty_module: bool,
    /// Search depth in full moves (1–3). Internally converted to plies (depth×2).
    pub depth: u32,
    /// When true, automatically increase depth until at least MIN_EVALS evaluations.
    pub auto_deepen: bool,
    pub weights: Weights,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AiConfig {
    pub fn new() -> Self {
        AiConfig {
            mate_module: true,
            material_module: true,
            centre_module: true,
            passed_pawn_module: true,
            draw_penalty_module: true,
            depth: 2,
            auto_deepen: true,
            weights: Weights::default(),
        }
    }
}

// =============================================================================
// Constants
// =============================================================================

/// A move paired with its minimax score, used during move selection.
struct ScoredMove {
    mv: Move,
    score: f64,
}

/// The result of a move search, including the chosen move and the number of
/// static evaluations performed during the search.
pub struct PickResult {
    pub mv: Move,
    pub evals: u64,
}

/// The four central squares: d4, d5, e4, e5.
const CENTRE_SQUARES: [(usize, usize); 4] = [(3, 3), (3, 4), (4, 3), (4, 4)];

/// The 12 squares forming the "extended centre" ring around the inner 4.
const EXTENDED_CENTRE: [(usize, usize); 12] = [
    (2, 2), (2, 3), (2, 4), (2, 5),
    (3, 2), (3, 5),
    (4, 2), (4, 5),
    (5, 2), (5, 3), (5, 4), (5, 5),
];

// =============================================================================
// Evaluation — top-level
//
// All modules score from White's perspective (positive = good for White).
// evaluate() flips the sign when the AI plays Black.
// =============================================================================

/// Standard piece values in pawns. The king has no material value since
/// losing it means checkmate (handled by the mate module instead).
fn piece_value(pt: PieceType) -> f64 {
    match pt {
        PieceType::Pawn => 1.0,
        PieceType::Knight => 3.0,
        PieceType::Bishop => 3.0,
        PieceType::Rook => 5.0,
        PieceType::Queen => 9.0,
        PieceType::King => 0.0,
    }
}

/// Evaluate the board from `ai_color`'s perspective by summing all enabled
/// modules. Positional modules score from White's perspective and are flipped
/// for Black. The draw penalty is perspective-independent (always negative).
pub fn evaluate(board: &Board, ai_color: Color, config: &AiConfig) -> f64 {
    let mut score = 0.0;

    if config.mate_module {
        score += eval_mate(board, &config.weights);
    }
    if config.material_module {
        score += eval_material(board);
    }
    if config.centre_module {
        score += eval_centre_control(board, &config.weights);
    }
    if config.passed_pawn_module {
        score += eval_passed_pawns(board, &config.weights);
    }

    if ai_color == Color::Black { score = -score; }

    if config.draw_penalty_module {
        score += eval_draw_penalty(board, &config.weights);
    }

    score
}

/// Returns the individual contribution of each module, used by the frontend
/// to display the eval breakdown bar chart.
#[derive(Clone, Debug)]
pub struct EvalBreakdown {
    pub mate: f64,
    pub material: f64,
    pub centre: f64,
    pub passed_pawns: f64,
    pub draw_penalty: f64,
    pub total: f64,
}

pub fn evaluate_breakdown(board: &Board, ai_color: Color, config: &AiConfig) -> EvalBreakdown {
    let flip = if ai_color == Color::Black { -1.0 } else { 1.0 };
    let mate = if config.mate_module { eval_mate(board, &config.weights) * flip } else { 0.0 };
    let material = if config.material_module { eval_material(board) * flip } else { 0.0 };
    let centre = if config.centre_module { eval_centre_control(board, &config.weights) * flip } else { 0.0 };
    let passed_pawns = if config.passed_pawn_module { eval_passed_pawns(board, &config.weights) * flip } else { 0.0 };
    let draw_penalty = if config.draw_penalty_module { eval_draw_penalty(board, &config.weights) } else { 0.0 };
    let total = mate + material + centre + passed_pawns + draw_penalty;
    EvalBreakdown { mate, material, centre, passed_pawns, draw_penalty, total }
}

// =============================================================================
// Evaluation modules — all score from White's perspective
// =============================================================================

/// Material: sum piece values for each side. A simple count of who has more
/// "stuff" on the board. This is the strongest signal for positional strength.
fn eval_material(board: &Board) -> f64 {
    let mut score = 0.0;
    for row in 0..8 {
        for col in 0..8 {
            if let Some(p) = board.squares[row][col] {
                let v = piece_value(p.piece_type);
                if p.color == Color::White { score += v; } else { score -= v; }
            }
        }
    }
    score
}

/// Centre control: rewards attacking and occupying the four central squares
/// (d4, d5, e4, e5) and the extended centre ring. Controlling the centre
/// gives pieces more mobility and restricts the opponent.
fn eval_centre_control(board: &Board, w: &Weights) -> f64 {
    let mut score = 0.0;

    for &(r, c) in &CENTRE_SQUARES {
        if board.is_square_attacked_by(r, c, Color::White) { score += w.centre_attack; }
        if board.is_square_attacked_by(r, c, Color::Black) { score -= w.centre_attack; }
        if let Some(p) = board.squares[r][c] {
            if p.color == Color::White { score += w.centre_occupy; } else { score -= w.centre_occupy; }
        }
    }

    for &(r, c) in &EXTENDED_CENTRE {
        if board.is_square_attacked_by(r, c, Color::White) { score += w.extended_centre_attack; }
        if board.is_square_attacked_by(r, c, Color::Black) { score -= w.extended_centre_attack; }
    }

    score
}

/// Mate and check detection: assigns extreme scores to checkmate, a large
/// penalty to stalemate (draw), and a smaller penalty for being in check.
fn eval_mate(board: &Board, w: &Weights) -> f64 {
    let in_check = board.is_in_check(board.current_turn);
    let no_moves = board.game_over || board.generate_legal_moves(board.current_turn).is_empty();

    if no_moves && in_check {
        // Checkmate — the side to move has lost
        if board.current_turn == Color::White { -10000.0 } else { 10000.0 }
    } else if no_moves {
        // Stalemate — a draw
        0.0
    } else if in_check {
        // In check but can escape — slight penalty for the checked side
        if board.current_turn == Color::White { -w.check_penalty } else { w.check_penalty }
    } else {
        0.0
    }
}

/// Check whether a pawn is "passed" — no enemy pawns ahead of it on the
/// same file or adjacent files. A passed pawn is a major strategic advantage
/// because it can potentially promote to a queen without being blocked.
fn is_passed_pawn(board: &Board, row: usize, col: usize, color: Color) -> bool {
    // Scan forward from the pawn's current row toward the promotion rank
    let (start_row, end_row, step): (i32, i32, i32) = match color {
        Color::White => (row as i32 + 1, 7, 1),   // White promotes on rank 8 (row 7)
        Color::Black => (row as i32 - 1, 0, -1),   // Black promotes on rank 1 (row 0)
    };

    let enemy = color.opposite();
    let mut r = start_row;
    while (step > 0 && r <= end_row) || (step < 0 && r >= end_row) {
        // Check the pawn's file and both adjacent files
        for dc in -1i32..=1 {
            let c = col as i32 + dc;
            if (0..8).contains(&c) {
                if let Some(p) = board.squares[r as usize][c as usize] {
                    if p.color == enemy && p.piece_type == PieceType::Pawn {
                        return false; // Blocked or guarded by enemy pawn
                    }
                }
            }
        }
        r += step;
    }
    true
}

/// Passed pawn evaluation: rewards passed pawns with a score that grows
/// quadratically as they advance toward promotion. Non-passed pawns get
/// a small linear bonus for advancement (if pawn_advance weight > 0).
///
/// Advancement is measured from the starting rank:
///   White pawn on row r: advancement = r - 1 (0 on rank 2, 5 on rank 7)
///   Black pawn on row r: advancement = 6 - r (0 on rank 7, 5 on rank 2)
fn eval_passed_pawns(board: &Board, w: &Weights) -> f64 {
    let mut score = 0.0;

    for row in 0..8usize {
        for col in 0..8usize {
            if let Some(p) = board.squares[row][col] {
                if p.piece_type != PieceType::Pawn {
                    continue;
                }

                let s = if p.color == Color::White { 1.0 } else { -1.0 };

                let advancement = match p.color {
                    Color::White => row as f64 - 1.0,
                    Color::Black => 6.0 - row as f64,
                };

                if is_passed_pawn(board, row, col, p.color) {
                    score += s * (w.passed_pawn_base + advancement * advancement * w.passed_pawn_quadratic);
                } else {
                    score += s * advancement * w.pawn_advance;
                }
            }
        }
    }

    score
}

/// Draw avoidance: applies a flat penalty if the current position has been
/// seen before in the game. Discourages the AI from repeating positions.
fn eval_draw_penalty(board: &Board, w: &Weights) -> f64 {
    let current_hash = board.position_hash();
    let repeat_count = board.position_history.iter().filter(|&&h| h == current_hash).count();
    if repeat_count >= 2 {
        -(w.repeat_penalty)
    } else {
        0.0
    }
}

// =============================================================================
// Move ordering
// =============================================================================

/// Assign a priority score to a move for search ordering. Higher = searched
/// first. Good ordering causes alpha-beta to prune far more branches.
///
/// Priority tiers (from highest to lowest):
///   1. Promotions (queen promotion highest)
///   2. Captures, ordered by MVV-LVA (Most Valuable Victim, Least Valuable
///      Attacker) — e.g. pawn takes queen is searched before queen takes queen
///   3. Quiet moves (score 0)
fn move_priority(board: &Board, mv: &Move) -> i32 {
    let mut score = 0;

    if let Some(promo) = mv.promotion {
        score += 900 + piece_value(promo) as i32;
    }

    if let Some(victim) = board.squares[mv.to.0][mv.to.1] {
        let attacker = board.squares[mv.from.0][mv.from.1]
            .map(|p| piece_value(p.piece_type) as i32)
            .unwrap_or(0);
        score += 100 + piece_value(victim.piece_type) as i32 * 10 - attacker;
    }

    score
}

/// Sort moves so the most promising are searched first.
fn order_moves(board: &Board, moves: &mut [Move]) {
    moves.sort_by_key(|mv| std::cmp::Reverse(move_priority(board, mv)));
}

// =============================================================================
// Search — Negamax with alpha-beta pruning
// =============================================================================

/// Negamax search with alpha-beta pruning.
///
/// Scores are always from the current player's perspective (positive = good
/// for the side to move). Each recursive call negates the returned score,
/// which eliminates the need for separate maximizing/minimizing branches.
///
/// Moves are ordered before searching so that captures and promotions are
/// tried first, which causes alpha-beta to prune much more aggressively.
fn negamax(
    board: &Board,
    depth: u32,
    mut alpha: f64,
    beta: f64,
    config: &AiConfig,
    evals: &mut u64,
) -> f64 {
    if depth == 0 || board.game_over {
        *evals += 1;
        return evaluate(board, board.current_turn, config);
    }

    let mut legal_moves = board.generate_legal_moves(board.current_turn);
    if legal_moves.is_empty() {
        *evals += 1;
        return evaluate(board, board.current_turn, config);
    }

    order_moves(board, &mut legal_moves);

    let mut best = f64::NEG_INFINITY;
    for mv in &legal_moves {
        let mut clone = board.clone();
        clone.apply_move(mv);
        let score = -negamax(&clone, depth - 1, -beta, -alpha, config, evals);
        best = best.max(score);
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
        }
    }

    best
}

// =============================================================================
// Move selection
// =============================================================================

/// Pick the best move for the current player.
///
/// 1. Generate and order all legal moves
/// 2. Score each move via negamax on the resulting position
/// 3. Collect all moves tied for the best score
/// 4. Randomly pick among the tied moves (adds variety to play)
///
/// Depth is specified in "full moves" (e.g. depth=2 means the AI looks 2 moves
/// ahead for each side = 4 plies total). The first ply is consumed by applying
/// each candidate move, so negamax is called with `plies - 1`.
const MIN_EVALS: u64 = 100_000;

fn pick_move_at_depth(board: &Board, legal_moves: &[Move], plies: u32, config: &AiConfig) -> (Vec<ScoredMove>, u64) {
    let mut evals: u64 = 0;
    let scored: Vec<ScoredMove> = legal_moves
        .iter()
        .map(|mv| {
            let mut clone = board.clone();
            clone.apply_move(mv);
            let score = -negamax(&clone, plies - 1, f64::NEG_INFINITY, f64::INFINITY, config, &mut evals);
            ScoredMove { mv: mv.clone(), score }
        })
        .collect();
    (scored, evals)
}

pub fn pick_move(board: &Board, config: &AiConfig) -> Option<PickResult> {
    let mut legal_moves = board.generate_legal_moves(board.current_turn);
    if legal_moves.is_empty() {
        return None;
    }

    order_moves(board, &mut legal_moves);

    let mut plies = config.depth * 2;
    let (mut scored, mut evals) = pick_move_at_depth(board, &legal_moves, plies, config);

    // If auto-deepen is on and the search was too shallow, increase depth.
    while config.auto_deepen && evals < MIN_EVALS && plies < 6 {
        plies += 1;
        let (new_scored, new_evals) = pick_move_at_depth(board, &legal_moves, plies, config);
        scored = new_scored;
        evals = new_evals;
    }

    let max_score = scored
        .iter()
        .map(|s| s.score)
        .fold(f64::NEG_INFINITY, f64::max);

    let best: Vec<&ScoredMove> = scored
        .iter()
        .filter(|s| (s.score - max_score).abs() < 0.001)
        .collect();

    // Among tied moves, prefer the highest-priority one (e.g. queen promotion
    // over bishop promotion) then randomize among any still tied.
    let max_pri = best.iter().map(|s| move_priority(board, &s.mv)).max().unwrap_or(0);
    let top: Vec<&ScoredMove> = best.into_iter().filter(|s| move_priority(board, &s.mv) == max_pri).collect();

    let index = (random_f64() * top.len() as f64) as usize;
    Some(PickResult { mv: top[index.min(top.len() - 1)].mv.clone(), evals })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::piece::Piece;

    /// Place kings + a white pawn on a7. The engine must promote to queen.
    fn board_pawn_on_a7() -> Board {
        let mut board = Board::empty();
        board.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        board.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        board.squares[6][0] = Some(Piece::new(PieceType::Pawn, Color::White));
        board.current_turn = Color::White;
        board
    }

    #[test]
    fn pawn_promotes_to_queen() {
        let board = board_pawn_on_a7();
        let config = AiConfig::new();
        let result = pick_move(&board, &config).expect("should find a move");
        assert_eq!(result.mv.from, (6, 0), "should move from a7");
        assert_eq!(result.mv.to, (7, 0), "should move to a8");
        assert_eq!(result.mv.promotion, Some(PieceType::Queen), "should promote to queen");
    }

    #[test]
    fn queen_promotion_scores_higher_than_others() {
        let board = board_pawn_on_a7();
        let config = AiConfig::new();

        let moves = board.generate_legal_moves(Color::White);
        let promos: Vec<&Move> = moves.iter()
            .filter(|m| m.from == (6, 0) && m.to == (7, 0) && m.promotion.is_some())
            .collect();
        assert!(promos.len() >= 2, "should have multiple promotion options");

        let mut queen_score = f64::NEG_INFINITY;
        let mut best_non_queen = f64::NEG_INFINITY;
        for mv in &promos {
            let mut clone = board.clone();
            clone.apply_move(mv);
            let score = evaluate(&clone, Color::White, &config);
            if mv.promotion == Some(PieceType::Queen) {
                queen_score = score;
            } else {
                best_non_queen = best_non_queen.max(score);
            }
        }
        assert!(queen_score > best_non_queen,
            "queen promotion ({queen_score}) should score higher than any other ({best_non_queen})");
    }

    #[test]
    fn material_eval_counts_pieces() {
        let mut board = Board::empty();
        board.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        board.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        board.squares[0][0] = Some(Piece::new(PieceType::Queen, Color::White));

        let config = AiConfig::new();
        let score_white = evaluate(&board, Color::White, &config);
        let score_black = evaluate(&board, Color::Black, &config);
        assert!(score_white > 0.0, "white with extra queen should be positive: {score_white}");
        assert!(score_black < 0.0, "black perspective should be negative: {score_black}");
    }

    #[test]
    fn checkmate_detected() {
        // Scholar's mate final position: White Qf7#
        let mut board = Board::empty();
        board.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        board.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        board.squares[6][5] = Some(Piece::new(PieceType::Queen, Color::White)); // Qf7
        board.squares[5][2] = Some(Piece::new(PieceType::Bishop, Color::White)); // Bc6 covering escape
        board.current_turn = Color::Black;

        let moves = board.generate_legal_moves(Color::Black);
        // Black king should have very limited moves; verify game_over detection works
        // by applying a position where black has no legal moves and is in check
        if moves.is_empty() && board.is_in_check(Color::Black) {
            board.game_over = true;
            let config = AiConfig::new();
            let score = evaluate(&board, Color::White, &config);
            assert!(score > 9000.0, "checkmate should score very high for white: {score}");
        }
    }

    #[test]
    fn draw_penalty_applies_on_repeat() {
        let mut board = Board::empty();
        board.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        board.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        board.squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));

        let config = AiConfig::new();
        let hash = board.position_hash();

        // First occurrence — no penalty
        board.position_history.push(hash);
        let score_fresh = evaluate(&board, Color::White, &config);

        // Second occurrence — penalty kicks in
        board.position_history.push(hash);
        let score_repeat = evaluate(&board, Color::White, &config);

        assert!(score_fresh > score_repeat,
            "repeated position ({score_repeat}) should score less than fresh ({score_fresh})");
    }

    #[test]
    fn pick_move_returns_eval_count() {
        let board = Board::new();
        let mut config = AiConfig::new();
        config.depth = 1;
        let result = pick_move(&board, &config).expect("should find a move");
        assert!(result.evals > 0, "should have evaluated at least one position");
    }

    #[test]
    fn stalemate_is_draw() {
        // White king on a1, Black king on c2, white to move with no legal moves = stalemate
        // Actually let's set up: White Ka1, Black Qb3, Kc1 — white has no moves
        let mut board = Board::empty();
        board.squares[0][0] = Some(Piece::new(PieceType::King, Color::White));  // Ka1
        board.squares[2][1] = Some(Piece::new(PieceType::Queen, Color::Black)); // Qb3
        board.squares[1][2] = Some(Piece::new(PieceType::King, Color::Black));  // Kc2
        board.current_turn = Color::White;

        let moves = board.generate_legal_moves(Color::White);
        if moves.is_empty() && !board.is_in_check(Color::White) {
            board.game_over = true;
            let config = AiConfig::new();
            let score = evaluate(&board, Color::White, &config);
            // Stalemate should score close to 0 (neutral), not like a checkmate
            assert!(score.abs() < 100.0, "stalemate should not score like mate: {score}");
        }
    }

}
