pub mod heuristic_v1 {
    use crate::chess::{ChessInstant, Piece, Player};
    use crate::minmax::BotSettings;
    use crate::movemap::MoveMap;
    use std::ops::Range;

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

    /// gives back a suggested depth for the min max fn
    pub fn dyn_depth(ci: &ChessInstant, mm: &MoveMap) -> i32 {
        let mut piece_val = 0;
        for (row, col) in mm.locations() {
            let piece = ci.board_get(row, col);
            if piece == 6 {
                continue;
            }
            if piece == 5 || piece == 12 {
                piece_val += 1;
            } else {
                piece_val += 4;
            }
        }

        match piece_val {
            61..=80 => 4,
            41..=60 => 5,
            21..=40 => 5,
            11..=20 => 6,
            0..=10 => 7,
            _ => 4,
        }
    }

    pub fn heuristic_v1_bot(depth: i32) -> BotSettings {
        BotSettings {
            search_depth: depth,
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

    /// generates a score for the current ci player based on a static analasis
    pub fn heuristic_fn(ci: &ChessInstant, mm: &MoveMap, bot: &BotSettings) -> i32 {
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
            let v = Piece::piece_value(&piece, &bot);
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
            if f_range.contains(&piece) && ci.threatened(mm, row, col, &piece) {
                danger -= Piece::piece_value(&piece, bot);
            }
        }

        danger / bot.attack_div
    }

    impl Piece {
        fn piece_value(id: &u32, bot: &BotSettings) -> i32 {
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
        fn threatened(&self, mm: &MoveMap, row: &usize, col: &u32, id: &u32) -> bool {
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
}
