use crate::chess::ChessInstant;
use crate::movemap::MoveMap;

pub struct BotSettings {
    pub search_depth: i32,
    pub pawn_value: i32,
    pub rook_value: i32,
    pub knight_value: i32,
    pub bishop_value: i32,
    pub queen_value: i32,
    pub king_value: i32,
    pub check_value: i32,
    pub position_mult: i32,
    pub attack_div: i32,
}

impl BotSettings {
    pub fn new() -> BotSettings {
        BotSettings {
            search_depth: 0,
            pawn_value: 100,
            rook_value: 563,
            knight_value: 305,
            bishop_value: 333,
            queen_value: 950,
            king_value: 000,
            check_value: 90,
            position_mult: 10,
            attack_div: 25,
        }
    }

    pub fn new_depth(depth: i32) -> BotSettings {
        let depth = if depth < 0 { 0 } else { depth };
        let mut bot = BotSettings::new();
        bot.search_depth = depth;
        bot
    }
}

/// returns a score for every possible move that could be made for the given chess instant,
/// if no valid moves returns an empty vector,
/// the heuristic fn has to calculate the value for the current player for the given chess instant
pub fn minimax(
    mm: &MoveMap,
    bot: &BotSettings,
    root: &ChessInstant,
    heuristic_fn: fn(&ChessInstant, &MoveMap, &BotSettings) -> i32,
) -> Vec<(ChessInstant, i32)> {
    let l1_moves = root.valid_games(mm);
    let mut graded = Vec::new();
    for l1ci in l1_moves {
        graded.push((
            l1ci.clone(),
            minmax_sub(
                &l1ci,
                mm,
                bot,
                bot.search_depth,
                false,
                i32::MIN,
                i32::MAX,
                heuristic_fn,
            ),
        ));
    }
    graded.sort_unstable_by(|(_, a), (_, b)| b.cmp(a)); // sort max to min

    return graded;

    // recursive min max fuction
    fn minmax_sub(
        ci: &ChessInstant,
        mm: &MoveMap,
        bot: &BotSettings,
        depth: i32,
        maximizing: bool,
        mut alpha: i32,
        mut beta: i32,
        heuristic_fn: fn(&ChessInstant, &MoveMap, &BotSettings) -> i32,
    ) -> i32 {
        if depth == 0 {
            // the heuristic value of the game
            if maximizing {
                return heuristic_fn(ci, mm, bot);
            } else {
                // multiply by -1 so minimizing will get the best score for the opponent
                // and maximizing will get the worst move for self
                return heuristic_fn(ci, mm, bot) * -1;
            }
        }

        let ci_children = ci.valid_games(mm);
        if ci_children.len() == 0 {
            if !ci.in_check(mm, &ci.king_id()) {
                return 0; // score for a draw
            }
            // else checkmate
            if maximizing {
                return i32::MIN + bot.search_depth - depth
            } else {
                return i32::MAX - bot.search_depth + depth
            }
        }
        if maximizing {
            let mut value = i32::MIN; // + bot.search_depth - depth;
            for child in ci_children {
                let score =
                    minmax_sub(&child, mm, bot, depth - 1, false, alpha, beta, heuristic_fn);
                value = value.max(score); // maximize score
                if value > beta {
                    break; // beta cutoff
                }

                alpha = if alpha > value { alpha } else { value };
            }
            return value;
        } else {
            let mut value = i32::MAX; // - bot.search_depth + depth;
            for child in ci_children {
                let score = minmax_sub(&child, mm, bot, depth - 1, true, alpha, beta, heuristic_fn);
                value = value.min(score); // minimize score
                if value < alpha {
                    break; // alpha cutoff
                }
                beta = if beta < value { beta } else { value };
            }
            return value;
        }
    }
}
