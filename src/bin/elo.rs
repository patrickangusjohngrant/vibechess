use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use chess::board::Board;
use chess::engine::{pick_move, AiConfig};
use chess::moves::Move;
use chess::piece::Color;

const STOCKFISH_PATH: &str = "/home/patrick/.local/bin/stockfish";
const MAX_MOVES: u32 = 200;
const GAMES_PER_CONFIG: usize = 10;

struct StockfishEngine {
    child: std::process::Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl StockfishEngine {
    fn new(skill_level: u32, move_time_ms: u32) -> Self {
        let mut child = Command::new(STOCKFISH_PATH)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start Stockfish");

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let mut sf = StockfishEngine {
            child,
            stdin,
            reader,
        };

        sf.send("uci");
        sf.wait_for("uciok");
        sf.send(&format!("setoption name Skill Level value {skill_level}"));
        sf.send(&format!("setoption name Move Overhead value {move_time_ms}"));
        sf.send("isready");
        sf.wait_for("readyok");

        sf
    }

    fn send(&mut self, cmd: &str) {
        writeln!(self.stdin, "{}", cmd).unwrap();
        self.stdin.flush().unwrap();
    }

    fn wait_for(&mut self, prefix: &str) -> String {
        let mut line = String::new();
        loop {
            line.clear();
            self.reader.read_line(&mut line).unwrap();
            if line.trim().starts_with(prefix) {
                return line.trim().to_string();
            }
        }
    }

    fn get_best_move(&mut self, moves: &[String], move_time_ms: u32) -> String {
        let pos_cmd = if moves.is_empty() {
            "position startpos".to_string()
        } else {
            format!("position startpos moves {}", moves.join(" "))
        };
        self.send(&pos_cmd);
        self.send(&format!("go movetime {move_time_ms}"));

        let line = self.wait_for("bestmove");
        line.split_whitespace()
            .nth(1)
            .unwrap_or("0000")
            .to_string()
    }

    fn shutdown(&mut self) {
        self.send("quit");
        let _ = self.child.wait();
    }
}

impl Drop for StockfishEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn play_game(
    ai_config: &AiConfig,
    ai_color: Color,
    sf_skill: u32,
    sf_time_ms: u32,
) -> &'static str {
    let mut sf = StockfishEngine::new(sf_skill, sf_time_ms);
    let mut board = Board::new();
    let mut uci_moves: Vec<String> = Vec::new();

    for _ in 0..MAX_MOVES {
        if board.game_over {
            break;
        }

        if board.current_turn == ai_color {
            match pick_move(&board, ai_config) {
                Some(result) => {
                    uci_moves.push(result.mv.to_uci());
                    board.apply_move(&result.mv);
                }
                None => break,
            }
        } else {
            let sf_uci = sf.get_best_move(&uci_moves, sf_time_ms);
            if sf_uci == "0000" || sf_uci == "(none)" {
                break;
            }
            let legal_moves = board.generate_legal_moves(board.current_turn);
            let parsed = Move::from_uci(&sf_uci);
            let matching = parsed.and_then(|pm| {
                legal_moves.iter().find(|m| {
                    m.from == pm.from && m.to == pm.to && m.promotion == pm.promotion
                })
            });
            match matching {
                Some(m) => {
                    uci_moves.push(m.to_uci());
                    board.apply_move(&m.clone());
                }
                None => {
                    eprintln!("  Stockfish returned illegal move: {sf_uci}");
                    break;
                }
            }
        }
    }

    if board.game_over {
        match board.result.as_deref() {
            Some("White wins") => {
                if ai_color == Color::White {
                    "win"
                } else {
                    "loss"
                }
            }
            Some("Black wins") => {
                if ai_color == Color::Black {
                    "win"
                } else {
                    "loss"
                }
            }
            _ => "draw",
        }
    } else {
        "draw"
    }
}

/// Estimate ELO difference from score.
/// score = (wins + 0.5*draws) / total
fn elo_diff(wins: u32, draws: u32, losses: u32) -> f64 {
    let total = (wins + draws + losses) as f64;
    if total == 0.0 {
        return 0.0;
    }
    let score = (wins as f64 + 0.5 * draws as f64) / total;
    if score <= 0.0 {
        return -999.0;
    }
    if score >= 1.0 {
        return 999.0;
    }
    -400.0 * (1.0 / score - 1.0).log10()
}

fn run_config(
    label: &str,
    config: &AiConfig,
    sf_skill: u32,
    sf_time_ms: u32,
) -> (u32, u32, u32) {
    let mut wins = 0u32;
    let mut draws = 0u32;
    let mut losses = 0u32;

    let half = GAMES_PER_CONFIG / 2;

    // Play as White
    for i in 0..half {
        let result = play_game(config, Color::White, sf_skill, sf_time_ms);
        match result {
            "win" => wins += 1,
            "draw" => draws += 1,
            _ => losses += 1,
        }
        print!("  [{label} as W] game {}: {result}  ", i + 1);
    }
    println!();

    // Play as Black
    for i in 0..half {
        let result = play_game(config, Color::Black, sf_skill, sf_time_ms);
        match result {
            "win" => wins += 1,
            "draw" => draws += 1,
            _ => losses += 1,
        }
        print!("  [{label} as B] game {}: {result}  ", i + 1);
    }
    println!();

    (wins, draws, losses)
}

fn main() {
    println!("=== Chess Engine ELO Estimation vs Stockfish ===\n");

    // Stockfish at very low skill (Skill 0 ≈ ~800 ELO, Skill 1 ≈ ~1000)
    // with short time to keep games fast
    let sf_skill = 0;
    let sf_time_ms = 50;

    println!("Stockfish config: skill={sf_skill}, movetime={sf_time_ms}ms");
    println!("Games per config: {GAMES_PER_CONFIG} ({} as White, {} as Black)", GAMES_PER_CONFIG / 2, GAMES_PER_CONFIG / 2);
    println!();

    let configs: Vec<(&str, AiConfig)> = vec![
        ("d2 all-on", {
            let mut c = AiConfig::new();
            c.depth = 2;
            c.auto_deepen = false;
            c
        }),
        ("d2 mat+centre", {
            let mut c = AiConfig::new();
            c.depth = 2;
            c.auto_deepen = false;
            c.passed_pawn_module = false;
            c
        }),
        ("d1 random", {
            let mut c = AiConfig::new();
            c.depth = 1;
            c.auto_deepen = false;
            c.mate_module = false;
            c.material_module = false;
            c.centre_module = false;
            c.passed_pawn_module = false;
            c
        }),
    ];

    println!("{:<25} {:>4} {:>5} {:>6} {:>8}", "Config", "W", "D", "L", "ELO±");
    println!("{}", "-".repeat(55));

    for (label, config) in &configs {
        let (w, d, l) = run_config(label, config, sf_skill, sf_time_ms);
        let elo = elo_diff(w, d, l);
        println!(
            "{:<25} {:>4} {:>5} {:>6} {:>+8.0}",
            label, w, d, l, elo
        );
        println!();
    }

    println!("\nNote: ELO± is relative to Stockfish skill {sf_skill} (~800 ELO).");
    println!("Estimated absolute ELO = 800 + ELO±");
}
