pub mod bot_0;
pub mod bot_1;
pub mod chess;
pub mod heuristic;
pub mod minmax;
pub mod movemap;

use crate::chess::{ChessInstant, Piece, Player};

use movemap::MoveMap;

pub fn new_chess_instant() -> ChessInstant {
    ChessInstant::new()
}

pub fn new_movement_map() -> MoveMap {
    MoveMap::new()
}

pub fn valid_games(chess_instant: &mut ChessInstant, move_map: &MoveMap) -> Vec<ChessInstant> {
    chess_instant.valid_games(move_map)
}

pub fn decode_board(chess_instant: &ChessInstant) -> Vec<Vec<Option<(Player, Piece)>>> {
    chess_instant.decode_board()
}

pub fn game_from_json_unchecked(game_json: &String) -> ChessInstant {
    serde_json::from_str(&game_json).unwrap()
}

pub fn heuristic_v1_moves(chess_instant: &ChessInstant) -> Vec<(ChessInstant, i32)> {
    let mm = MoveMap::new();
    let depth = heuristic::heuristic_v1::dyn_depth(chess_instant, &mm);
    let bot = heuristic::heuristic_v1::heuristic_v1_bot(depth);
    let hf = heuristic::heuristic_v1::heuristic_fn;
    let graded_moves = minmax::minimax(&mm, &bot, chess_instant, hf);
    graded_moves
}

#[cfg(test)]
mod tests {
    // run to print statements: cargo test -- --nocapture
    //use crate::chess::ChessInstant;
    use crate::{decode_board, new_chess_instant, new_movement_map};

    #[test]
    fn decode() {
        let game = new_chess_instant();
        let decoded = decode_board(&game);
        assert_eq!(8, decoded.len());
        assert_eq!(8, decoded[0].len());
    }

    #[test]
    fn six_plie() {
        let time = std::time::Instant::now();
        let game = new_chess_instant();
        let mm = new_movement_map();
        let mut games = Vec::with_capacity(125_000_000);
        games.push(game);
        for i in 1..=6 {
            let mut checkmates = 0;
            let mut found = Vec::with_capacity(i * 21_000_000);
            for g in &mut games {
                let vg = g.valid_games(&mm);
                if vg.len() == 0 {
                    checkmates += 1;
                } else {
                    found.extend(vg);
                }
            }
            games = found;
            let shannon = match i {
                1 => 20,
                2 => 400,
                3 => 8_902,
                4 => 197_281,
                5 => 4_865_609,
                6 => 119_060_324,
                _ => 0,
            };
            assert_eq!(games.len(), shannon);

            let check = match i {
                1 => 0,
                2 => 0,
                3 => 0,
                4 => 0,
                5 => 8,
                6 => 347,
                _ => 0,
            };

            assert_eq!(check, checkmates);
        }
        println!(
            "six plie competed in: {} milliseconds.",
            time.elapsed().as_millis()
        );
    }
}
