use chess::board::Board;
use chess::engine::{pick_move, AiConfig, Weights};

const MAX_MOVES: u32 = 150;
const GAMES_PER_MATCHUP: usize = 10;
const SIM_DEPTH: u32 = 1;

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
    let mut result = MatchResult {
        white_wins: 0,
        black_wins: 0,
        draws: 0,
    };

    // Each config plays both colors
    let half = GAMES_PER_MATCHUP / 2;

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
        "  {label_a} vs {label_b}: {label_a} wins {a_total_wins}, {label_b} wins {b_total_wins}, draws {total_draws} (out of {GAMES_PER_MATCHUP})"
    );

    MatchResult {
        white_wins: a_total_wins,
        black_wins: b_total_wins,
        draws: total_draws,
    }
}

fn main() {
    println!("=== Chess AI Weight Optimization ===");
    println!("Games per matchup: {GAMES_PER_MATCHUP}, max moves per game: {MAX_MOVES}, depth: {SIM_DEPTH}\n");

    let mut baseline = AiConfig::new();
    baseline.depth = SIM_DEPTH;

    // Weight variations to test
    let variations: Vec<(&str, AiConfig)> = vec![
        ("baseline", baseline.clone()),
        // Centre attack weight
        ("centre_atk=0.15", make_config(|w| w.centre_attack = 0.15)),
        ("centre_atk=0.5", make_config(|w| w.centre_attack = 0.5)),
        ("centre_atk=0.7", make_config(|w| w.centre_attack = 0.7)),
        // Centre occupy weight
        ("centre_occ=0.2", make_config(|w| w.centre_occupy = 0.2)),
        ("centre_occ=0.6", make_config(|w| w.centre_occupy = 0.6)),
        ("centre_occ=0.8", make_config(|w| w.centre_occupy = 0.8)),
        // Extended centre
        ("ext_centre=0.0", make_config(|w| w.extended_centre_attack = 0.0)),
        ("ext_centre=0.2", make_config(|w| w.extended_centre_attack = 0.2)),
        ("ext_centre=0.3", make_config(|w| w.extended_centre_attack = 0.3)),
        // Passed pawn base
        ("pp_base=0.2", make_config(|w| w.passed_pawn_base = 0.2)),
        ("pp_base=0.8", make_config(|w| w.passed_pawn_base = 0.8)),
        ("pp_base=1.2", make_config(|w| w.passed_pawn_base = 1.2)),
        // Passed pawn quadratic
        ("pp_quad=0.08", make_config(|w| w.passed_pawn_quadratic = 0.08)),
        ("pp_quad=0.3", make_config(|w| w.passed_pawn_quadratic = 0.3)),
        ("pp_quad=0.5", make_config(|w| w.passed_pawn_quadratic = 0.5)),
        // Pawn advance
        ("pawn_adv=0.0", make_config(|w| w.pawn_advance = 0.0)),
        ("pawn_adv=0.1", make_config(|w| w.pawn_advance = 0.1)),
        ("pawn_adv=0.2", make_config(|w| w.pawn_advance = 0.2)),
        // Repeat penalty
        ("rep_pen=0.5", make_config(|w| w.repeat_penalty = 0.5)),
        ("rep_pen=1.0", make_config(|w| w.repeat_penalty = 1.0)),
        ("rep_pen=3.0", make_config(|w| w.repeat_penalty = 3.0)),
    ];

    // Phase 1: test each variation against the baseline
    println!("--- Phase 1: Each variation vs baseline ---\n");
    let mut scores: Vec<(&str, i32)> = Vec::new();

    for (label, config) in &variations {
        if *label == "baseline" {
            continue;
        }
        let result = run_matchup(label, config, "baseline", &baseline);
        // Score: +1 per win, -1 per loss
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

    // Find the best for each weight parameter
    let categories = [
        ("centre_atk", vec!["centre_atk=0.15", "centre_atk=0.5", "centre_atk=0.7"]),
        ("centre_occ", vec!["centre_occ=0.2", "centre_occ=0.6", "centre_occ=0.8"]),
        ("ext_centre", vec!["ext_centre=0.0", "ext_centre=0.2", "ext_centre=0.3"]),
        ("pp_base", vec!["pp_base=0.2", "pp_base=0.8", "pp_base=1.2"]),
        ("pp_quad", vec!["pp_quad=0.08", "pp_quad=0.3", "pp_quad=0.5"]),
        ("pawn_adv", vec!["pawn_adv=0.0", "pawn_adv=0.1", "pawn_adv=0.2"]),
        ("rep_pen", vec!["rep_pen=0.5", "rep_pen=1.0", "rep_pen=3.0"]),
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
            // Apply the best weight
            apply_weight(&mut best_weights, best_label);
        } else {
            println!("    {cat}: baseline (no improvement found)");
        }
    }

    let mut combined = AiConfig::new();
    combined.depth = SIM_DEPTH;
    combined.weights = best_weights.clone();

    println!("\n  Combined weights: {best_weights:?}");
    println!("\n  Testing combined vs baseline...\n");

    let result = run_matchup("combined", &combined, "baseline", &baseline);

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
    println!("  repeat_penalty: {}", best_weights.repeat_penalty);
}

fn make_config(f: impl FnOnce(&mut Weights)) -> AiConfig {
    let mut config = AiConfig::new();
    config.depth = SIM_DEPTH;
    f(&mut config.weights);
    config
}

fn apply_weight(weights: &mut Weights, label: &str) {
    match label {
        "centre_atk=0.15" => weights.centre_attack = 0.15,
        "centre_atk=0.5" => weights.centre_attack = 0.5,
        "centre_atk=0.7" => weights.centre_attack = 0.7,
        "centre_occ=0.2" => weights.centre_occupy = 0.2,
        "centre_occ=0.6" => weights.centre_occupy = 0.6,
        "centre_occ=0.8" => weights.centre_occupy = 0.8,
        "ext_centre=0.0" => weights.extended_centre_attack = 0.0,
        "ext_centre=0.2" => weights.extended_centre_attack = 0.2,
        "ext_centre=0.3" => weights.extended_centre_attack = 0.3,
        "pp_base=0.2" => weights.passed_pawn_base = 0.2,
        "pp_base=0.8" => weights.passed_pawn_base = 0.8,
        "pp_base=1.2" => weights.passed_pawn_base = 1.2,
        "pp_quad=0.08" => weights.passed_pawn_quadratic = 0.08,
        "pp_quad=0.3" => weights.passed_pawn_quadratic = 0.3,
        "pp_quad=0.5" => weights.passed_pawn_quadratic = 0.5,
        "pawn_adv=0.0" => weights.pawn_advance = 0.0,
        "pawn_adv=0.1" => weights.pawn_advance = 0.1,
        "pawn_adv=0.2" => weights.pawn_advance = 0.2,
        "rep_pen=0.5" => weights.repeat_penalty = 0.5,
        "rep_pen=1.0" => weights.repeat_penalty = 1.0,
        "rep_pen=3.0" => weights.repeat_penalty = 3.0,
        _ => {}
    }
}
