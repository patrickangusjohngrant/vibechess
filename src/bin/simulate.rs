use std::io::Write;
use chess::board::Board;
use chess::engine::{pick_move, AiConfig, Weights};

const MAX_MOVES: u32 = 150;
const GAMES_PER_MATCHUP: usize = 10;
const PHASE2_GAMES: usize = 10;

#[derive(Debug)]
struct MatchResult {
    white_wins: u32,
    black_wins: u32,
    draws: u32,
}

fn play_game(white_config: &AiConfig, black_config: &AiConfig) -> Option<&'static str> {
    let mut board = Board::new();
    for _ in 0..MAX_MOVES {
        if board.game_over {
            break;
        }
        let config = if board.current_turn == chess::piece::Color::White {
            white_config
        } else {
            black_config
        };
        match pick_move(&board, config) {
            Some(result) => board.apply_move(&result.mv),
            None => break,
        }
    }

    if board.game_over {
        board.result.as_deref().map(|r| match r {
            "White wins" => "white",
            "Black wins" => "black",
            _ => "draw",
        })
    } else {
        Some("draw") // hit move limit
    }
}

fn run_matchup(
    label_a: &str,
    config_a: &AiConfig,
    label_b: &str,
    config_b: &AiConfig,
) -> MatchResult {
    run_matchup_n(label_a, config_a, label_b, config_b, GAMES_PER_MATCHUP)
}

fn run_matchup_n(
    label_a: &str,
    config_a: &AiConfig,
    label_b: &str,
    config_b: &AiConfig,
    num_games: usize,
) -> MatchResult {
    let mut result = MatchResult {
        white_wins: 0,
        black_wins: 0,
        draws: 0,
    };

    let half = num_games / 2;

    // A as white, B as black
    for _ in 0..half {
        match play_game(config_a, config_b) {
            Some("white") => result.white_wins += 1,
            Some("black") => result.black_wins += 1,
            _ => result.draws += 1,
        }
    }
    let a_white_wins = result.white_wins;
    let b_black_wins = result.black_wins;
    let draws_1 = result.draws;

    // B as white, A as black
    let mut result2 = MatchResult {
        white_wins: 0,
        black_wins: 0,
        draws: 0,
    };
    for _ in 0..half {
        match play_game(config_b, config_a) {
            Some("white") => result2.white_wins += 1,
            Some("black") => result2.black_wins += 1,
            _ => result2.draws += 1,
        }
    }

    let a_total_wins = a_white_wins + result2.black_wins;
    let b_total_wins = b_black_wins + result2.white_wins;
    let total_draws = draws_1 + result2.draws;

    println!(
        "  {label_a} vs {label_b}: {label_a} wins {a_total_wins}, {label_b} wins {b_total_wins}, draws {total_draws} (out of {num_games})"
    );
    std::io::stdout().flush().ok();

    MatchResult {
        white_wins: a_total_wins,
        black_wins: b_total_wins,
        draws: total_draws,
    }
}

/// Simulation baseline: depth 1, no auto-deepen for speed.
/// Weight results still apply to medium (auto-deepen just deepens the search,
/// it doesn't change how weights are used).
fn medium_config() -> AiConfig {
    let mut c = AiConfig::new();
    c.depth = 1;
    c.auto_deepen = false;
    c
}

fn make_config(f: impl FnOnce(&mut Weights)) -> AiConfig {
    let mut config = medium_config();
    f(&mut config.weights);
    config
}

