// use models::CompressionAlgorithm;
// use stream_tools::DynAsyncReader;

// /// Trait for producing stream adapters from compression algorithm
// descriptors. pub(crate) trait AlgorithmToAdapter {
//   fn compressor_adapter(&self)
//     -> Box<dyn Fn(DynAsyncReader) -> DynAsyncReader>;
//   fn decompressor_adapter(
//     &self,
//   ) -> Box<dyn Fn(DynAsyncReader) -> DynAsyncReader>;
// }

// impl AlgorithmToAdapter for CompressionAlgorithm {
//   fn compressor_adapter(
//     &self,
//   ) -> Box<dyn Fn(DynAsyncReader) -> DynAsyncReader> {
//     match self {
//       CompressionAlgorithm::Snappy => {
//         Box::new(|reader| Box::new(snap::read::FrameEncoder::new(reader)))
//       }
//     }
//   }

//   fn decompressor_adapter(
//     &self,
//   ) -> Box<dyn Fn(DynAsyncReader) -> DynAsyncReader> {
//     match self {
//       CompressionAlgorithm::Snappy => {
//         Box::new(|reader| Box::new(snap::read::FrameDecoder::new(reader)))
//       }
//     }
//   }
// }
