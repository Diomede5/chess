// first implementation of a bot only gades the piceses and looks at a single plie
use serde::{Deserialize, Serialize};



use crate::{
    chess::{ChessInstant, Piece, Player},
    movemap::MoveMap,
};

impl Piece {
    fn value(&self) -> i32 {
        match self {
            Piece::Rook => 500,
            Piece::Knight => 300,
            Piece::Bishop => 300,
            Piece::Queen => 900,
            Piece::King => 0,
            Piece::Pawn => 100,
        }
    }
}

pub fn bot_0_1_moves(ci: &mut ChessInstant, mm: &MoveMap) -> Vec<(ChessInstant, i32)> {
    let valid_moves = ci.valid_games(mm);
    let player = ci.player();
    let mut graded: Vec<(ChessInstant, i32)> = valid_moves
        .iter()
        .map(|vci| (vci.clone(), grade(vci, &player)))
        .collect();

    graded.sort_unstable_by(|(_, a), (_, b)| b.cmp(&a));

    graded
}

pub fn bot_0_2_moves(ci: &mut ChessInstant, mm: &MoveMap) -> Vec<(ChessInstant, i32)> {
    let mut valid_moves = ci.valid_games(mm);
    let player = ci.player();
    let depth: u32 = 25;

    let mut graded: Vec<(ChessInstant, i32)> = valid_moves
        .iter_mut()
        .map(|mut vci| (vci.clone(), eval_plie(&mut vci, mm, &player, depth)))
        .collect();

    graded.sort_unstable_by(|(_, a), (_, b)| b.cmp(&a));

    graded
}

fn eval_plie(ci: &mut ChessInstant, mm: &MoveMap, player: &Player, depth: u32) -> i32 {
    let mut found = ci.valid_games(mm);

    if found.len() == 0 {
        if ci.player() == *player {
            return i32::MIN; // being in check mate is as bad as it gets
        } else {
            return i32::MAX; // putting opponent in checkmate is as good as it gets
        }
    }

    let mut graded: Vec<(ChessInstant, i32)> = found
        .iter_mut()
        .map(|vci| (vci.clone(), gradev2(vci, &ci.player(), mm)))
        .collect();

    graded.sort_unstable_by(|(_, a), (_, b)| b.cmp(&a));

    if depth == 0 {
        return graded[0].1;
    } else if ci.player() == *player {
        return eval_plie(&mut graded[0].0, mm, player, depth);
    } else {
        return eval_plie(&mut graded[0].0, mm, player, depth - 1);
    }
}

fn grade(ci: &ChessInstant, player: &Player) -> i32 {
    let mut grade = 0;
    let decoded = ci.decode_board();
    for row in &decoded {
        for op in row {
            if let Some((piece_player, piece)) = op {
                if *player == *piece_player {
                    grade += piece.value();
                } else {
                    grade -= piece.value();
                }
            }
        }
    }

    grade
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BotSettings {
    pawn_value: i32,
    rook_value: i32,
    knight_value: i32,
    bishop_value: i32,
    queen_value: i32,
    king_value: i32,
    search_depth: i32,
    piece_value_multiplier: i32,
    opponent_moves_multiplier: i32,
    trade_multiplier: i32,
    threat_multiplier: i32,
}

fn gradev2(ci: &mut ChessInstant, player: &Player, mm: &MoveMap) -> i32 {
    let opponent_moves_multiplier = 5;
    let piece_value_multiplier = 10;
    let trade_multiplier = 10;
    let threat_multiplier = 1;

    let mut pieces_value = 0;
    let decoded = ci.decode_board();
    for row in &decoded {
        for op in row {
            if let Some((piece_player, piece)) = op {
                if *player == *piece_player {
                    pieces_value += piece.value();
                } else {
                    pieces_value -= piece.value();
                }
            }
        }
    }

    let opponent_moves_grade = ci.valid_games(mm).len() as i32 * (-1);

    let mut threat = 0;
    let mut trade_potential = 0;
    for row in 0..7 {
        for col in 0..7 {
            let p = decoded[row as usize][col as usize];
            if None == p {
                continue;
            }

            let (found_player, found_piece) = p.unwrap();
            if found_player == *player {
                // calculate trade potential
                let threats: Vec<Piece> = ci.threatened_by(&row, &col, mm).iter().map(|id| Piece::from_id(id)).collect();
                if threats.len() > 0 {
                    let covered: Vec<Piece> = ci.protected_by(&row, &col, mm).iter().map(|id| Piece::from_id(id)).collect();
                    if covered.len() > 0 {
                        let mut min_threat = threats[0].value();
                        threats.iter().for_each(|t| {
                            if t.value() < min_threat {
                                min_threat = t.value()
                            }
                        });
                        trade_potential = min_threat - found_piece.value();
                    }
                    // if not covered by piece
                    else {
                        trade_potential = found_piece.value() * (-1);
                    }
                }

                // calculate the max threat
                let threatens: Vec<Piece> = ci.threatens(&row, &col, mm).iter().map(|id| Piece::from_id(id)).collect();
                if threatens.len() > 0 {
                    let mut max_threat = threatens[0].value();
                    threatens.iter().for_each(|t| {
                        if t.value() > max_threat {
                            max_threat = t.value();
                        }
                    });
                    threat = max_threat;
                }
            }
        }
    }

    (pieces_value * piece_value_multiplier)
        + (opponent_moves_grade * opponent_moves_multiplier)
        + (trade_potential * trade_multiplier)
        + (threat * threat_multiplier)
}