fn main() {
    println!("=== Chess AI Weight Optimization (medium: d1 auto-deepen 25k) ===");
    println!("Games per matchup: {GAMES_PER_MATCHUP}, max moves per game: {MAX_MOVES}\n");

    let baseline = medium_config();

    // Weight variations to test
    let variations: Vec<(&str, AiConfig)> = vec![
        ("baseline", baseline.clone()),
        // Centre attack weight
        ("centre_atk=0.05", make_config(|w| w.centre_attack = 0.05)),
        ("centre_atk=0.3", make_config(|w| w.centre_attack = 0.3)),
        ("centre_atk=0.5", make_config(|w| w.centre_attack = 0.5)),
        // Centre occupy weight
        ("centre_occ=0.1", make_config(|w| w.centre_occupy = 0.1)),
        ("centre_occ=0.4", make_config(|w| w.centre_occupy = 0.4)),
        ("centre_occ=0.8", make_config(|w| w.centre_occupy = 0.8)),
        // Extended centre
        ("ext_centre=0.1", make_config(|w| w.extended_centre_attack = 0.1)),
        ("ext_centre=0.2", make_config(|w| w.extended_centre_attack = 0.2)),
        ("ext_centre=0.5", make_config(|w| w.extended_centre_attack = 0.5)),
        // Passed pawn base
        ("pp_base=0.1", make_config(|w| w.passed_pawn_base = 0.1)),
        ("pp_base=0.5", make_config(|w| w.passed_pawn_base = 0.5)),
        ("pp_base=1.0", make_config(|w| w.passed_pawn_base = 1.0)),
        // Passed pawn quadratic
        ("pp_quad=0.1", make_config(|w| w.passed_pawn_quadratic = 0.1)),
        ("pp_quad=0.5", make_config(|w| w.passed_pawn_quadratic = 0.5)),
        ("pp_quad=0.8", make_config(|w| w.passed_pawn_quadratic = 0.8)),
        // Pawn advance
        ("pawn_adv=0.0", make_config(|w| w.pawn_advance = 0.0)),
        ("pawn_adv=0.1", make_config(|w| w.pawn_advance = 0.1)),
        ("pawn_adv=0.2", make_config(|w| w.pawn_advance = 0.2)),
        // Check penalty
        ("chk_pen=0.0", make_config(|w| w.check_penalty = 0.0)),
        ("chk_pen=0.3", make_config(|w| w.check_penalty = 0.3)),
        ("chk_pen=1.0", make_config(|w| w.check_penalty = 1.0)),
        ("chk_pen=2.0", make_config(|w| w.check_penalty = 2.0)),
        // Repeat penalty
        ("rep_pen=0.5", make_config(|w| w.repeat_penalty = 0.5)),
        ("rep_pen=5.0", make_config(|w| w.repeat_penalty = 5.0)),
        ("rep_pen=20.0", make_config(|w| w.repeat_penalty = 20.0)),
        // Early queen centre multiplier
        ("eq_mult=0.0", make_config(|w| w.early_queen_centre_mult = 0.0)),
        ("eq_mult=0.5", make_config(|w| w.early_queen_centre_mult = 0.5)),
        ("eq_mult=0.8", make_config(|w| w.early_queen_centre_mult = 0.8)),
        ("eq_mult=1.0", make_config(|w| w.early_queen_centre_mult = 1.0)),
        // Early queen until move N
        ("eq_until=5", make_config(|w| w.early_queen_centre_until = 5)),
        ("eq_until=8", make_config(|w| w.early_queen_centre_until = 8)),
        ("eq_until=15", make_config(|w| w.early_queen_centre_until = 15)),
    ];

    // Phase 1: test each variation against the baseline
    println!("--- Phase 1: Each variation vs baseline ---\n");
    let mut scores: Vec<(&str, i32)> = Vec::new();

    for (label, config) in &variations {
        if *label == "baseline" {
            continue;
        }
        print!("  Testing {label}... ");
        std::io::stdout().flush().ok();
        let result = run_matchup(label, config, "baseline", &baseline);
        let net = result.white_wins as i32 - result.black_wins as i32;
        scores.push((label, net));
    }

    scores.sort_by(|a, b| b.1.cmp(&a.1));

    println!("\n--- Phase 1 Rankings (net wins vs baseline) ---\n");
    for (label, net) in &scores {
        let indicator = if *net > 0 {
            "+"
        } else if *net < 0 {
            ""
        } else {
            " "
        };
        println!("  {indicator}{net:>3}  {label}");
    }

    // Phase 2: combine the best from each category and test
    println!("\n--- Phase 2: Combined best weights ---\n");

    let categories = [
        ("centre_atk", vec!["centre_atk=0.05", "centre_atk=0.3", "centre_atk=0.5"]),
        ("centre_occ", vec!["centre_occ=0.1", "centre_occ=0.4", "centre_occ=0.8"]),
        ("ext_centre", vec!["ext_centre=0.1", "ext_centre=0.2", "ext_centre=0.5"]),
        ("pp_base", vec!["pp_base=0.1", "pp_base=0.5", "pp_base=1.0"]),
        ("pp_quad", vec!["pp_quad=0.1", "pp_quad=0.5", "pp_quad=0.8"]),
        ("pawn_adv", vec!["pawn_adv=0.0", "pawn_adv=0.1", "pawn_adv=0.2"]),
        ("chk_pen", vec!["chk_pen=0.0", "chk_pen=0.3", "chk_pen=1.0", "chk_pen=2.0"]),
        ("rep_pen", vec!["rep_pen=0.5", "rep_pen=5.0", "rep_pen=20.0"]),
        ("eq_mult", vec!["eq_mult=0.0", "eq_mult=0.5", "eq_mult=0.8", "eq_mult=1.0"]),
        ("eq_until", vec!["eq_until=5", "eq_until=8", "eq_until=15"]),
    ];

    let mut best_weights = Weights::default();
    println!("  Best per category (vs baseline):");

    for (cat, labels) in &categories {
        let mut best_label = "baseline";
        let mut best_net = 0i32;
        for &label in labels {
            if let Some((_, net)) = scores.iter().find(|(l, _)| *l == label) {
                if *net > best_net {
                    best_net = *net;
                    best_label = label;
                }
            }
        }

        if best_label != "baseline" {
            println!("    {cat}: {best_label} (net {best_net:+})");
            apply_weight(&mut best_weights, best_label);
        } else {
            println!("    {cat}: baseline (no improvement found)");
        }
    }

    let mut combined = medium_config();
    combined.weights = best_weights.clone();

    let baseline_p2 = medium_config();

    println!("\n  Combined weights: {best_weights:?}");
    println!("\n  Testing combined vs baseline ({PHASE2_GAMES} games)...\n");

    let result = run_matchup_n("combined", &combined, "baseline", &baseline_p2, PHASE2_GAMES);

    println!("\n--- Final Result ---\n");
    println!(
        "  Combined wins: {}, Baseline wins: {}, Draws: {}",
        result.white_wins, result.black_wins, result.draws
    );

    println!("\n--- Recommended weights ---\n");
    if result.white_wins > result.black_wins {
        println!("  The combined weights are BETTER than baseline:");
    } else if result.white_wins < result.black_wins {
        println!("  The combined weights are WORSE than baseline, keeping defaults:");
        println!("  {:?}", Weights::default());
        return;
    } else {
        println!("  No significant difference; combined weights:");
    }
    println!("  centre_attack: {}", best_weights.centre_attack);
    println!("  centre_occupy: {}", best_weights.centre_occupy);
    println!("  extended_centre_attack: {}", best_weights.extended_centre_attack);
    println!("  passed_pawn_base: {}", best_weights.passed_pawn_base);
    println!("  passed_pawn_quadratic: {}", best_weights.passed_pawn_quadratic);
    println!("  pawn_advance: {}", best_weights.pawn_advance);
    println!("  check_penalty: {}", best_weights.check_penalty);
    println!("  early_queen_centre_mult: {}", best_weights.early_queen_centre_mult);
    println!("  early_queen_centre_until: {}", best_weights.early_queen_centre_until);
    println!("  repeat_penalty: {}", best_weights.repeat_penalty);
}

