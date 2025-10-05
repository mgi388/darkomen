use super::pcm::Pcm16Block;

#[rustfmt::skip]
const INDEX_TABLE: [i16; 16] = [
    -1, -1, -1, -1, 2, 4, 6, 8,
    -1, -1, -1, -1, 2, 4, 6, 8,
];

#[rustfmt::skip]
const STEP_TABLE: [i16; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17,
    19, 21, 23, 25, 28, 31, 34, 37, 41, 45,
    50, 55, 60, 66, 73, 80, 88, 97, 107, 118,
    130, 143, 157, 173, 190, 209, 230, 253, 279, 307,
    337, 371, 408, 449, 494, 544, 598, 658, 724, 796,
    876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
    2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358,
    5894, 6484, 7132, 7845, 8630, 9493, 10442, 11487, 12635, 13899,
    15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct AdpcmBlock {
    pub sample: i16,
    pub index: i16,
    pub data: Vec<u8>,
}

impl AdpcmBlock {
    pub fn new(sample: i16, index: i16, data: Vec<u8>) -> Self {
        Self {
            sample,
            index,
            data,
        }
    }

    pub fn as_pcm16_block(&self) -> Pcm16Block {
        let mut decoder = BlockDecoder {
            sample: self.sample as i32,
            index: self.index,
        };

        let mut data = Vec::new();

        for byt in &self.data {
            data.push(decoder.decode(byt & 0x0f));
            data.push(decoder.decode(byt >> 4));
        }

        Pcm16Block::from_int16_slice(&data)
    }
}

struct BlockDecoder {
    sample: i32,
    index: i16,
}

impl BlockDecoder {
    /// Original sample is a 4-bit ADPCM sample.
    ///
    /// The returned new sample is the resulting 16-bit two's complement
    /// variable.
    ///
    /// See https://www.cs.columbia.edu/~hgs/audio/dvi/IMA_ADPCM.pdf at page 32
    /// for the algorithm and for example input. Note: The example input does
    /// appear to be wrong though because `if (0x8763 > 32767) == FALSE` is
    /// actually true.
    fn decode(&mut self, original_sample: u8) -> i16 {
        // Find quantizer step size.
        let step_size = STEP_TABLE[self.index as usize] as i32;

        // Calculate difference:
        //
        //   diff = (original_sample + 1/2) * step_size/4
        //
        // Perform multiplication through repetitive addition.
        let mut diff = 0;
        if original_sample & 4 != 0 {
            diff += step_size;
        }
        if original_sample & 2 != 0 {
            diff += step_size >> 1;
        }
        if original_sample & 1 != 0 {
            diff += step_size >> 2;
        }
        diff += step_size >> 3;

        // Account for sign bit.
        if original_sample & 8 != 0 {
            diff = -diff;
        }

        // Adjust predicted sample based on calculated difference.
        let new_sample = self.sample + diff;

        // Check for underflow and overflow and store 16-bit new sample.
        self.sample = new_sample.clamp(i16::MIN as i32, i16::MAX as i32);

        // Adjust index into step size lookup table using original sample.
        let index = self.index + INDEX_TABLE[original_sample as usize];
        // Check for index underflow and overflow.
        self.index = index.clamp(0, 88);

        // Value has been clamped, can now convert to i16.
        self.sample as i16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // See https://www.cs.columbia.edu/~hgs/audio/dvi/IMA_ADPCM.pdf at page 32
    // for the algorithm and for example input. Note: The example input does
    // appear to be wrong though because `if (0x8763 > 32767) == FALSE` is
    // actually true.
    #[test]
    fn test_decode() {
        let mut decoder = BlockDecoder {
            sample: 0x8700,
            index: 24,
        };
        let original_sample = 0x3;

        assert_eq!(decoder.decode(original_sample), 0x7FFF); // returns new sample
        assert_eq!(decoder.sample, 0x7FFF); // saves new sample on decoder
        assert_eq!(decoder.index, 23); // saves new index on decoder
    }
}
