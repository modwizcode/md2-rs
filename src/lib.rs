/// Implementation of the [MD2](https://datatracker.ietf.org/doc/html/rfc1319) hash algorithm.
#[derive(Clone, Copy)]
pub struct MD2 {
    /// Leftover state after processing in [`Self::update`]. These will actually be the final
    /// digest output after finalization.
    state: [u8; 16],
    /// Incrementally computed "checksum" bytes appended at the end.
    checksum: [u8; 16],
    /// Buffer to hold the remaining input that wasn't able to be processed last update.
    buffer: [u8; 16],
    /// Number of valid bytes in [`Self::buffer`].
    count: usize
}

impl std::fmt::Display for MD2 {
    /// Format the final digest as a hex string without affecting state.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Compute the final checksum on a copy since [`finalize`] consumes self
        let result = (*self).finalize();

        // Format the final checksum
        for b in result {
           f.write_fmt(format_args!("{:02x}", b))?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for MD2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <MD2 as std::fmt::Display>::fmt(self, f)
    }
}

impl MD2 {
    /// Creates a new [`MD2`] ready for use.
    pub const fn new() -> Self {
        Self {
            state: [0u8; 16],
            checksum: [0u8; 16],
            buffer: [0u8; 16],
            count: 0,
        }
    }

    /// Creates a new [`MD2`] with an initial input buffer processed.
    pub fn with_input(input: &[u8]) -> Self {
        let mut initial = Self::new();
        initial.update(input);
        initial
    }

    /// Internally update the state of the hash with a chunk of input
    /// NOTE: This function is internal as it will only process the provided buffer and requires
    /// full 16 byte buffers to be provided.
    fn _update(&mut self, input: &[u8]) {
        debug_assert_eq!(input.len(), 16, "Provided input slice to must be /exactly/ 16 bytes in size");
        // State buffer used in digest computation
        let mut x = [0u8; 48];

        // Initial portion is remaining checksum from previous steps
        x[..16].copy_from_slice(&self.state);
        // Followed by the input
        x[16..32].copy_from_slice(&input);

        // This section is almost verbatim as written in the RFC itself.

        for j in 0..16 {
            x[32 + j] = self.state[j] ^ input[j];
        }

        let mut t = 0;

        for j in 0..18 {
            for k in 0..48 {
                x[k] ^= S[t];
                t = x[k] as usize;
            }

            t = (t + j) % 256;
        }

        // This is the only portion of X that persists into the next cycle
        self.state.copy_from_slice(&x[..16]);

        // Update the checksum that is appended in [`Self::finalize`].

        // L starts at `self.checksum[15]` because if we were following the pseudocode description
        // the next iteration of the checksum loop would start with the last `L = C[j]` where `j = 15`
        // from the final iteration of the inner loop
        let mut l = self.checksum[15];

        // Compute the update checksum bytes
        for j in 0..16 {
            // NOTE: The updated checksum state is computed by using xor instead of following the
            // pseudocode description, this matches the source code, all available implementations
            // and the reference hash results. This appears to be an error in the RFC describing
            // MD2's implementation.
            self.checksum[j] ^= S[(input[j] ^ l) as usize];
            l = self.checksum[j];
        }
    }

    /// Provide input to compute the digest over.
    pub fn update(&mut self, input: &[u8]) {
        // Compute the available input bytes to use, capped at the amount that will fit into the leftover buffer.
        let available = input.len().min(16 - self.count);

        // Attempt to fill the leftover buffer with input data
        self.buffer[self.count..self.count + available].copy_from_slice(&input[..available]);
        // Account for the new data in the buffer
        self.count += available;

        // If we filled up the buffer then we can compute an update cycle over it
        if self.count == 16 {
            self._update(&self.buffer.clone());
        } else {
            // NOTE: We /must/ exit here as  otherwise we will assume we processed all the data
            // and that the remaining bytes in the input are the left over bytes to process next
            // iteration, ignoring the initial unprocessed bytes.
            return;
        }

        // If there's left over data in input then we should process it until we run out
        let mut offset = available;
        let mut remaining = input.len() - available;
        while remaining >= 16 {
            self._update(&input[offset..offset+16]);
            remaining -= 16;
            offset += 16;
        }

        // Store any resulting data back into the leftover buffer
        self.count = remaining;
        self.buffer[..remaining].copy_from_slice(&input[offset..]);
    }