fn apply_weight(weights: &mut Weights, label: &str) {
    match label {
        "centre_atk=0.05" => weights.centre_attack = 0.05,
        "centre_atk=0.3" => weights.centre_attack = 0.3,
        "centre_atk=0.5" => weights.centre_attack = 0.5,
        "centre_occ=0.1" => weights.centre_occupy = 0.1,
        "centre_occ=0.4" => weights.centre_occupy = 0.4,
        "centre_occ=0.8" => weights.centre_occupy = 0.8,
        "ext_centre=0.1" => weights.extended_centre_attack = 0.1,
        "ext_centre=0.2" => weights.extended_centre_attack = 0.2,
        "ext_centre=0.5" => weights.extended_centre_attack = 0.5,
        "pp_base=0.1" => weights.passed_pawn_base = 0.1,
        "pp_base=0.5" => weights.passed_pawn_base = 0.5,
        "pp_base=1.0" => weights.passed_pawn_base = 1.0,
        "pp_quad=0.1" => weights.passed_pawn_quadratic = 0.1,
        "pp_quad=0.5" => weights.passed_pawn_quadratic = 0.5,
        "pp_quad=0.8" => weights.passed_pawn_quadratic = 0.8,
        "pawn_adv=0.0" => weights.pawn_advance = 0.0,
        "pawn_adv=0.1" => weights.pawn_advance = 0.1,
        "pawn_adv=0.2" => weights.pawn_advance = 0.2,
        "chk_pen=0.0" => weights.check_penalty = 0.0,
        "chk_pen=0.3" => weights.check_penalty = 0.3,
        "chk_pen=1.0" => weights.check_penalty = 1.0,
        "chk_pen=2.0" => weights.check_penalty = 2.0,
        "rep_pen=0.5" => weights.repeat_penalty = 0.5,
        "rep_pen=5.0" => weights.repeat_penalty = 5.0,
        "rep_pen=20.0" => weights.repeat_penalty = 20.0,
        "eq_mult=0.0" => weights.early_queen_centre_mult = 0.0,
        "eq_mult=0.5" => weights.early_queen_centre_mult = 0.5,
        "eq_mult=0.8" => weights.early_queen_centre_mult = 0.8,
        "eq_mult=1.0" => weights.early_queen_centre_mult = 1.0,
        "eq_until=5" => weights.early_queen_centre_until = 5,
        "eq_until=8" => weights.early_queen_centre_until = 8,
        "eq_until=15" => weights.early_queen_centre_until = 15,
        _ => {}
    }
}
