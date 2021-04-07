/// These constants have been reverse engineered through the following transactions:
///
/// https://blockstream.info/liquid/tx/a17f4063b3a5fdf46a7012c82390a337e9a0f921933dccfb8a40241b828702f2
/// https://blockstream.info/liquid/tx/d12ff4e851816908810c7abc839dd5da2c54ad24b4b52800187bee47df96dd5c
/// https://blockstream.info/liquid/tx/47e60a3bc5beed45a2cf9fb7a8d8969bab4121df98b0034fb0d44f6ed2d60c7d
///
/// This gives us the following set of linear equations:
///
/// - 1 in, 1 out, 1 fee = 1332
/// - 1 in, 2 out, 1 fee = 2516
/// - 2 in, 2 out, 1 fee = 2623
///
/// Which we can solve using wolfram alpha: https://www.wolframalpha.com/input/?i=1x+%2B+1y+%2B+1z+%3D+1332%2C+1x+%2B+2y+%2B+1z+%3D+2516%2C+2x+%2B+2y+%2B+1z+%3D+2623
pub mod avg_vbytes {
    pub const INPUT: u64 = 107;
    pub const OUTPUT: u64 = 1184;
    pub const FEE: u64 = 41;
}

/// Estimate the virtual size of a transaction based on the number of inputs and outputs.
pub fn estimate_virtual_size(number_of_inputs: u64, number_of_outputs: u64) -> u64 {
    number_of_inputs * avg_vbytes::INPUT + number_of_outputs * avg_vbytes::OUTPUT + avg_vbytes::FEE
}
