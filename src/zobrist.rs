use crate::{Color, Hand, Piece, PieceType, Square};
use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use std::ops;

#[derive(Clone, Copy, Debug)]
pub struct Key(u64);

impl Key {
    pub const ZERO: Key = Key(0);
    pub const COLOR: Key = Key(1);

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl ops::Not for Key {
    type Output = Self;

    fn not(self) -> Self::Output {
        Key(!self.0)
    }
}

impl ops::BitAnd for Key {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Key(self.0 & rhs.0)
    }
}

impl ops::BitXor for Key {
    type Output = Self;

    fn bitxor(self, rhs: Key) -> Self::Output {
        Key(self.0 ^ rhs.0)
    }
}

impl ops::BitXorAssign for Key {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

pub struct ZobristTable {
    board: [[[Key; PieceType::NUM]; Color::NUM]; Square::NUM],
    hands: [[[Key; ZobristTable::MAX_HAND_NUM + 1]; PieceType::NUM_HAND]; Color::NUM],
}

impl ZobristTable {
    const MAX_HAND_NUM: usize = 18;
    pub fn board(&self, sq: Square, p: Piece) -> Key {
        self.board[sq.index()][p.color().index()][p.piece_type().index()]
    }
    pub fn hand(&self, c: Color, pt: PieceType, num: u8) -> Key {
        self.hands[c.index()][Hand::PIECE_TYPE_INDEX[pt.index()]][num as usize]
    }
}

pub static ZOBRIST_TABLE: Lazy<ZobristTable> = Lazy::new(|| {
    let mut board = [[[Key::ZERO; PieceType::NUM]; Color::NUM]; Square::NUM];
    let mut hands = [[[Key::ZERO; 19]; PieceType::NUM_HAND]; Color::NUM];
    let mut rng = StdRng::seed_from_u64(2022);
    for sq in Square::ALL {
        for c in Color::ALL {
            for pt in PieceType::ALL {
                board[sq.index()][c.index()][pt.index()] = Key(rng.gen()) & !Key::COLOR;
            }
        }
    }
    for c in Color::ALL {
        for pt in PieceType::ALL_HAND {
            for num in 0..=ZobristTable::MAX_HAND_NUM {
                hands[c.index()][pt.index()][num] = Key(rng.gen()) & !Key::COLOR;
            }
        }
    }
    ZobristTable { board, hands }
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Move, Position};
    use std::collections::HashSet;

    #[test]
    fn empty() {
        let pos = Position::new(
            [None; Square::NUM],
            [[0; PieceType::NUM_HAND]; Color::NUM],
            Color::Black,
            1,
        );
        assert_eq!(0, pos.key());
    }

    #[test]
    fn default() {
        let pos = Position::default();
        assert_eq!(0x68ac_9984_04fd_3cb0, pos.key());
    }

    #[test]
    fn uniqueness() {
        let mut hs = HashSet::new();
        let mut pos = Position::default();
        for i in 0..100 {
            let moves = pos.legal_moves().into_iter().collect::<Vec<_>>();
            let choice = moves[(i * 100) % moves.len()];
            pos.do_move(choice);
            let key = pos.key();
            assert_eq!(key % 2 == 0, i % 2 == 1);
            hs.insert(key);
        }
        assert_eq!(100, hs.len());
    }

    #[test]
    fn joined() {
        // P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
        // P2 * -HI *  *  *  *  * -KA *
        // P3-FU-FU-FU-FU-FU-FU * -FU-FU
        // P4 *  *  *  *  *  * -FU *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  * +FU *  *  *  * +FU *
        // P7+FU+FU * +FU+FU+FU+FU * +FU
        // P8 *  *  *  *  *  *  * +HI *
        // P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
        // P+00KA
        // P-00KA
        // -
        let key0 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+2726FU
            let moves = [
                Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
                Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
                Move::new_normal(Square::SQ27, Square::SQ26, false, Piece::BFU),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.key()
        };
        let key1 = {
            let mut pos = Position::default();
            // +2726FU,-3334FU,+7776FU
            let moves = [
                Move::new_normal(Square::SQ27, Square::SQ26, false, Piece::BFU),
                Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
                Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.key()
        };
        assert_eq!(key0, key1);
    }

    #[test]
    fn same_board() {
        // P1-KY-KE-GI-KI-OU-KI-GI-KE-KY
        // P2 * -HI *  *  *  *  *  *  *
        // P3-FU-FU-FU-FU-FU-FU * -FU-FU
        // P4 *  *  *  *  *  * -FU *  *
        // P5 *  *  *  *  *  *  *  *  *
        // P6 *  * +FU *  *  *  *  *  *
        // P7+FU+FU * +FU+FU+FU+FU+FU+FU
        // P8 * +KA *  *  *  *  * +HI *
        // P9+KY+KE+GI+KI+OU+KI+GI+KE+KY
        // +
        let keys0 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+8822KA,-3122GI,+0088KA,-2231GI
            // => P-00KA
            let moves = [
                Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
                Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
                Move::new_normal(Square::SQ88, Square::SQ22, false, Piece::BKA),
                Move::new_normal(Square::SQ31, Square::SQ22, false, Piece::WGI),
                Move::new_drop(Square::SQ88, Piece::BKA),
                Move::new_normal(Square::SQ22, Square::SQ31, false, Piece::WGI),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.keys()
        };
        let keys1 = {
            let mut pos = Position::default();
            // +7776FU,-3334FU,+8822KA,-3142GI,+2288KA,-4231GI
            // => P+00KA
            let moves = [
                Move::new_normal(Square::SQ77, Square::SQ76, false, Piece::BFU),
                Move::new_normal(Square::SQ33, Square::SQ34, false, Piece::WFU),
                Move::new_normal(Square::SQ88, Square::SQ22, false, Piece::BKA),
                Move::new_normal(Square::SQ31, Square::SQ42, false, Piece::WGI),
                Move::new_normal(Square::SQ22, Square::SQ88, false, Piece::BKA),
                Move::new_normal(Square::SQ42, Square::SQ31, false, Piece::WGI),
            ];
            moves.iter().for_each(|&m| pos.do_move(m));
            pos.keys()
        };
        println!("{:x?}", keys0);
        println!("{:x?}", keys1);
        assert!(keys0 != keys1);
        assert!(keys0.0 == keys1.0)
    }
}