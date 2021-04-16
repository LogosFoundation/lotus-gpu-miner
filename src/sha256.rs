use std::convert::TryInto;

pub const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

pub struct Sha256 {
    state: [u32; 8],
}

impl Sha256 {
    pub fn new() -> Sha256 {
        Sha256 {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
        }
    }

    pub fn update_prepad(&mut self, chunk: &[u8; 64]) {
        let mut w = [0u32; 64];
        for (i, val) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes(val.try_into().unwrap());
        }
        for i in 16..64 {
            let s0 =
                w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 =
                w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = (a.rotate_right(2)) ^ (a.rotate_right(13)) ^ (a.rotate_right(22));
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        let added = [a, b, c, d, e, f, g, h];
        for (val, &add) in self.state.iter_mut().zip(added.iter()) {
            *val = val.wrapping_add(add);
        }
    }

    #[allow(dead_code)]
    pub fn hash(&self) -> [u8; 32] {
        let mut hash = [0u8; 32];
        for (bytes, &val) in hash.chunks_exact_mut(4).zip(self.state.iter()) {
            bytes.copy_from_slice(&val.to_be_bytes());
        }
        return hash;
    }

    pub fn state(&self) -> [u32; 8] {
        self.state
    }
}

pub fn sha256d(data: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let hash = sha2::Sha256::digest(&data);
    let hash = sha2::Sha256::digest(&hash);
    hash.into()
}

#[test]
fn test_sha() {
    use hex_literal::hex;
    use sha2::Digest;
    let msg = b"abc";
    let padding = hex!("80000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000018");
    let mut padded_msg = [0; 64];
    padded_msg[..3].copy_from_slice(msg);
    padded_msg[3..].copy_from_slice(&padding);
    let mut sha = Sha256::new();
    sha.update_prepad(&padded_msg);
    let expected = sha2::Sha256::digest(msg);
    assert_eq!(&sha.hash(), expected.as_slice());
}

#[test]
fn test_sha_header() {
    use hex_literal::hex;
    use sha2::Digest;
    let padding = hex!("800000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000280");
    let header = hex!("0000002003682e3420727dacccbce858bdd83fc6bf1fa0d64a04331ce6a0f70700000000d00fb5c33ebff9fd8770587aa485f00685d7fb7b2061bfca4c30a6a68b19057fc27a78609c0d231c00000000");
    let mut padded_header = [0; 128];
    padded_header[..80].copy_from_slice(&header);
    padded_header[80..].copy_from_slice(&padding);
    let mut sha = Sha256::new();
    sha.update_prepad(&padded_header[..64].try_into().unwrap());
    sha.update_prepad(&padded_header[64..].try_into().unwrap());
    let expected = sha2::Sha256::digest(&header);
    assert_eq!(&sha.hash(), expected.as_slice());
}
