// this module is for finding all valid moves that could be made

use crate::movemap::{MoveMap, MovePaths};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChessInstant {
    board: [u32; 8],
    prv_move: (u8, u8),
    plie: u16,
    p1_king: u8,
    p2_king: u8,
    p1_passant: u8,
    p2_passant: u8,
    valid_castles: (bool, bool, bool, bool),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Player {
    P1,
    P2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Piece {
    Pawn,
    Rook,
    Knight,
    Bishop,
    Queen,
    King,
}

// values for pieces are such
// player 1: 0-5
// no piece: 6
// player 2: 7-12
// unused: 13-15
// piece order None 6, rook 0/7 , knight 1/8, bishop 2/9, queen 3/10, king 4/11, pawn 5/12

impl ChessInstant {
    /// gives back the range of friendly piece ids for given plie
    pub fn player_range(&self) -> std::ops::Range<u32> {
        return if self.plie % 2 == 0 { 7..13 } else { 0..6 };
    }

    /// rutens the correct id for the current king
    pub fn king_id(&self) -> u32 {
        if self.plie % 2 == 0 {
            11
        } else {
            4
        }
    }

    /// gives back the player based on the turn number
    pub fn player(&self) -> Player {
        if self.plie % 2 == 0 {
            Player::P2
        } else {
            Player::P1
        }
    }

    /// determimes if the current player is in check, pice id is any friendly id of the king you are checking for ckeck
    pub fn in_check(&self, mm: &MoveMap, piece_id: &u32) -> bool {
        let (kr, kc) = if *piece_id < 6 {
            ChessInstant::decode_index(&self.p1_king)
        } else {
            ChessInstant::decode_index(&self.p2_king)
        };
        let kcover = mm.king_cover(piece_id, &kr, &kc);

        'paths: for path in &kcover.cover {
            for (r, c, atk) in path {
                let fpiece = self.board_get(r, c);
                if self.is_friendly(piece_id, &fpiece) {
                    continue 'paths;
                }
                if atk.contains(&fpiece) {
                    return true;
                }
                if fpiece != 6 {
                    continue 'paths;
                }
            }
        }
        false
    }

    /// DO NOT USE TO APART FROM ANALASIS. adds one to the plie will mess up and game you try and play after
    pub fn add_plie(&mut self) {
        self.plie += 1;
    }

    /// compair against another piece and determine if they are friendly with eachother
    pub fn is_friendly(&self, piece_id: &u32, test_piece_id: &u32) -> bool {
        //range.contains(test_piece_id)
        if *piece_id < 6 && *test_piece_id < 6 {
            return true;
        } else if *piece_id > 6 && *test_piece_id > 6 {
            return true;
        }
        false
    }

    /// every friendly piece that is covering the piece at the given location
    pub fn protected_by(&self, row: &usize, col: &u32, mm: &MoveMap) -> Vec<u32> {
        let piece = self.board_get(row, col);
        if piece == 6 || piece > 12 {
            return vec![];
        }
        let tmp_id = if piece < 6 { piece + 7 } else { piece - 7 };

        let movements = mm.king_cover(&tmp_id, row, col);
        let mut friends = Vec::new();
        'paths: for path in &movements.cover {
            for (r, c, ids) in path {
                let f = self.board_get(r, c);
                if f == 6 {
                    continue;
                }
                if self.is_friendly(&piece, &f) && ids.contains(&f) {
                    friends.push(f);
                }
                continue 'paths;
            }
        }

        friends
    }
    /// every opponent piece (id) that could take the piece at given row and column
    pub fn threatened_by(&self, row: &usize, col: &u32, mm: &MoveMap) -> Vec<u32> {
        let piece = self.board_get(row, col);
        if piece == 6 || piece > 12 {
            return vec![];
        }
        let movements = mm.king_cover(&piece, row, col);
        let mut threats = Vec::new();
        'paths: for path in &movements.cover {
            for (r, c, ids) in path {
                let f = self.board_get(r, c);
                if f == 6 {
                    continue;
                }
                if !self.is_friendly(&piece, &f) && ids.contains(&f) {
                    threats.push(f);
                }
                continue 'paths;
            }
        }

        threats
    }
    // every opponent id that is in the path blocked or not of the piece at the given location
    pub fn in_path(&self, row: &usize, col: &u32, mm: &MoveMap) -> Vec<u32> {
        let piece = self.board_get(row, col);
        let pawn;
        let movements = match piece {
            0 | 7 => &mm.get_rook_moves(row, col).paths,
            1 | 8 => &mm.get_knight_moves(row, col).paths,
            2 | 9 => &mm.get_bishop_moves(row, col).paths,
            3 | 10 => &mm.get_queen_moves(row, col).paths,
            4 | 11 => &mm.get_king_moves(row, col).paths,
            5 | 12 => {
                pawn = mm
                    .get_pawn_moves(row, col, &piece)
                    .atk
                    .iter()
                    .map(|p| vec![p.clone()])
                    .collect();
                &pawn
            }
            _ => return vec![],
        };
        let mut found = Vec::new();
        for path in movements {
            for (r, c) in path {
                let loc = self.board_get(r, c);
                if loc == 6 {
                    continue;
                }
                if !self.is_friendly(&piece, &loc) {
                    found.push(loc);
                }
            }
        }
        found
    }
    /// gives back a list of all the piece ids that the piece at that location can hit
    pub fn threatens(&self, row: &usize, col: &u32, mm: &MoveMap) -> Vec<u32> {
        let piece = self.board_get(row, col);
        let pawn;
        let movements = match piece {
            0 | 7 => &mm.get_rook_moves(row, col).paths,
            1 | 8 => &mm.get_knight_moves(row, col).paths,
            2 | 9 => &mm.get_bishop_moves(row, col).paths,
            3 | 10 => &mm.get_queen_moves(row, col).paths,
            4 | 11 => &mm.get_king_moves(row, col).paths,
            5 | 12 => {
                pawn = mm
                    .get_pawn_moves(row, col, &piece)
                    .atk
                    .iter()
                    .map(|p| vec![p.clone()])
                    .collect();
                &pawn
            }
            _ => return vec![],
        };
        let mut found = Vec::new();
        'path: for path in movements {
            for (r, c) in path {
                let loc = self.board_get(r, c);
                if loc == 6 {
                    continue;
                }
                if !self.is_friendly(&piece, &loc) {
                    found.push(loc);
                }
                continue 'path;
            }
        }
        found
    }
    /// returns the option for the player and piece for given row and col
    pub fn player_piece(&self, row: &usize, col: &u32) -> Option<(Player, Piece)> {
        let piece_id = self.board_get(row, col);
        if piece_id == 6 {
            return None;
        }

        let player = if piece_id < 6 { Player::P1 } else { Player::P2 };

        let piece = Piece::from_id(&piece_id);

        Some((player, piece))
    }
    /// decodes the board into a more readable state
    pub fn decode_board(&self) -> Vec<Vec<Option<(Player, Piece)>>> {
        let mut board = Vec::new();

        for r in 0..8 {
            let mut row = Vec::new();
            for c in 0..8 {
                row.push(self.player_piece(&r, &c));
            }
            board.push(row);
        }

        board
    }

    /// returns all valid chess instants that can result from self
    pub fn valid_games(&self, mm: &MoveMap) -> Vec<ChessInstant> {
        let mut ci = *self;
        ci.reset_en_passant();
        let mut found_valid: Vec<ChessInstant> = Vec::with_capacity(50);
        let player_range = self.player_range();
        for (row, col) in mm.locations() {
            let piece = self.board_get(&row, &col);
            if !player_range.contains(&piece) {
                continue;
            }
            match piece {
                0 | 7 => {
                    ci.rook_movement(&mut found_valid, &row, &col, mm, &piece);
                    ci.castling_movement(&mut found_valid, &row, &col, mm, &piece);
                }
                1 | 8 => ci.standard_movement(
                    mm.get_knight_moves(&row, &col),
                    &mut found_valid,
                    &row,
                    &col,
                    mm,
                    &piece,
                ),
                2 | 9 => ci.standard_movement(
                    mm.get_bishop_moves(&row, &col),
                    &mut found_valid,
                    &row,
                    &col,
                    mm,
                    &piece,
                ),
                3 | 10 => ci.standard_movement(
                    mm.get_queen_moves(&row, &col),
                    &mut found_valid,
                    &row,
                    &col,
                    mm,
                    &piece,
                ),
                4 | 11 => ci.king_movement(&mut found_valid, &row, &col, mm, &piece),
                5 | 12 => ci.pawn_movement(&mut found_valid, &row, &col, mm, &piece), // pawn movement,
                _ => continue,
            }
        }
        found_valid
    }

    /// check validity and add a castling move
    fn castling_movement(
        &self,
        valid: &mut Vec<ChessInstant>,
        row: &usize,
        col: &u32,
        mm: &MoveMap,
        piece_id: &u32,
    ) {
        // check if valid to make castle move
        if *piece_id < 6 && !self.valid_castles.0 && !self.valid_castles.1 {
            return;
        }
        if *piece_id > 6 && !self.valid_castles.2 && !self.valid_castles.3 {
            return;
        }
        if self.in_check(mm, piece_id) {
            return;
        }

        // player 1 left
        if *row == 7 && *col == 0 {
            let (krow, kcol) = ChessInstant::decode_index(&self.p1_king);
            let movement = vec![(7, 3), (7, 2)];
            make_castle_move(valid, *self, mm, &movement, &krow, &kcol, &4, row, col, 0);
        }
        // player 1 right
        if *row == 7 && *col == 7 {
            let (krow, kcol) = ChessInstant::decode_index(&self.p1_king);
            let movement = vec![(7, 5), (7, 6)];
            make_castle_move(valid, *self, mm, &movement, &krow, &kcol, &4, row, col, 0);
        }

        // player 2 left
        if *row == 0 && *col == 0 {
            let (krow, kcol) = ChessInstant::decode_index(&self.p2_king);
            let movement = vec![(0, 3), (0, 2)];
            make_castle_move(valid, *self, mm, &movement, &krow, &kcol, &11, row, col, 7);
        }
        // player 2 right
        if *row == 0 && *col == 7 {
            let (krow, kcol) = ChessInstant::decode_index(&self.p2_king);
            let movement = vec![(0, 5), (0, 6)];
            make_castle_move(valid, *self, mm, &movement, &krow, &kcol, &11, row, col, 7);
        }

        fn make_castle_move(
            valid: &mut Vec<ChessInstant>,
            mut game: ChessInstant,
            mm: &MoveMap,
            movement: &Vec<(usize, u32)>,
            row: &usize,
            col: &u32,
            piece_id: &u32,
            rookr: &usize,
            rookc: &u32,
            rookid: u32,
        ) {
            for (r, c) in movement {
                if game.board_get(r, c) != 6 {
                    return; // if any movespace is occupied not valid castle
                }
                // check the left castle extra space
                if *c == 2 {
                    if game.board_get(row, &1) != 6 {
                        return;
                    }
                }
            }

            let (mut prvr, mut prvc) = (*row, *col);
            for (r, c) in movement {
                // move the king
                game.board_set(&prvr, &prvc, 6);
                game.board_set(&r, &c, *piece_id);
                // move king location
                if *piece_id < 6 {
                    game.p1_king = ChessInstant::encode_index(r, c);
                } else {
                    game.p2_king = ChessInstant::encode_index(r, c);
                }
                // return if in check at any point
                if game.in_check(mm, piece_id) {
                    return;
                }
                // update prevous row and col
                (prvr, prvc) = (*r, *c);
            }

            if *piece_id < 6 {
                game.valid_castles.0 = false;
                game.valid_castles.1 = false;
            } else {
                game.valid_castles.2 = false;
                game.valid_castles.3 = false;
            }

            // move the rook
            game.board_set(rookr, rookc, 6);
            game.board_set(&movement[0].0, &movement[0].1, rookid);
            game.prv_move = (
                ChessInstant::encode_index(row, col),
                ChessInstant::encode_index(&prvr, &prvc),
            );
            game.plie += 1;
            // add the valid castle move
            valid.push(game);
        }
    }

    /// handle movement for rook, knight, bishop,queen, and king
    fn standard_movement(
        &self,
        movements: &MovePaths,
        valid: &mut Vec<ChessInstant>,
        row: &usize,
        col: &u32,
        mm: &MoveMap,
        piece_id: &u32,
    ) {
        //let movements = mm.get_standard_moves(row, col, piece_id);
        'paths: for path in &movements.paths {
            for (tr, tc) in path {
                let found = self.board_get(tr, tc);
                if self.is_friendly(piece_id, &found) {
                    continue 'paths;
                }
                self.make_move_standard(mm, valid, piece_id, row, col, tr, tc);
                if found != 6 {
                    continue 'paths;
                }
            }
        }
    }

    /// handle movement for the king
    fn king_movement(
        &self,
        valid: &mut Vec<ChessInstant>,
        row: &usize,
        col: &u32,
        mm: &MoveMap,
        piece_id: &u32,
    ) {
        let movements = mm.get_king_moves(row, col);
        'paths: for path in &movements.paths {
            for (tr, tc) in path {
                let found = self.board_get(tr, tc);
                if self.is_friendly(piece_id, &found) {
                    continue 'paths;
                }
                self.make_move_king(mm, valid, piece_id, row, col, tr, tc);
                if found != 6 {
                    continue 'paths;
                }
            }
        }
    }

    /// handle movement for the rook
    fn rook_movement(
        &self,
        valid: &mut Vec<ChessInstant>,
        row: &usize,
        col: &u32,
        mm: &MoveMap,
        piece_id: &u32,
    ) {
        let movements = mm.get_rook_moves(row, col);
        'paths: for path in &movements.paths {
            for (tr, tc) in path {
                let found = self.board_get(tr, tc);
                if self.is_friendly(piece_id, &found) {
                    continue 'paths;
                }
                self.make_move_rook(mm, valid, piece_id, row, col, tr, tc);
                if found != 6 {
                    continue 'paths;
                }
            }
        }
    }

    /// handle all pawn movement
    fn pawn_movement(
        &self,
        valid: &mut Vec<ChessInstant>,
        row: &usize,
        col: &u32,
        mm: &MoveMap,
        piece_id: &u32,
    ) {
        let movements = mm.get_pawn_moves(row, col, piece_id);
        // forwayd movement
        for (i, (r, c)) in movements.mov.iter().enumerate() {
            let found = self.board_get(r, c);
            if found != 6 {
                break; // can only move onto a blank space
            }
            if i == 1 {
                self.make_move_double_pawn(mm, valid, piece_id, row, col, r, c);
            } else {
                if *r == 0 || *r == 7 {
                    // if on end row promote that pawn!
                    let ids = if *piece_id < 6 {
                        vec![0, 1, 2, 3]
                    } else {
                        vec![7, 8, 9, 10]
                    };
                    for id in ids {
                        self.make_move_standard(mm, valid, &id, row, col, r, c);
                    }
                } else {
                    self.make_move_standard(mm, valid, piece_id, row, col, r, c);
                }
            }
        }
        // attacking movement
        for (r, c) in &movements.atk {
            let found = self.board_get(r, c);
            if found == 6 || self.is_friendly(piece_id, &found) {
                continue;
            }
            if *r == 0 || *r == 7 {
                // if on end row promote that pawn!
                let ids = if *piece_id < 6 {
                    vec![0, 1, 2, 3]
                } else {
                    vec![7, 8, 9, 10]
                };
                for id in ids {
                    self.make_move_standard(mm, valid, &id, row, col, r, c);
                }
            } else {
                self.make_move_standard(mm, valid, piece_id, row, col, r, c);
            }
        }
        // en passant
        let ep = if *piece_id < 6 {
            &self.p2_passant
        } else {
            &self.p1_passant
        };
        if *ep > 63 {
            return; // not a valid row and col
        }
        let (epr, epc) = ChessInstant::decode_index(ep);
        if movements.atk.contains(&(epr, epc)) {
            self.make_move_en_passant(mm, valid, piece_id, row, col, &epr, &epc);
        }
    }

    /// moves piece at from to to then checks if king is in check if not adds new game indtance to valid
    fn make_move_rook(
        &self,
        mm: &MoveMap,
        valid: &mut Vec<ChessInstant>,
        piece_id: &u32,
        from_row: &usize,
        from_col: &u32,
        to_row: &usize,
        to_col: &u32,
    ) {
        let mut clone = *self;
        clone.board_set(from_row, from_col, 6); // set from to blank value
        clone.board_set(to_row, to_col, *piece_id); // set to to piece id
        if clone.in_check(mm, piece_id) {
            return; // not valid if in check
        }
        if *from_row == 7 && *from_col == 0 {
            clone.valid_castles.0 = false;
        } else if *from_row == 7 && *from_col == 7 {
            clone.valid_castles.1 = false;
        } else if *from_row == 0 && *from_col == 0 {
            clone.valid_castles.2 = false;
        } else if *from_row == 0 && *from_col == 7 {
            clone.valid_castles.3 = false;
        }

        clone.plie += 1;
        clone.prv_move = (
            ChessInstant::encode_index(from_row, from_col),
            ChessInstant::encode_index(to_row, to_col),
        );
        valid.push(clone);
    }

    /// moves piece at from to to then checks if king is in check if not adds new game indtance to valid
    fn make_move_king(
        &self,
        mm: &MoveMap,
        valid: &mut Vec<ChessInstant>,
        piece_id: &u32,
        from_row: &usize,
        from_col: &u32,
        to_row: &usize,
        to_col: &u32,
    ) {
        let mut clone = *self;
        clone.board_set(from_row, from_col, 6); // set from to blank value
        clone.board_set(to_row, to_col, *piece_id); // set to to piece id
                                                    // move stored king location
        if *piece_id < 6 {
            clone.p1_king = ChessInstant::encode_index(to_row, to_col);
            clone.valid_castles.0 = false; // castling no longer valid for player 1
            clone.valid_castles.1 = false;
        } else {
            clone.p2_king = ChessInstant::encode_index(to_row, to_col);
            clone.valid_castles.2 = false; // castling no longer valid for player 2
            clone.valid_castles.3 = false;
        }
        if clone.in_check(mm, piece_id) {
            return; // not valid if in check
        }
        clone.plie += 1;
        clone.prv_move = (
            ChessInstant::encode_index(from_row, from_col),
            ChessInstant::encode_index(to_row, to_col),
        );
        valid.push(clone);
    }

    /// handles logic for the en passant attack
    fn make_move_en_passant(
        &self,
        mm: &MoveMap,
        valid: &mut Vec<ChessInstant>,
        piece_id: &u32,
        from_row: &usize,
        from_col: &u32,
        to_row: &usize,
        to_col: &u32,
    ) {
        let mut clone = *self;
        clone.board_set(from_row, from_col, 6); // set from to blank value
        clone.board_set(to_row, to_col, *piece_id); // set to to piece id
        clone.board_set(from_row, to_col, 6); // take the piece en passant
        if clone.in_check(mm, piece_id) {
            return; // not valid if in check
        }
        clone.plie += 1;
        clone.prv_move = (
            ChessInstant::encode_index(from_row, from_col),
            ChessInstant::encode_index(to_row, to_col),
        );
        valid.push(clone);
    }

    /// does the same as standard move but sets the en passant value
    fn make_move_double_pawn(
        &self,
        mm: &MoveMap,
        valid: &mut Vec<ChessInstant>,
        piece_id: &u32,
        from_row: &usize,
        from_col: &u32,
        to_row: &usize,
        to_col: &u32,
    ) {
        let mut clone = *self;
        clone.board_set(from_row, from_col, 6); // set from to blank value
        clone.board_set(to_row, to_col, *piece_id); // set to to piece id
        if clone.in_check(mm, piece_id) {
            return; // not valid if in check
        }
        clone.plie += 1;
        if *piece_id < 6 {
            clone.p1_passant = ChessInstant::encode_index(&(from_row - 1), from_col);
        } else {
            clone.p2_passant = ChessInstant::encode_index(&(from_row + 1), from_col);
        }
        clone.prv_move = (
            ChessInstant::encode_index(from_row, from_col),
            ChessInstant::encode_index(to_row, to_col),
        );
        valid.push(clone);
    }

    /// moves piece at from to to then checks if king is in check if not adds new game indtance to valid
    fn make_move_standard(
        &self,
        mm: &MoveMap,
        valid: &mut Vec<ChessInstant>,
        piece_id: &u32,
        from_row: &usize,
        from_col: &u32,
        to_row: &usize,
        to_col: &u32,
    ) {
        let mut clone = *self;
        clone.board_set(from_row, from_col, 6); // set from to blank value
        clone.board_set(to_row, to_col, *piece_id); // set to to piece id
        if clone.in_check(mm, piece_id) {
            return; // not valid if in check
        }
        clone.plie += 1;
        clone.prv_move = (
            ChessInstant::encode_index(from_row, from_col),
            ChessInstant::encode_index(to_row, to_col),
        );
        valid.push(clone);
    }

    /// resets the en passant value for the current player
    fn reset_en_passant(&mut self) {
        if self.plie % 2 == 0 {
            self.p2_passant = 64;
        } else {
            self.p1_passant = 64;
        }
    }

    /// encodes a row and column into a u8 index
    fn encode_index(row: &usize, col: &u32) -> u8 {
        (row << 3) as u8 + *col as u8 // row * 8 + column
    }

    /// gives back the (row, column) for a given index
    fn decode_index(index: &u8) -> (usize, u32) {
        ((index >> 3) as usize, (index % 8) as u32)
    }

    /// get the value at given row and column as a u32 btween 0-15,
    /// !!!unchecked bounds will panic if row or col is greater than 7!!!
    pub fn board_get(&self, row: &usize, col: &u32) -> u32 {
        // shift over the column to currect 8 bits, then exclude with the & the extra bits
        self.board[*row] >> (col << 2) & 0x0000000f
    }

    /// sets the value at given row and column val should be between 0-15
    fn board_set(&mut self, row: &usize, col: &u32, val: u32) {
        // filter is used to clear whatever was in the location already
        let filter = u32::MAX ^ (0x0000000f << (col << 2));
        // (set the valid 4 bits to 0) + (the new value being set)
        self.board[*row] = (self.board[*row] & filter) + (val << (col << 2));
    }

    /// creates a new game with plie at 1 and pieces in starting location
    pub fn new() -> ChessInstant {
        let mut c = ChessInstant {
            board: [0; 8],
            prv_move: (0, 0),
            plie: 1,
            p1_king: 60,
            p2_king: 4,
            p1_passant: 64,
            p2_passant: 64,
            valid_castles: (true, true, true, true),
        };
        // set all squares to none val
        for row in 0..8 {
            for col in 0..8 {
                c.board_set(&row, &col, 6);
            }
        }
        // piece order None 6, rook 0/7 , knight 1/8, bishop 2/9, queen 3/10, king 4/11, pawn 5/12
        // player 2
        for i in 0..8 {
            c.board_set(&1, &i, 12);
        }
        c.board_set(&0, &0, 7);
        c.board_set(&0, &1, 8);
        c.board_set(&0, &2, 9);
        c.board_set(&0, &3, 10);
        c.board_set(&0, &4, 11);
        c.board_set(&0, &5, 9);
        c.board_set(&0, &6, 8);
        c.board_set(&0, &7, 7);
        // player 1
        for i in 0..8 {
            c.board_set(&6, &i, 5);
        }
        c.board_set(&7, &0, 0);
        c.board_set(&7, &1, 1);
        c.board_set(&7, &2, 2);
        c.board_set(&7, &3, 3);
        c.board_set(&7, &4, 4);
        c.board_set(&7, &5, 2);
        c.board_set(&7, &6, 1);
        c.board_set(&7, &7, 0);

        c
    }

    /// prints the state to the console
    #[allow(unused)]
    pub fn print(&self) {
        println!();
        println!("Plie: {}, {}", self.plie, self.player().as_str());
        for row in 0..8 {
            for col in 0..8 {
                let str = match self.board_get(&row, &col) {
                    0 => "1RO",
                    1 => "1KN",
                    2 => "1BI",
                    3 => "1QU",
                    4 => "1KI",
                    5 => "1PA",
                    6 => "[ ]",
                    7 => "2RO",
                    8 => "2KN",
                    9 => "2BI",
                    10 => "2QU",
                    11 => "2KI",
                    12 => "2PA",
                    _ => "[!]",
                };
                print!("{str} ");
            }
            println!();
        }
    }
}

impl Player {
    fn as_str(&self) -> &'static str {
        match self {
            Player::P1 => "P1-White Moves",
            Player::P2 => "P2-Black Moves",
        }
    }

    pub fn swap(&self) -> Player {
        match self {
            Player::P1 => Player::P2,
            Player::P2 => Player::P1,
        }
    }
}

impl Piece {
    pub fn from_id(id: &u32) -> Piece {
        match id {
            0 | 7 => Piece::Rook,
            1 | 8 => Piece::Knight,
            2 | 9 => Piece::Bishop,
            3 | 10 => Piece::Queen,
            4 | 11 => Piece::King,
            _ => Piece::Pawn,
        }
    }
}
