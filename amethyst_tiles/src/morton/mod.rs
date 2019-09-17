mod luts;
use crate::CoordinateEncoder;
use luts::*;

#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn morton_encode_lut(x: u32, y: u32, z: u32) -> u32 {
    let mut answer: u64 = u64::from(MORTON256_Z[((z >> 16) & 0xFF) as usize]) | // we start by shifting the third byte, since we only look at the first 21 bits
        u64::from(MORTON256_Y[((y >> 16) & 0x0000_00FF) as usize])  |
        u64::from(MORTON256_X[((x >> 16) & 0x0000_00FF) as usize]);

    answer = answer << 48 | u64::from(MORTON256_Z[((z >> 8) & 0xFF) as usize]) | // shifting second byte
        u64::from(MORTON256_Y[((y >> 8) & 0x0000_00FF) as usize])|
        u64::from(MORTON256_X[((x >> 8) & 0x0000_00FF) as usize]);

    (answer << 24 |
        u64::from(MORTON256_Z[((z) & 0x0000_00FF) as usize]) | // first byte
        u64::from(MORTON256_Y[((y) & 0x0000_00FF) as usize]) |
        u64::from(MORTON256_X[((x) & 0x0000_00FF) as usize])) as u32
}

#[inline]
pub fn morton_decode_lut(morton: u32) -> (u32, u32, u32) {
    let single_coord = |morton, shift, table: &[u8]| -> u32 {
        let mut a: u32 = 0;
        for i in 0 as u32..4 {
            a |= u32::from(table[(morton >> ((i * 9) + shift) & 0x0000_01FF) as usize]) << (3 * i);
        }

        a
    };

    (
        single_coord(morton, 0, MORTON512_DECODE_X),
        single_coord(morton, 0, MORTON512_DECODE_Y),
        single_coord(morton, 0, MORTON512_DECODE_Z),
    )
}

#[inline]
pub fn morton_encode_intr_3d(x: u32, y: u32, z: u32) -> u32 {
    use bitintr::Pdep;
    z.pdep(0x2492_4924) | y.pdep(0x1249_2492) | x.pdep(0x0924_9249)
}

#[inline]
pub fn morton_decode_intr_3d(morton: u32) -> (u32, u32, u32) {
    use bitintr::Pext;

    (
        morton.pext(0x0924_9249),
        morton.pext(0x1249_2492),
        morton.pext(0x2492_4924),
    )
}

#[inline]
#[cfg(target_feature = "bmi2")]
pub fn encode(x: u32, y: u32, z: u32) -> u32 {
    morton_encode_intr_3d(x, y, z)
}

#[inline]
#[cfg(target_feature = "bmi2")]
pub fn decode(morton: u32) -> (u32, u32, u32) {
    morton_decode_intr_3d(morton)
}

/// 3D Morton (Z-Order) encoding implementation.
/// This implementation uses the `bmi2` CPU intrinsic if it is available via the `bitintr` crate. If this instruction
/// set is not available, it falls back on simpler computation methods. Using these CPU instruction optimizations requires
/// `RUSTFLAGS=-C target-feature=+bmi2`. If this target feature is not provided, a LUT (Look Up Table) implementation
/// of Morton encoding is used, considered extremely fast but still slightly slower than BMI2 intrinsics.
#[derive(Default, Clone)]
pub struct Encoder;
impl CoordinateEncoder for Encoder {
    fn from_dimensions(_: u32, _: u32, _: u32) -> Self {
        Self {}
    }

    #[inline]
    fn encode(&self, x: u32, y: u32, z: u32) -> Option<u32> {
        Some(encode(x, y, z))
    }
    #[inline]
    fn decode(&self, morton: u32) -> Option<(u32, u32, u32)> {
        Some(decode(morton))
    }
}

/// 3D Morton (Z-Order) encoding implementation.
/// This implementation uses the `bmi2` CPU intrinsic if it is available via the `bitintr` crate. If this instruction
/// set is not available, it falls back on simpler computation methods. Using these CPU instruction optimizations requires
/// `RUSTFLAGS=-C target-feature=+bmi2`. If this target feature is not provided, a LUT (Look Up Table) implementation
/// of Morton encoding is used, considered extremely fast but still slightly slower than BMI2 intrinsics.
///
/// This implementation only performs 2D morton encoding on any given Z-level, while providing Z-levels ia a standard
/// flat-array multiplicative manner. This means that each Z-level is contiguous in memory, but its inner coordinates
/// are still Z-order encoded for some spatial locality.
#[derive(Default, Clone)]
pub struct Encoder2D {
    dimensions: (u32, u32, u32),
    len: u32,
}
impl CoordinateEncoder for Encoder2D {
    fn from_dimensions(x: u32, y: u32, z: u32) -> Self {
        Self {
            dimensions: (x, y, z),
            len: x * y,
        }
    }

