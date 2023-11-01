use rand::Rng;

pub fn generate_zobrist_table() -> [[u64; 12]; 64] {
    let mut rng = rand::thread_rng();
    let mut zobrist: [[u64; 12]; 64] = [[0; 12]; 64];
    for i in 0..64 {
        for j in 0..12 {
            zobrist[i][j] = rng.gen();
        }
    }
    zobrist
}