mod value_encode;
use crate::value_encode::encode_locator_char;
use value_encode::{encode_num_str, EncodeError};

const WSPR_BIT_COUNT: usize = 162;
const WSPR_MESSAGE_SIZE: usize = 11;
type EncodedWspr = [u8; 162];
pub struct Wspr {
    // third character must always be a number
    // 6 chars
    callsign: String,
    // 4 digit gridsquare
    locator: String,
    // power 0 - 60 dBm
    power_dbm: u8,
}

impl Wspr {
    pub fn new(callsign: String, locator: String, power_dbm: u8) -> Self {
        Self {
            callsign,
            locator,
            power_dbm,
        }
    }
    pub fn encode(&self) -> Result<EncodedWspr, EncodeError> {
        let convolved = Wspr::convolve(self.message()?);
        let interleaved = Wspr::interleave(convolved);
        let levels = Wspr::integrate_sync_values(interleaved);

        Ok(levels)
    }

    fn convolve(data: [u8; 7]) -> EncodedWspr {
        let mut padded_input: [u8; WSPR_MESSAGE_SIZE] = [0; WSPR_MESSAGE_SIZE];
        padded_input[..7].copy_from_slice(&data[..7]);

        let mut output: [u8; WSPR_BIT_COUNT] = [0; WSPR_BIT_COUNT];

        let mut reg0: u32 = 0;
        let mut reg1: u32 = 0;
        let mut bit_index: usize = 0;

        #[allow(clippy::needless_range_loop)]
        for i in 0..WSPR_MESSAGE_SIZE {
            for j in 0..8 {
                // Set input bit according the MSB of current element
                let input_bit = if ((padded_input[i] << j) & 0x80) == 0x80 {
                    1
                } else {
                    0
                };

                // Shift both registers and put in the new input bit
                reg0 <<= 1;
                reg1 <<= 1;
                reg0 |= input_bit;
                reg1 |= input_bit;

                // AND Register 0 with feedback taps, calculate parity
                let mut reg_temp = reg0 & 0xf2d05351;
                let mut parity_bit = 0;
                for _k in 0..32 {
                    parity_bit = ((parity_bit as u32) ^ (reg_temp & 0x01)) as u8;
                    reg_temp >>= 1;
                }
                output[bit_index] = parity_bit;
                bit_index += 1;

                // AND Register 1 with feedback taps, calculate parity
                reg_temp = reg1 & 0xe4613c47;
                parity_bit = 0;
                for _k in 0..32 {
                    parity_bit = ((parity_bit as u32) ^ (reg_temp & 0x01)) as u8;
                    reg_temp >>= 1;
                }

                output[bit_index] = parity_bit;
                bit_index += 1;
                if bit_index >= WSPR_BIT_COUNT {
                    return output;
                }
            }
        }

        output
    }

    fn interleave(data: EncodedWspr) -> EncodedWspr {
        let mut d: EncodedWspr = [0; WSPR_BIT_COUNT];
        let mut i = 0;

        for j in 0..255 {
            let mut j2 = j;
            let mut rev = 0;

            for k in 0..8 {
                if (j2 & 0x01) > 0 {
                    rev |= 1 << (7 - k);
                }
                j2 >>= 1;
            }

            if rev < WSPR_BIT_COUNT {
                d[rev] = data[i];
                i += 1;
            }

            if i >= WSPR_BIT_COUNT {
                return d;
                // panic!("too big?!")
            }
        }

        d
    }

    fn integrate_sync_values(data: EncodedWspr) -> EncodedWspr {
        let sync = [
            1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0,
            0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 0,
            0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 1, 1,
            0, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0,
        ];

        assert_eq!(data.len(), sync.len());

        let mut result: EncodedWspr = [0; WSPR_BIT_COUNT];

        for i in 0..(WSPR_BIT_COUNT - 1) {
            result[i] = data[i] * 2 + sync[i];
        }

        result
    }

