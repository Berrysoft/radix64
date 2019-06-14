use crate::u6::U6;
use crate::{Config, CustomConfig};

mod arch;

pub trait IntoBlockEncoder: Copy {
    type BlockEncoder: BlockEncoder;

    fn into_block_encoder(self) -> Self::BlockEncoder;
}

pub trait BlockEncoder: Copy {
    fn encode_blocks<'a, 'b>(
        self,
        input: &'a [u8],
        output: &'b mut [u8],
    ) -> (&'a [u8], &'b mut [u8]);
}

#[derive(Debug, Clone, Copy)]
pub struct ScalarBlockEncoder<C>(C);

impl<C> ScalarBlockEncoder<C>
where
    C: Config,
{
    #[inline]
    pub(crate) fn new(config: C) -> Self {
        ScalarBlockEncoder(config)
    }

    fn encode_chunk(self, input: u64, output: &mut [u8; 8]) {
        for (idx, out) in output.iter_mut().enumerate() {
            let shift_amount = 64 - (idx as u64 + 1) * 6;
            let shifted_input = input >> shift_amount;
            *out = self.0.encode_u6(U6::from_low_six_bits(shifted_input as u8));
        }
    }
}

impl<C> BlockEncoder for ScalarBlockEncoder<C>
where
    C: Config,
{
    #[inline]
    fn encode_blocks<'a, 'b>(
        self,
        input: &'a [u8],
        output: &'b mut [u8],
    ) -> (&'a [u8], &'b mut [u8]) {
        use arrayref::{array_mut_ref, array_ref};
        let mut iter = BlockIter::new(input, output);
        for (input_block, output_block) in iter.by_ref() {
            for i in 0..4 {
                self.encode_chunk(
                    u64::from_be_bytes(*array_ref!(input_block, i * 6, 8)),
                    array_mut_ref!(output_block, i * 8, 8),
                );
            }
        }
        iter.remaining()
    }
}

define_block_iter!(
    name = BlockIter,
    input_chunk_size = 26,
    input_stride = 24,
    output_chunk_size = 32,
    output_stride = 32
);

impl IntoBlockEncoder for &CustomConfig {
    type BlockEncoder = ScalarBlockEncoder<Self>;

    #[inline]
    fn into_block_encoder(self) -> Self::BlockEncoder {
        ScalarBlockEncoder::new(self)
    }
}
