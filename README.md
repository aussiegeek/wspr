# Rust WSPR generation

This library generates frequency shifts required for generating [WSPR](http://www.wsprnet.org) amateur radio transmissions.

It's return output is a 161 byte array of uint8 values of 0-4, which is the encoded symbol needing to be transmitted. Each symbol is shifted by a multiple of $\frac{12000}{8192}$ (1.46Hz), and transmitted for the reciporical of the tone spacing $\frac{1}{12000/8192}$ (ie. 0.68 seconds)

For example with a base frequency of 7,040,000:

| Symbol | Frequency (hz) |
| ------ | -------------- |
| 0      | 7,040,000.00   |
| 1      | 7,040,001.46   |
| 2      | 7,040,002.92   |
| 3      | 7,040,004.35   |

## References

1. <https://swharden.com/software/FSKview/wspr/>
2. <https://github.com/swharden/WsprSharp> (although I don't think it handles callsign padding correctly)
3. <https://gist.github.com/bd1es/a782e2529b8289288fadd35e407f6440>
4. <https://github.com/kholia/wspr_encoder_web>