    fn encode_call(&self) -> Result<u64, EncodeError> {
        if self.callsign.len() != 6 {
            return Err(EncodeError::InvalidCallsign);
        }
        let bytes = self.callsign.as_bytes();
        let mut n = encode_num_str(bytes[0])?;
        n = n * 36 + encode_num_str(bytes[1])?; // this shouldnt allow spaces
        n = n * 10 + encode_num_str(bytes[2])?;
        n = n * 27 + encode_num_str(bytes[3])? - 10; // no numbers here
        n = n * 27 + encode_num_str(bytes[4])? - 10; // no numbers here
        n = n * 27 + encode_num_str(bytes[5])? - 10; // no numbers here

        Ok(n)
    }

    fn encode_m1(&self) -> Result<u64, EncodeError> {
        let locator_bytes = self.locator.as_bytes();
        let m1: u64 = ((179
            - 10 * encode_locator_char(locator_bytes[0])? as i32
            - encode_num_str(locator_bytes[2])? as i32)
            * 180
            + 10 * encode_locator_char(locator_bytes[1])? as i32
            + encode_num_str(locator_bytes[3])? as i32) as u64;

        Ok(m1)
    }

    fn encode_m(&self) -> Result<u64, EncodeError> {
        let m: u64 = self.encode_m1()? * 128 + self.power_dbm as u64 + 64;

        Ok(m)
    }
    fn message(&self) -> Result<[u8; 7], EncodeError> {
        let mut int_a: u32 = self.encode_call()? as u32;
        let mut int_b: u32 = self.encode_m()? as u32;

        // translate the two integers into a 7-byte array
        let mut bytes: [u8; 7] = [0; 7];
        bytes[3] = ((int_a & 0xF) as u8) << 4;
        int_a >>= 4;
        bytes[2] = (int_a & 0xFF) as u8;
        int_a >>= 8;
        bytes[1] = (int_a & 0xFF) as u8;
        int_a >>= 8;
        bytes[0] = (int_a & 0xFF) as u8;
        bytes[6] = ((int_b & 0x3) << 6) as u8;
        int_b >>= 2;
        bytes[5] = (int_b & 0xFF) as u8;
        int_b >>= 8;
        bytes[4] = (int_b & 0xFF) as u8;
        int_b >>= 8;
        bytes[3] |= int_b as u8 & 0xF;

        Ok(bytes)
    }

