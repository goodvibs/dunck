use lazy_static::lazy_static;
use crate::zobrist::generate_zobrist_table;

lazy_static! {
    pub static ref ZOBRIST_TABLE: [[u64; 12]; 64] = generate_zobrist_table();
}