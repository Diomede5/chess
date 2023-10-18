use std::ops::Range;

use crate::{
    chess::{ChessInstant, Piece, Player},
    movemap::MoveMap,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BotSettings {
    search_depth: i32,
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

pub fn give_static_score(ci: &ChessInstant, mm: &MoveMap) -> i32 {
    let bot = BotSettings::new();
    return static_score(ci, mm, &bot);
}

pub fn parsed_score(ci: &ChessInstant, mm: &MoveMap) -> String {
    let bot = BotSettings::new();
    let f_range = match ci.player() {
        Player::P1 => 0..6,
        Player::P2 => 7..13,
    };
    let mat = material(ci, mm, &bot, &f_range);
    let check = check(ci, mm, &bot, &f_range);
    let pos = position(ci, mm, &bot, &f_range);
    let atked = attacked(ci, mm, &bot, &f_range);

    let total = mat + check + pos + atked;
    format!(
        "
    Total score: {total}
    \n\tCheck: {check} 
    \n\tMat: {mat} 
    \n\tPos: {pos} 
    \n\tAtk: {atked} 
    "
    )
}

pub fn bot_1_moves_default(ci: &ChessInstant, mm: &MoveMap) -> Vec<(ChessInstant, i32)> {
    let mut num_pieces = 0;
    for (row, col) in mm.locations() {
        let piece = ci.board_get(row, col);
        if piece != 6 {
            num_pieces += 1;
        }
    }

    let depth = match num_pieces {
        29..=32 => 4,
        25..=28 => 4,
        21..=24 => 4,
        17..=20 => 4,
        13..=16 => 5,
        9..=12 => 5,
        4..=8 => 5,
        0..=3 => 8,
        _ => 4,
    };

    let bot = BotSettings::new_depth(depth);
    return minimax(mm, &bot, ci);
}

pub fn bot_1_moves(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings) -> Vec<(ChessInstant, i32)> {
    return minimax(mm, bot, ci);
}

fn minimax(mm: &MoveMap, bot: &BotSettings, root: &ChessInstant) -> Vec<(ChessInstant, i32)> {
    let l1_moves = root.valid_games(mm);
    let mut graded = Vec::new();
    for l1ci in l1_moves {
        graded.push((
            l1ci.clone(),
            minmax_sub(&l1ci, mm, bot, bot.search_depth, false, i32::MIN, i32::MAX),
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
    ) -> i32 {
        if depth == 0 {
            // the heuristic value of the game
            if maximizing {
                return static_score(ci, mm, bot);
            } else {
                // multiply by -1 so minimizing will get the best score for the opponent
                // and maximizing will get the worst move for self
                return static_score(ci, mm, bot) * -1;
            }
        }

        let ci_children = ci.valid_games(mm);
        if ci_children.len() == 0 && !ci.in_check(mm, &ci.king_id()){
            return 0; // score for a draw 
        }
        if maximizing {
            let mut value = i32::MIN; // + bot.search_depth - depth;
            for child in ci_children {
                let score = minmax_sub(&child, mm, bot, depth - 1, false, alpha, beta);
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
                let score = minmax_sub(&child, mm, bot, depth - 1, true, alpha, beta);
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

/// generates a score for the current ci player based on a static analasis
fn static_score(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings) -> i32 {
    let f_range = ci.player_range();

    material(ci, mm, bot, &f_range)
        + check(ci, mm, bot, &f_range)
        + position(ci, mm, bot, &f_range)
        + attacked(ci, mm, bot, &f_range)
}

/// material value of the board
fn material(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings, f_range: &Range<u32>) -> i32 {
    let mut pieces_value = 0;

    for (row, col) in mm.locations() {
        let piece = ci.board_get(row, col);
        if piece == 6 {
            continue;
        }
        let v = Piece::piece_value_dep(&piece, &bot);
        if f_range.contains(&piece) {
            pieces_value += v;
        } else {
            pieces_value -= v;
        }
    }

    pieces_value
}

/// returns check value if the opponent is in check else return 0
fn check(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings, f_range: &Range<u32>) -> i32 {
    let opp_id = if f_range.contains(&4) { 11 } else { 4 };

    if ci.in_check(mm, &opp_id) {
        return bot.check_value;
    }

    0
}

/// score for the players position pieces on the center being better
fn position(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings, f_range: &Range<u32>) -> i32 {
    let mut position = 0;
    for (row, col) in mm.locations() {
        let piece = ci.board_get(row, col);
        if piece == 6 {
            continue;
        }
        // r c will give value 0-3 depening on how far from the center the location is
        let r = 3.5 - (*row as f64 - 3.5).abs();
        let c = 3.5 - (*col as f64 - 3.5).abs();
        if f_range.contains(&piece) {
            position += (r + c) as i32; // between 0-6 depending on how close to center
        }
    }

    position * bot.position_mult
}

/// gives a negetive score of the piece values that are under attack
fn attacked(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings, f_range: &Range<u32>) -> i32 {
    let mut danger = 0;
    for (row, col) in mm.locations() {
        let piece = ci.board_get(row, col);
        if piece == 6 {
            continue;
        }
        if f_range.contains(&piece) && ci.threatened_dep(mm, row, col, &piece) {
            danger -= Piece::piece_value_dep(&piece, bot);
        }
    }

    danger / bot.attack_div
}

impl Piece {
    fn piece_value_dep(id: &u32, bot: &BotSettings) -> i32 {
        match id {
            0 | 7 => bot.rook_value,
            1 | 8 => bot.knight_value,
            2 | 9 => bot.bishop_value,
            3 | 10 => bot.queen_value,
            4 | 11 => bot.king_value,
            5 | 12 => bot.pawn_value,
            _ => 0,
        }
    }
}

impl ChessInstant {
    fn threatened_dep(&self, mm: &MoveMap, row: &usize, col: &u32, id: &u32) -> bool {
        if *id == 6 || *id > 12 {
            return false;
        }

        let movements = mm.king_cover(&id, row, col);
        'paths: for path in &movements.cover {
            for (r, c, ids) in path {
                let f = self.board_get(r, c);
                if f == 6 {
                    continue;
                }
                if !self.is_friendly(&id, &f) && ids.contains(&f) {
                    return true;
                }
                continue 'paths;
            }
        }

        false
    }
}