    #[inline]
    fn encode(&self, x: u32, y: u32, z: u32) -> Option<u32> {
        use bitintr::Pdep;

        #[cfg(debug_assertions)]
        {
            let check = u32::max_value() / 3;
            if x > check || y > check || z > check {
                return None;
            }
        }

        let morton = (x.pdep(0x5555_5555) | y.pdep(0xAAAA_AAAA)) + (z * self.len);

        Some(morton)
    }
    #[inline]
    fn decode(&self, mut morton: u32) -> Option<(u32, u32, u32)> {
        use bitintr::Pext;

        let z = morton / self.len;
        morton -= z * self.len;

        Some((morton.pext(0x5555_5555), morton.pext(0xAAAA_AAAA), z))
    }
}

#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn encode(x: u32, y: u32, z: u32) -> u32 {
    morton_encode_lut(x, y, z)
}

#[inline]
#[cfg(not(target_feature = "bmi2"))]
pub fn decode(morton: u32) -> (u32, u32, u32) {
    morton_decode_lut(morton)
}

#[inline]
pub fn min(morton1: u32, morton2: u32) -> u32 {
    let d1 = decode(morton1);
    let d2 = decode(morton2);

    if d1.0 > d2.0 || d1.1 > d2.1 || d1.2 > d2.2 {
        morton2
    } else {
        morton1
    }
}

#[inline]
pub fn max(morton1: u32, morton2: u32) -> u32 {
    let d1 = decode(morton1);
    let d2 = decode(morton2);

    if d1.0 < d2.0 || d1.1 < d2.1 || d1.2 < d2.2 {
        morton2
    } else {
        morton1
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use more_asserts::*;
    use rayon::prelude::*;

    #[test]
    fn morton_noncubic_inbounds() -> Result<(), failure::Error> {
        let max_x = 128;
        let max_y = 128;
        let max_z = 12;

        let max = max_x * max_y * max_z;

        let encoder = Encoder2D::from_dimensions(max_x, max_y, max_z);

        for x in 0..max_x {
            for y in 0..max_y {
                for z in 0..max_z {
                    let morton = encoder.encode(x, y, z).unwrap();
                    println!("morton={}", morton);
                    assert_lt!(morton, max);
                    let (dx, dy, dz) = encoder.decode(morton).unwrap();
                    println!("got: {},{},{} vs. {},{},{}", x, y, z, dx, dy, dz);
                    assert_eq!(dx, x);
                    assert_eq!(dy, y);
                    assert_eq!(dz, z);
                }
            }
        }

        Ok(())
    }

    #[test]
    fn morton_intr_decode_encode_match() {
        let test_side: u32 = 128; // 12-bit?

        (0..test_side).into_par_iter().for_each(|x| {
            for y in 0..test_side {
                for z in 0..test_side {
                    let morton = morton_encode_intr_3d(x, y, z);
                    let decode = morton_decode_intr_3d(morton);
                    assert_eq!((x, y, z), decode);
                }
            }
        });
    }

    #[test]
    fn morton_intr_match_lut() {
        let test_side: u32 = 128;
        (0..test_side).into_par_iter().for_each(|x| {
            for y in 0..test_side {
                for z in 0..test_side {
                    let morton_lut = morton_encode_lut(x, y, z);
                    let morton_intr = morton_encode_intr_3d(x, y, z);
                    assert_eq!(morton_lut, morton_intr);
                }
            }
        });
    }

    #[test]
    fn morton_within_array() {
        let test_side: u32 = 128; // 12-bit?
        let max: u32 = test_side * test_side * test_side;

        (0..test_side).into_par_iter().for_each(|x| {
            for y in 0..test_side {
                for z in 0..test_side {
                    let morton = morton_encode_lut(x, y, z);
                    assert_lt!(morton, max);
                }
            }
        });
    }
}