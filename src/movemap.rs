pub struct MoveMap {
    row_col: Vec<(usize, u32)>,
    rook: Vec<Vec<MovePaths>>,
    knight: Vec<Vec<MovePaths>>,
    bishop: Vec<Vec<MovePaths>>,
    queen: Vec<Vec<MovePaths>>,
    king: Vec<Vec<MovePaths>>,
    p1_pawn: Vec<Vec<PawnMoveMap>>,
    p2_pawn: Vec<Vec<PawnMoveMap>>,
    p1_check_king: Vec<Vec<KingCover>>,
    p2_check_king: Vec<Vec<KingCover>>,
}

pub struct KingCover {
    pub cover: Vec<Vec<(usize, u32, Vec<u32>)>>,
}

pub struct PawnMoveMap {
    pub mov: Vec<(usize, u32)>,
    pub atk: Vec<(usize, u32)>,
}

pub struct MovePaths {
    pub paths: Vec<Vec<(usize, u32)>>,
}

impl MoveMap {
    pub fn locations(&self) -> &Vec<(usize, u32)> {
        &self.row_col
    }
    pub fn get_king_moves(&self, row: &usize, col: &u32) -> &MovePaths {
        &self.king[*row][*col as usize]
    }
    pub fn get_knight_moves(&self, row: &usize, col: &u32) -> &MovePaths {
        &self.knight[*row][*col as usize]
    }
    pub fn get_queen_moves(&self, row: &usize, col: &u32) -> &MovePaths {
        &self.queen[*row][*col as usize]
    }
    pub fn get_bishop_moves(&self, row: &usize, col: &u32) -> &MovePaths {
        &self.bishop[*row][*col as usize]
    }
    pub fn get_rook_moves(&self, row: &usize, col: &u32) -> &MovePaths {
        &self.rook[*row][*col as usize]
    }
    pub fn get_pawn_moves(&self, row: &usize, col: &u32, piece_id: &u32) -> &PawnMoveMap {
        if *piece_id < 6 {
            &self.p1_pawn[*row][*col as usize]
        } else {
            &self.p2_pawn[*row][*col as usize]
        }
    }
    /// returns all the ways the piece at the row and col could be attacked
    pub fn king_cover(&self, piece_id: &u32, row: &usize, col: &u32) -> &KingCover {
        if *piece_id < 6 {
            &self.p1_check_king[*row][*col as usize]
        } else {
            &self.p2_check_king[*row][*col as usize]
        }
    }
    pub fn new() -> MoveMap {
        MoveMap {
            row_col: MoveMap::row_col(),
            rook: MoveMap::rook(),
            knight: MoveMap::knight(),
            bishop: MoveMap::bishop(),
            queen: MoveMap::queen(),
            king: MoveMap::king(),
            p1_pawn: MoveMap::p1_pawn(),
            p2_pawn: MoveMap::p2_pawn(),
            p1_check_king: MoveMap::p1_check_king(),
            p2_check_king: MoveMap::p2_check_king(),
        }
    }
    fn row_col() -> Vec<(usize, u32)> {
        let mut rc = Vec::new();
        for r in 0..8 {
            for c in 0..8 {
                rc.push((r,c));
            }
        }
        rc
    }
    fn p1_check_king() -> Vec<Vec<KingCover>> {
        let mut ck = Vec::with_capacity(8);

        let knight = MoveMap::knight();
        let bishop = MoveMap::bishop();
        let rook = MoveMap::rook();
        // p2 piece numbers: 7-rook 8-knight 9-bishop 10-queen 11-king 12-pawn
        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut cover: Vec<Vec<(usize, u32, Vec<u32>)>> = Vec::new();
                for p in &knight[row][col].paths {
                    cover.push(p.iter().map(|(r, c)| (*r, *c, vec![8])).collect())
                    // knight atk
                    // 8 p2 knight
                }
                for p in &bishop[row][col].paths {
                    let mut cv: Vec<(usize, u32, Vec<u32>)> =
                        p.iter().map(|(r, c)| (*r, *c, vec![9, 10])).collect(); // bishop and queen atk
                    cv[0].2.push(11); // king atk
                    if cv[0].0 < row {
                        cv[0].2.push(12); // pawn atk
                    }
                    cover.push(cv);
                }
                for p in &rook[row][col].paths {
                    let mut cv: Vec<(usize, u32, Vec<u32>)> =
                        p.iter().map(|(r, c)| (*r, *c, vec![7, 10])).collect(); // rook and queen atk
                    cv[0].2.push(11); // king atk
                    cover.push(cv);
                }
                // sort so attacks from the front come first
                cover.sort_by(|a, b| a[0].0.cmp(&b[0].0));
                r.push(KingCover { cover });
            }
            ck.push(r);
        }

        ck
    }
    fn p2_check_king() -> Vec<Vec<KingCover>> {
        let mut ck = Vec::with_capacity(8);

        let knight = MoveMap::knight();
        let bishop = MoveMap::bishop();
        let rook = MoveMap::rook();
        // p1 piece numbers: 0-rook 1-knight 2-bishop 3-queen 4-king 5-pawn
        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut cover: Vec<Vec<(usize, u32, Vec<u32>)>> = Vec::new();
                for p in &knight[row][col].paths {
                    cover.push(p.iter().map(|(r, c)| (*r, *c, vec![1])).collect())
                    // knight atk
                    // 8 p2 knight
                }
                for p in &bishop[row][col].paths {
                    let mut cv: Vec<(usize, u32, Vec<u32>)> =
                        p.iter().map(|(r, c)| (*r, *c, vec![2, 3])).collect(); // bishop and queen atk
                    cv[0].2.push(4); // king atk
                    if cv[0].0 > row {
                        cv[0].2.push(5); // pawn atk
                    }
                    cover.push(cv);
                }
                for p in &rook[row][col].paths {
                    let mut cv: Vec<(usize, u32, Vec<u32>)> =
                        p.iter().map(|(r, c)| (*r, *c, vec![0, 3])).collect(); // rook and queen atk
                    cv[0].2.push(4); // king atk
                    cover.push(cv);
                }
                // sort so attacks from the front come first
                cover.sort_by(|a, b| b[0].0.cmp(&a[0].0));
                r.push(KingCover { cover });
            }
            ck.push(r);
        }

        ck
    }
    fn knight() -> Vec<Vec<MovePaths>> {
        let mut m = Vec::with_capacity(8);

        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut paths = Vec::new();

                if row > 1 && col > 0 {
                    paths.push(vec![(row - 2, col - 1)]);
                }
                if row > 1 && col < 7 {
                    paths.push(vec![(row - 2, col + 1)]);
                }

                if row > 0 && col > 1 {
                    paths.push(vec![(row - 1, col - 2)]);
                }
                if row > 0 && col < 6 {
                    paths.push(vec![(row - 1, col + 2)]);
                }

                if row < 7 && col > 1 {
                    paths.push(vec![(row + 1, col - 2)]);
                }
                if row < 7 && col < 6 {
                    paths.push(vec![(row + 1, col + 2)]);
                }

                if row < 6 && col > 0 {
                    paths.push(vec![(row + 2, col - 1)]);
                }
                if row < 6 && col < 7 {
                    paths.push(vec![(row + 2, col + 1)]);
                }

                r.push(MovePaths { paths })
            }
            m.push(r);
        }

        m
    }
    fn king() -> Vec<Vec<MovePaths>> {
        let mut m = Vec::with_capacity(8);
        let queen = MoveMap::queen();

        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut paths = Vec::new();
                for p in &queen[row][col].paths {
                    paths.push(vec![p[0].clone()]);
                }
                r.push(MovePaths { paths });
            }
            m.push(r);
        }

        m
    }
    fn queen() -> Vec<Vec<MovePaths>> {
        let bishop = MoveMap::bishop();
        let rook = MoveMap::rook();

        let mut m = Vec::new();
        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut paths = Vec::new();
                paths.extend(bishop[row][col].paths.clone());
                paths.extend(rook[row][col].paths.clone());
                r.push(MovePaths { paths });
            }
            m.push(r);
        }

        m
    }
    fn bishop() -> Vec<Vec<MovePaths>> {
        let mut m = Vec::with_capacity(8);
        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut paths = Vec::new();
                if row > 0 && col > 0 {
                    let mut path = Vec::new();
                    let range = std::cmp::min(row, col);
                    for i in 1..=range {
                        path.push(((row - i) as usize, (col - i) as u32));
                    }
                    paths.push(path);
                }
                if row > 0 && col < 7 {
                    let mut path = Vec::new();
                    let range = std::cmp::min(row, 7 - col);
                    for i in 1..=range {
                        path.push(((row - i) as usize, (col + i) as u32));
                    }
                    paths.push(path);
                }
                if row < 7 && col < 7 {
                    let mut path = Vec::new();
                    let range = std::cmp::min(7 - row, 7 - col);
                    for i in 1..=range {
                        path.push(((row + i) as usize, (col + i) as u32));
                    }
                    paths.push(path);
                }
                if row < 7 && col > 0 {
                    let mut path = Vec::new();
                    let range = std::cmp::min(7 - row, col);
                    for i in 1..=range {
                        path.push(((row + i) as usize, (col - i) as u32));
                    }
                    paths.push(path);
                }
                r.push(MovePaths { paths })
            }
            m.push(r);
        }
        m
    }
    fn rook() -> Vec<Vec<MovePaths>> {
        let mut m = Vec::with_capacity(8);
        for row in 0..8 {
            let mut r = Vec::with_capacity(8);
            for col in 0..8 {
                let mut paths = Vec::new();
                if row > 0 {
                    let mut path = Vec::new();
                    for i in 1..=row {
                        path.push((row - i, col));
                    }
                    paths.push(path);
                }
                if row < 7 {
                    let mut path = Vec::new();
                    for i in 1..=7 - row {
                        path.push((row + i, col));
                    }
                    paths.push(path);
                }
                if col > 0 {
                    let mut path = Vec::new();
                    for i in 1..=col {
                        path.push((row, col - i));
                    }
                    paths.push(path);
                }
                if col < 7 {
                    let mut path = Vec::new();
                    for i in 1..=7 - col {
                        path.push((row, col + i));
                    }
                    paths.push(path);
                }
                r.push(MovePaths { paths })
            }
            m.push(r);
        }
        m
    }
    fn p1_pawn() -> Vec<Vec<PawnMoveMap>> {
        let mut mm = Vec::with_capacity(8);
        for row in 0..8 {
            let mut rowv = Vec::with_capacity(8);
            for col in 0..8 {
                rowv.push(PawnMoveMap::new_p1(row, col));
            }
            mm.push(rowv);
        }
        mm
    }
    fn p2_pawn() -> Vec<Vec<PawnMoveMap>> {
        let mut mm = Vec::with_capacity(8);
        for row in 0..8 {
            let mut rowv = Vec::with_capacity(8);
            for col in 0..8 {
                rowv.push(PawnMoveMap::new_p2(row, col));
            }
            mm.push(rowv);
        }
        mm
    }
}

impl PawnMoveMap {
    fn new_p1(row: usize, col: u32) -> PawnMoveMap {
        let mut mov = Vec::new();
        let mut atk = Vec::new();
        if row > 0 {
            mov.push((row - 1, col));
            if row == 6 {
                mov.push((row - 2, col));
            }
            if col > 0 {
                atk.push((row - 1, col - 1));
            }
            if col < 7 {
                atk.push((row - 1, col + 1));
            }
        }
        PawnMoveMap { mov, atk }
    }
    fn new_p2(row: usize, col: u32) -> PawnMoveMap {
        let mut mov = Vec::new();
        let mut atk = Vec::new();
        if row < 7 {
            mov.push((row + 1, col));
            if row == 1 {
                mov.push((row + 2, col));
            }
            if col > 0 {
                atk.push((row + 1, col - 1));
            }
            if col < 7 {
                atk.push((row + 1, col + 1));
            }
        }
        PawnMoveMap { mov, atk }
    }
}