    /// Consume self and return the computed digest.
    pub fn finalize(mut self) -> [u8; 16] {
        // Compute padding bytes required
        let padding_len = (16 - self.count) as u8;

        // Take advantage of internals to directly shove padding bytes into input buffer
        self.buffer[self.count..].fill(padding_len);
        // Apply internal update over the exactly filled and padded buffer
        self._update(&self.buffer.clone());

        // Finally append the checksum bytes (note the clone required to not borrow from self twice)
        // Again using [`Self::_update`] since we already know we're feeding it a properly sized buffer
        self._update(&self.checksum.clone());

        // Final hash is last state
        self.state
    }
}

impl Default for MD2 {
    fn default() -> Self {
        Self::new()
    }
}

/// Substitutions used in the computation of MD2; these are effectively just random bytes for any
/// meaningful purposes.
const S: [u8; 256] = [
    41, 46, 67, 201, 162, 216, 124, 1, 61, 54, 84, 161, 236, 240, 6, 19, 98, 167, 5, 243, 192, 199,
    115, 140, 152, 147, 43, 217, 188, 76, 130, 202, 30, 155, 87, 60, 253, 212, 224, 22, 103, 66,
    111, 24, 138, 23, 229, 18, 190, 78, 196, 214, 218, 158, 222, 73, 160, 251, 245, 142, 187, 47,
    238, 122, 169, 104, 121, 145, 21, 178, 7, 63, 148, 194, 16, 137, 11, 34, 95, 33, 128, 127, 93,
    154, 90, 144, 50, 39, 53, 62, 204, 231, 191, 247, 151, 3, 255, 25, 48, 179, 72, 165, 181, 209,
    215, 94, 146, 42, 172, 86, 170, 198, 79, 184, 56, 210, 150, 164, 125, 182, 118, 252, 107, 226,
    156, 116, 4, 241, 69, 157, 112, 89, 100, 113, 135, 32, 134, 91, 207, 101, 230, 45, 168, 2, 27,
    96, 37, 173, 174, 176, 185, 246, 28, 70, 97, 105, 52, 64, 126, 15, 85, 71, 163, 35, 221, 81,
    175, 58, 195, 92, 249, 206, 186, 197, 234, 38, 44, 83, 13, 110, 133, 40, 132, 9, 211, 223, 205,
    244, 65, 129, 77, 82, 106, 220, 55, 200, 108, 193, 171, 250, 36, 225, 123, 8, 12, 189, 177, 74,
    120, 136, 149, 139, 227, 99, 232, 109, 233, 203, 213, 254, 59, 0, 29, 57, 242, 239, 183, 14,
    102, 88, 208, 228, 166, 119, 114, 248, 235, 117, 75, 10, 49, 68, 80, 180, 143, 237, 31, 26,
    219, 153, 141, 51, 159, 17, 131, 20,
];

#[cfg(test)]
mod test {
    use crate::MD2;

    /// Helper function to simplify testing hashes of strings against their known good results.
    fn test_hash(input: &str, expectation: &str) {
        let result = MD2::with_input(input.as_bytes()).to_string();
        assert_eq!(result, expectation,
            "Testing hash for \"{}\", expected \"{}\" but got \"{}\"", input, expectation, result);
    }

    /// Test against the reference hashes from the [RFC](https://datatracker.ietf.org/doc/html/rfc1319)
    #[test]
    fn basic() {
        test_hash("", "8350e5a3e24c153df2275c9f80692773");
        test_hash("a", "32ec01ec4a6dac72c0ab96fb34c0b5d1");
        test_hash("abc", "da853b0d3f88d99b30283a69e6ded6bb");
        test_hash("message digest", "ab4f496bfb2a530b219ff33031fe06b0");
        test_hash("abcdefghijklmnopqrstuvwxyz", "4e8ddff3650292ab5a4108c3aa47940b");
        test_hash("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                  "da33def2a42df13975352846c30338cd"
        );
        test_hash("12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                  "d5976f79d83d3a0dc9806c3c66f3efd8"
        );
    }
}