    pub fn message_str(&self) -> Result<String, EncodeError> {
        Ok(self
            .message()?
            .into_iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn vk3xe_qf22_23() {
        let expected: Result<EncodedWspr, EncodeError> = Ok([
            3, 3, 0, 2, 0, 2, 0, 2, 1, 2, 2, 0, 3, 3, 3, 2, 2, 2, 1, 2, 0, 1, 2, 1, 3, 1, 1, 2, 2,
            2, 2, 2, 0, 2, 3, 2, 0, 1, 0, 3, 2, 2, 0, 0, 0, 0, 3, 0, 3, 1, 2, 2, 1, 3, 2, 1, 2, 0,
            0, 1, 1, 0, 1, 2, 2, 0, 2, 3, 3, 2, 3, 2, 1, 0, 1, 0, 3, 2, 0, 3, 2, 2, 3, 0, 1, 3, 2,
            2, 2, 1, 3, 0, 1, 2, 1, 0, 0, 0, 1, 2, 2, 2, 2, 2, 3, 0, 0, 3, 0, 2, 1, 3, 1, 0, 3, 3,
            2, 2, 3, 1, 2, 1, 2, 2, 2, 3, 1, 3, 2, 2, 0, 0, 0, 1, 0, 1, 2, 0, 1, 1, 2, 0, 2, 0, 0,
            0, 0, 3, 3, 2, 3, 2, 3, 3, 0, 2, 0, 3, 1, 0, 0, 0,
        ]);

        let result = Wspr::new("VK3XE ".to_owned(), "QF22".to_owned(), 23).encode();
        assert_eq!(expected, result);
    }

    #[test]
    fn vk3tcp_qf22_23() {
        let expected: Result<EncodedWspr, EncodeError> = Ok([
            3, 1, 0, 2, 0, 2, 2, 2, 1, 0, 2, 0, 1, 1, 1, 2, 2, 0, 3, 2, 2, 1, 2, 1, 3, 1, 1, 2, 0,
            0, 2, 2, 0, 2, 3, 2, 2, 3, 0, 3, 2, 0, 0, 0, 2, 0, 1, 0, 3, 1, 2, 2, 3, 1, 0, 1, 2, 0,
            2, 1, 1, 0, 3, 2, 2, 0, 0, 3, 1, 0, 3, 2, 1, 0, 1, 0, 3, 2, 2, 3, 2, 0, 3, 0, 3, 1, 0,
            2, 2, 1, 1, 0, 1, 2, 1, 0, 0, 2, 3, 2, 2, 2, 2, 2, 3, 0, 0, 3, 2, 0, 1, 3, 1, 2, 3, 3,
            2, 2, 3, 1, 2, 3, 2, 2, 2, 3, 3, 3, 2, 0, 2, 0, 2, 1, 0, 1, 2, 0, 3, 1, 0, 2, 0, 0, 0,
            0, 0, 3, 1, 2, 1, 2, 3, 3, 2, 2, 2, 3, 1, 0, 0, 0,
        ]);

        let result = Wspr::new("VK3TCP".to_owned(), "QF22".to_owned(), 23).encode();
        assert_eq!(expected, result);
    }

    #[test]
    fn test_encode_g0upl_io91_20() {
        let expected: Result<EncodedWspr, EncodeError> = Ok([
            3, 3, 0, 2, 0, 0, 2, 2, 1, 2, 0, 0, 3, 3, 1, 0, 2, 0, 3, 2, 2, 3, 2, 3, 1, 3, 1, 2, 0,
            0, 2, 2, 0, 2, 1, 2, 0, 3, 2, 1, 0, 0, 0, 2, 0, 2, 1, 2, 1, 3, 2, 2, 3, 3, 0, 1, 0, 0,
            0, 1, 1, 2, 1, 0, 2, 2, 0, 1, 1, 0, 1, 2, 3, 0, 3, 2, 1, 0, 2, 3, 2, 2, 1, 0, 3, 3, 2,
            0, 2, 1, 3, 0, 3, 0, 3, 0, 2, 2, 3, 2, 2, 2, 2, 2, 3, 2, 0, 1, 2, 0, 3, 3, 1, 0, 1, 1,
            0, 2, 1, 3, 2, 3, 2, 2, 2, 1, 1, 1, 2, 2, 2, 0, 2, 1, 0, 3, 2, 0, 1, 1, 2, 2, 2, 0, 2,
            2, 0, 3, 3, 2, 3, 0, 1, 1, 2, 2, 0, 3, 1, 0, 2, 0,
        ]);
        let encoded = Wspr::new(String::from(" G0UPL"), String::from("IO91"), 20).encode();

        assert_eq!(expected, encoded);
    }

    #[test]
    fn test_encode_call_vk3tcp() {
        let encoded = Wspr::new(String::from("VK3TCP"), String::from("QF22"), 23)
            .encode_call()
            .unwrap();
        assert_eq!(encoded, 223671849)
    }

    #[test]
    fn test_encode_call() {
        let encoded = Wspr::new(String::from("VK3XE "), String::from("QF22"), 23)
            .encode_call()
            .unwrap();

        assert_eq!(encoded, 223674830)
    }

    #[test]
    fn test_encode_m1() {
        let encoded = Wspr::new(String::from("VK3XE "), String::from("QF22"), 23)
            .encode_m1()
            .unwrap();

        assert_eq!(encoded, 3112)
    }

    #[test]
    fn test_encode_m() {
        let expected = Wspr::new(String::from("VK3XE "), String::from("QF22"), 23)
            .encode_m()
            .unwrap();

        assert_eq!(expected, 398423);
    }
}
