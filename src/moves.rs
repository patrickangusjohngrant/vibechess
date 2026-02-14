use serde::{Deserialize, Serialize};

use crate::piece::PieceType;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub promotion: Option<PieceType>,
}

impl Move {
    /// Convert to UCI notation, e.g. "e2e4", "a7a8q"
    pub fn to_uci(&self) -> String {
        let fc = (b'a' + self.from.1 as u8) as char;
        let fr = (b'1' + self.from.0 as u8) as char;
        let tc = (b'a' + self.to.1 as u8) as char;
        let tr = (b'1' + self.to.0 as u8) as char;
        let promo = match self.promotion {
            Some(PieceType::Queen) => "q",
            Some(PieceType::Rook) => "r",
            Some(PieceType::Bishop) => "b",
            Some(PieceType::Knight) => "n",
            _ => "",
        };
        format!("{fc}{fr}{tc}{tr}{promo}")
    }

    /// Parse from UCI notation
    pub fn from_uci(s: &str) -> Option<Move> {
        let bytes = s.as_bytes();
        if bytes.len() < 4 {
            return None;
        }
        let fc = (bytes[0] - b'a') as usize;
        let fr = (bytes[1] - b'1') as usize;
        let tc = (bytes[2] - b'a') as usize;
        let tr = (bytes[3] - b'1') as usize;
        let promotion = if bytes.len() > 4 {
            match bytes[4] {
                b'q' => Some(PieceType::Queen),
                b'r' => Some(PieceType::Rook),
                b'b' => Some(PieceType::Bishop),
                b'n' => Some(PieceType::Knight),
                _ => None,
            }
        } else {
            None
        };
        Some(Move {
            from: (fr, fc),
            to: (tr, tc),
            promotion,
        })
    }
}
