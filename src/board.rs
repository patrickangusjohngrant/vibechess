use serde::{Deserialize, Serialize};

use crate::moves::Move;
use crate::piece::{Color, Piece, PieceType};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
    pub current_turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_target: Option<(usize, usize)>,
    pub halfmove_clock: u32,
    pub fullmove_number: u32,
    pub game_over: bool,
    pub result: Option<String>,
    pub captured_white: Vec<PieceType>,
    pub captured_black: Vec<PieceType>,
    pub last_move: Option<((usize, usize), (usize, usize))>,
    pub position_history: Vec<u64>,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Create an empty board with no pieces. Useful for setting up test positions.
    pub fn empty() -> Self {
        let board = Board {
            squares: [[None; 8]; 8],
            current_turn: Color::White,
            castling_rights: CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: false,
                black_queenside: false,
            },
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            game_over: false,
            result: None,
            captured_white: Vec::new(),
            captured_black: Vec::new(),
            last_move: None,
            position_history: Vec::new(),
        };
        board
    }

    pub fn new() -> Self {
        let mut squares = [[None; 8]; 8];

        // White pieces (rows 0-1)
        squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));
        squares[0][1] = Some(Piece::new(PieceType::Knight, Color::White));
        squares[0][2] = Some(Piece::new(PieceType::Bishop, Color::White));
        squares[0][3] = Some(Piece::new(PieceType::Queen, Color::White));
        squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        squares[0][5] = Some(Piece::new(PieceType::Bishop, Color::White));
        squares[0][6] = Some(Piece::new(PieceType::Knight, Color::White));
        squares[0][7] = Some(Piece::new(PieceType::Rook, Color::White));
        for sq in &mut squares[1] {
            *sq = Some(Piece::new(PieceType::Pawn, Color::White));
        }

        // Black pieces (rows 6-7)
        for sq in &mut squares[6] {
            *sq = Some(Piece::new(PieceType::Pawn, Color::Black));
        }
        squares[7][0] = Some(Piece::new(PieceType::Rook, Color::Black));
        squares[7][1] = Some(Piece::new(PieceType::Knight, Color::Black));
        squares[7][2] = Some(Piece::new(PieceType::Bishop, Color::Black));
        squares[7][3] = Some(Piece::new(PieceType::Queen, Color::Black));
        squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        squares[7][5] = Some(Piece::new(PieceType::Bishop, Color::Black));
        squares[7][6] = Some(Piece::new(PieceType::Knight, Color::Black));
        squares[7][7] = Some(Piece::new(PieceType::Rook, Color::Black));

        let mut board = Board {
            squares,
            current_turn: Color::White,
            castling_rights: CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            },
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            game_over: false,
            result: None,
            captured_white: Vec::new(),
            captured_black: Vec::new(),
            last_move: None,
            position_history: Vec::new(),
        };
        board.position_history.push(board.position_hash());
        board
    }

    fn in_bounds(row: i32, col: i32) -> bool {
        (0..8).contains(&row) && (0..8).contains(&col)
    }

    pub fn find_king(&self, color: Color) -> Option<(usize, usize)> {
        for r in 0..8 {
            for c in 0..8 {
                if let Some(p) = self.squares[r][c] {
                    if p.piece_type == PieceType::King && p.color == color {
                        return Some((r, c));
                    }
                }
            }
        }
        None
    }

    pub fn is_square_attacked_by(&self, row: usize, col: usize, attacker: Color) -> bool {
        // Check knight attacks
        let knight_offsets: [(i32, i32); 8] = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1),
        ];
        for (dr, dc) in &knight_offsets {
            let r = row as i32 + dr;
            let c = col as i32 + dc;
            if Self::in_bounds(r, c) {
                if let Some(p) = self.squares[r as usize][c as usize] {
                    if p.color == attacker && p.piece_type == PieceType::Knight {
                        return true;
                    }
                }
            }
        }

        // Check king attacks
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = row as i32 + dr;
                let c = col as i32 + dc;
                if Self::in_bounds(r, c) {
                    if let Some(p) = self.squares[r as usize][c as usize] {
                        if p.color == attacker && p.piece_type == PieceType::King {
                            return true;
                        }
                    }
                }
            }
        }

        // Check pawn attacks
        let pawn_dir: i32 = if attacker == Color::White { 1 } else { -1 };
        // A pawn on (row - pawn_dir, col ± 1) attacks (row, col)
        let pawn_row = row as i32 - pawn_dir;
        for dc in &[-1i32, 1] {
            let pc = col as i32 + dc;
            if Self::in_bounds(pawn_row, pc) {
                if let Some(p) = self.squares[pawn_row as usize][pc as usize] {
                    if p.color == attacker && p.piece_type == PieceType::Pawn {
                        return true;
                    }
                }
            }
        }

        // Check sliding pieces (rook/queen on straights, bishop/queen on diagonals)
        let straight_dirs: [(i32, i32); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        for (dr, dc) in &straight_dirs {
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;
            while Self::in_bounds(r, c) {
                if let Some(p) = self.squares[r as usize][c as usize] {
                    if p.color == attacker
                        && (p.piece_type == PieceType::Rook || p.piece_type == PieceType::Queen)
                    {
                        return true;
                    }
                    break;
                }
                r += dr;
                c += dc;
            }
        }

        let diag_dirs: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        for (dr, dc) in &diag_dirs {
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;
            while Self::in_bounds(r, c) {
                if let Some(p) = self.squares[r as usize][c as usize] {
                    if p.color == attacker
                        && (p.piece_type == PieceType::Bishop || p.piece_type == PieceType::Queen)
                    {
                        return true;
                    }
                    break;
                }
                r += dr;
                c += dc;
            }
        }

        false
    }

    /// Check if a queen of the given color attacks the target square.
    pub fn is_square_attacked_by_queen(&self, row: usize, col: usize, attacker: Color) -> bool {
        let dirs: [(i32, i32); 8] = [
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        for &(dr, dc) in &dirs {
            let (mut r, mut c) = (row as i32 + dr, col as i32 + dc);
            while Self::in_bounds(r, c) {
                if let Some(p) = self.squares[r as usize][c as usize] {
                    if p.color == attacker && p.piece_type == PieceType::Queen {
                        return true;
                    }
                    break;
                }
                r += dr;
                c += dc;
            }
        }
        false
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        if let Some((kr, kc)) = self.find_king(color) {
            self.is_square_attacked_by(kr, kc, color.opposite())
        } else {
            false
        }
    }

    pub fn position_hash(&self) -> u64 {
        let mut hash: u64 = 0;
        for r in 0..8 {
            for c in 0..8 {
                if let Some(p) = self.squares[r][c] {
                    let piece_val = match p.piece_type {
                        PieceType::Pawn => 1u64,
                        PieceType::Knight => 2,
                        PieceType::Bishop => 3,
                        PieceType::Rook => 4,
                        PieceType::Queen => 5,
                        PieceType::King => 6,
                    };
                    let color_val = if p.color == Color::White { 0u64 } else { 7u64 };
                    let sq_val = piece_val + color_val;
                    hash ^= sq_val.wrapping_mul(0x9e3779b97f4a7c15u64.wrapping_add(((r * 8 + c) as u64).wrapping_mul(0x517cc1b727220a95)));
                }
            }
        }
        if self.current_turn == Color::Black { hash ^= 0xdeadbeefcafe1234; }
        if self.castling_rights.white_kingside { hash ^= 0x1; }
        if self.castling_rights.white_queenside { hash ^= 0x2; }
        if self.castling_rights.black_kingside { hash ^= 0x4; }
        if self.castling_rights.black_queenside { hash ^= 0x8; }
        if let Some((r, c)) = self.en_passant_target {
            hash ^= (r as u64 * 8 + c as u64).wrapping_mul(0xabcdef0123456789);
        }
        hash
    }

    pub fn is_threefold_repetition(&self) -> bool {
        if self.position_history.len() < 5 {
            return false;
        }
        let current = self.position_hash();
        let count = self.position_history.iter().filter(|&&h| h == current).count();
        count >= 3 // current position is already in history, so 3 entries = 3 occurrences
    }

    pub fn has_insufficient_material(&self) -> bool {
        let mut white_pieces = Vec::new();
        let mut black_pieces = Vec::new();
        for r in 0..8 {
            for c in 0..8 {
                if let Some(p) = self.squares[r][c] {
                    match p.color {
                        Color::White => white_pieces.push(p.piece_type),
                        Color::Black => black_pieces.push(p.piece_type),
                    }
                }
            }
        }
        // King vs King
        if white_pieces.len() == 1 && black_pieces.len() == 1 {
            return true;
        }
        // King+minor vs King
        if white_pieces.len() == 1 && black_pieces.len() == 2
            && black_pieces.iter().any(|&pt| pt == PieceType::Bishop || pt == PieceType::Knight)
        {
            return true;
        }
        if black_pieces.len() == 1 && white_pieces.len() == 2
            && white_pieces.iter().any(|&pt| pt == PieceType::Bishop || pt == PieceType::Knight)
        {
            return true;
        }
        false
    }

    pub fn generate_moves(&self, color: Color) -> Vec<Move> {
        let mut moves = Vec::new();

        for row in 0..8usize {
            for col in 0..8usize {
                if let Some(piece) = self.squares[row][col] {
                    if piece.color != color {
                        continue;
                    }
                    match piece.piece_type {
                        PieceType::Pawn => self.generate_pawn_moves(row, col, color, &mut moves),
                        PieceType::Knight => self.generate_knight_moves(row, col, color, &mut moves),
                        PieceType::Bishop => self.generate_bishop_moves(row, col, color, &mut moves),
                        PieceType::Rook => self.generate_rook_moves(row, col, color, &mut moves),
                        PieceType::Queen => self.generate_queen_moves(row, col, color, &mut moves),
                        PieceType::King => self.generate_king_moves(row, col, color, &mut moves),
                    }
                }
            }
        }

        moves
    }

    fn generate_pawn_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        let (dir, start_row, promo_row): (i32, usize, usize) = match color {
            Color::White => (1, 1, 7),
            Color::Black => (-1, 6, 0),
        };

        let forward = row as i32 + dir;

        // Single push
        if Self::in_bounds(forward, col as i32) && self.squares[forward as usize][col].is_none() {
            if forward as usize == promo_row {
                for pt in &[PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                    moves.push(Move {
                        from: (row, col),
                        to: (forward as usize, col),
                        promotion: Some(*pt),
                    });
                }
            } else {
                moves.push(Move {
                    from: (row, col),
                    to: (forward as usize, col),
                    promotion: None,
                });

                // Double push
                if row == start_row {
                    let double = forward + dir;
                    if Self::in_bounds(double, col as i32)
                        && self.squares[double as usize][col].is_none()
                    {
                        moves.push(Move {
                            from: (row, col),
                            to: (double as usize, col),
                            promotion: None,
                        });
                    }
                }
            }
        }

        // Captures (including en passant)
        for dc in &[-1i32, 1] {
            let nc = col as i32 + dc;
            if !Self::in_bounds(forward, nc) {
                continue;
            }
            let tr = forward as usize;
            let tc = nc as usize;

            let is_capture = self.squares[tr][tc]
                .map(|p| p.color != color)
                .unwrap_or(false);
            let is_en_passant = self.en_passant_target == Some((tr, tc));

            if is_capture || is_en_passant {
                if tr == promo_row {
                    for pt in &[PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                        moves.push(Move {
                            from: (row, col),
                            to: (tr, tc),
                            promotion: Some(*pt),
                        });
                    }
                } else {
                    moves.push(Move {
                        from: (row, col),
                        to: (tr, tc),
                        promotion: None,
                    });
                }
            }
        }
    }

    fn generate_knight_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        let offsets: [(i32, i32); 8] = [
            (-2, -1), (-2, 1), (-1, -2), (-1, 2),
            (1, -2), (1, 2), (2, -1), (2, 1),
        ];
        for (dr, dc) in &offsets {
            let r = row as i32 + dr;
            let c = col as i32 + dc;
            if !Self::in_bounds(r, c) {
                continue;
            }
            let tr = r as usize;
            let tc = c as usize;
            if self.squares[tr][tc].map(|p| p.color == color).unwrap_or(false) {
                continue;
            }
            moves.push(Move {
                from: (row, col),
                to: (tr, tc),
                promotion: None,
            });
        }
    }

    fn generate_sliding_moves(
        &self,
        row: usize,
        col: usize,
        color: Color,
        directions: &[(i32, i32)],
        moves: &mut Vec<Move>,
    ) {
        for (dr, dc) in directions {
            let mut r = row as i32 + dr;
            let mut c = col as i32 + dc;
            while Self::in_bounds(r, c) {
                let tr = r as usize;
                let tc = c as usize;
                if let Some(p) = self.squares[tr][tc] {
                    if p.color != color {
                        moves.push(Move {
                            from: (row, col),
                            to: (tr, tc),
                            promotion: None,
                        });
                    }
                    break;
                }
                moves.push(Move {
                    from: (row, col),
                    to: (tr, tc),
                    promotion: None,
                });
                r += dr;
                c += dc;
            }
        }
    }

    fn generate_bishop_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        let dirs = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
        self.generate_sliding_moves(row, col, color, &dirs, moves);
    }

    fn generate_rook_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        let dirs = [(0, 1), (0, -1), (1, 0), (-1, 0)];
        self.generate_sliding_moves(row, col, color, &dirs, moves);
    }

    fn generate_queen_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        let dirs = [
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        self.generate_sliding_moves(row, col, color, &dirs, moves);
    }

    fn generate_king_moves(&self, row: usize, col: usize, color: Color, moves: &mut Vec<Move>) {
        for dr in -1..=1i32 {
            for dc in -1..=1i32 {
                if dr == 0 && dc == 0 {
                    continue;
                }
                let r = row as i32 + dr;
                let c = col as i32 + dc;
                if !Self::in_bounds(r, c) {
                    continue;
                }
                let tr = r as usize;
                let tc = c as usize;
                if self.squares[tr][tc].map(|p| p.color == color).unwrap_or(false) {
                    continue;
                }
                moves.push(Move {
                    from: (row, col),
                    to: (tr, tc),
                    promotion: None,
                });
            }
        }

        // Castling
        let back_rank = match color {
            Color::White => 0,
            Color::Black => 7,
        };

        if row != back_rank || col != 4 {
            return;
        }

        if self.is_in_check(color) {
            return;
        }

        // Kingside
        let can_kingside = match color {
            Color::White => self.castling_rights.white_kingside,
            Color::Black => self.castling_rights.black_kingside,
        };
        if can_kingside {
            // Squares between king and rook must be empty (cols 5, 6)
            if self.squares[back_rank][5].is_none()
                && self.squares[back_rank][6].is_none()
                // Rook must be present
                && self.squares[back_rank][7]
                    .map(|p| p.piece_type == PieceType::Rook && p.color == color)
                    .unwrap_or(false)
                // King must not pass through check (col 5) or land in check (col 6)
                && !self.is_square_attacked_by(back_rank, 5, color.opposite())
                && !self.is_square_attacked_by(back_rank, 6, color.opposite())
            {
                moves.push(Move {
                    from: (row, col),
                    to: (back_rank, 6),
                    promotion: None,
                });
            }
        }

        // Queenside
        let can_queenside = match color {
            Color::White => self.castling_rights.white_queenside,
            Color::Black => self.castling_rights.black_queenside,
        };
        if can_queenside {
            // Squares between king and rook must be empty (cols 1, 2, 3)
            if self.squares[back_rank][1].is_none()
                && self.squares[back_rank][2].is_none()
                && self.squares[back_rank][3].is_none()
                // Rook must be present
                && self.squares[back_rank][0]
                    .map(|p| p.piece_type == PieceType::Rook && p.color == color)
                    .unwrap_or(false)
                // King must not pass through check (col 3) or land in check (col 2)
                && !self.is_square_attacked_by(back_rank, 3, color.opposite())
                && !self.is_square_attacked_by(back_rank, 2, color.opposite())
            {
                moves.push(Move {
                    from: (row, col),
                    to: (back_rank, 2),
                    promotion: None,
                });
            }
        }
    }

    pub fn generate_legal_moves(&self, color: Color) -> Vec<Move> {
        let pseudo_legal = self.generate_moves(color);
        pseudo_legal
            .into_iter()
            .filter(|m| {
                let mut clone = self.clone();
                clone.apply_move_no_check(m);
                !clone.is_in_check(color)
            })
            .collect()
    }

    /// Apply a move without checking for game-over conditions (used internally).
    fn apply_move_no_check(&mut self, m: &Move) {
        let (fr, fc) = m.from;
        let (tr, tc) = m.to;
        self.last_move = Some(((fr, fc), (tr, tc)));

        let piece = match self.squares[fr][fc] {
            Some(p) => p,
            None => return,
        };

        let is_pawn_move = piece.piece_type == PieceType::Pawn;

        // Record capture
        if let Some(captured) = self.squares[tr][tc] {
            match captured.color {
                Color::White => self.captured_white.push(captured.piece_type),
                Color::Black => self.captured_black.push(captured.piece_type),
            }
        }

        let is_capture = self.squares[tr][tc].is_some();

        // En passant capture
        if is_pawn_move && Some((tr, tc)) == self.en_passant_target {
            let captured_row = fr;
            if let Some(ep_piece) = self.squares[captured_row][tc] {
                match ep_piece.color {
                    Color::White => self.captured_white.push(ep_piece.piece_type),
                    Color::Black => self.captured_black.push(ep_piece.piece_type),
                }
            }
            self.squares[captured_row][tc] = None;
        }

        // Move the piece
        self.squares[tr][tc] = Some(piece);
        self.squares[fr][fc] = None;

        // Handle promotion
        if let Some(promo_type) = m.promotion {
            self.squares[tr][tc] = Some(Piece::new(promo_type, piece.color));
        }

        // Handle castling (move the rook)
        if piece.piece_type == PieceType::King {
            let col_diff = tc as i32 - fc as i32;
            if col_diff == 2 {
                // Kingside
                self.squares[fr][5] = self.squares[fr][7];
                self.squares[fr][7] = None;
            } else if col_diff == -2 {
                // Queenside
                self.squares[fr][3] = self.squares[fr][0];
                self.squares[fr][0] = None;
            }
        }

        // Update castling rights
        if piece.piece_type == PieceType::King {
            match piece.color {
                Color::White => {
                    self.castling_rights.white_kingside = false;
                    self.castling_rights.white_queenside = false;
                }
                Color::Black => {
                    self.castling_rights.black_kingside = false;
                    self.castling_rights.black_queenside = false;
                }
            }
        }
        if piece.piece_type == PieceType::Rook {
            match (piece.color, fr, fc) {
                (Color::White, 0, 0) => self.castling_rights.white_queenside = false,
                (Color::White, 0, 7) => self.castling_rights.white_kingside = false,
                (Color::Black, 7, 0) => self.castling_rights.black_queenside = false,
                (Color::Black, 7, 7) => self.castling_rights.black_kingside = false,
                _ => {}
            }
        }
        // If a rook is captured, also revoke castling rights
        match (tr, tc) {
            (0, 0) => self.castling_rights.white_queenside = false,
            (0, 7) => self.castling_rights.white_kingside = false,
            (7, 0) => self.castling_rights.black_queenside = false,
            (7, 7) => self.castling_rights.black_kingside = false,
            _ => {}
        }

        // Update en passant target
        if is_pawn_move && ((fr as i32 - tr as i32).abs() == 2) {
            let ep_row = (fr + tr) / 2;
            self.en_passant_target = Some((ep_row, fc));
        } else {
            self.en_passant_target = None;
        }

        // Update halfmove clock
        if is_pawn_move || is_capture {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }

        // Toggle turn
        if self.current_turn == Color::Black {
            self.fullmove_number += 1;
        }
        self.current_turn = self.current_turn.opposite();
        self.position_history.push(self.position_hash());
    }

    /// Apply a move and check for game-over conditions.
    pub fn apply_move(&mut self, m: &Move) {
        self.apply_move_no_check(m);

        // Check for game-over conditions
        let legal_moves = self.generate_legal_moves(self.current_turn);
        if legal_moves.is_empty() {
            self.game_over = true;
            if self.is_in_check(self.current_turn) {
                // Checkmate
                self.result = Some(match self.current_turn {
                    Color::White => "Black wins".to_string(),
                    Color::Black => "White wins".to_string(),
                });
            } else {
                // Stalemate
                self.result = Some("Draw".to_string());
            }
        }

        // 50-move rule
        if self.halfmove_clock >= 100 {
            self.game_over = true;
            self.result = Some("Draw — 50 move rule".to_string());
        }

        // Threefold repetition
        if !self.game_over && self.is_threefold_repetition() {
            self.game_over = true;
            self.result = Some("Draw by repetition".to_string());
        }

        // Insufficient material
        if !self.game_over && self.has_insufficient_material() {
            self.game_over = true;
            self.result = Some("Draw — insufficient material".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: make a move from algebraic-style coordinates.
    fn mv(from: (usize, usize), to: (usize, usize)) -> Move {
        Move { from, to, promotion: None }
    }

    /// Threefold repetition requires the same position to occur THREE times,
    /// not two. Shuffle a knight back and forth so the position repeats twice
    /// (two full round-trips) — the game must NOT be over yet. Only on the
    /// third occurrence should it be declared a draw.
    #[test]
    fn threefold_repetition_requires_three_occurrences() {
        // Kings + rooks (sufficient material) + a white knight to shuffle.
        let mut board = Board::empty();
        board.squares[0][4] = Some(Piece::new(PieceType::King, Color::White));
        board.squares[7][4] = Some(Piece::new(PieceType::King, Color::Black));
        board.squares[0][0] = Some(Piece::new(PieceType::Rook, Color::White));
        board.squares[7][0] = Some(Piece::new(PieceType::Rook, Color::Black));
        board.squares[0][1] = Some(Piece::new(PieceType::Knight, Color::White));
        board.current_turn = Color::White;
        board.position_history.push(board.position_hash());

        // Round-trip 1: Nb1→c3, Ke8→d8, Nc3→b1, Kd8→e8
        // Returns to the initial position → 2nd occurrence. NOT a draw yet.
        board.apply_move(&mv((0, 1), (2, 2))); // Nb1→c3
        assert!(!board.game_over);
        board.apply_move(&mv((7, 4), (7, 3))); // Ke8→d8
        assert!(!board.game_over);
        board.apply_move(&mv((2, 2), (0, 1))); // Nc3→b1
        assert!(!board.game_over);
        board.apply_move(&mv((7, 3), (7, 4))); // Kd8→e8
        assert!(!board.game_over, "game ended after only 2 occurrences — should require 3");

        // Round-trip 2: same moves → 3rd occurrence = draw.
        board.apply_move(&mv((0, 1), (2, 2))); // Nb1→c3
        assert!(!board.game_over);
        board.apply_move(&mv((7, 4), (7, 3))); // Ke8→d8
        assert!(!board.game_over);
        board.apply_move(&mv((2, 2), (0, 1))); // Nc3→b1
        assert!(!board.game_over);
        board.apply_move(&mv((7, 3), (7, 4))); // Kd8→e8
        assert!(board.game_over, "game should be over after 3 occurrences");
        assert_eq!(board.result.as_deref(), Some("Draw by repetition"));
    }
}
