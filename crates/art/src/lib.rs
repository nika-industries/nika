//! Provides a proc-macro for rendering logos to ASCII-art strings.

extern crate proc_macro;

use std::{
  fs::File,
  io::{Cursor, Read},
  num::NonZeroU32,
  path::{Path, PathBuf},
  str::FromStr,
};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// Prints ASCII-art decoded from the provided image file.
#[proc_macro]
pub fn ascii_art(input: TokenStream) -> TokenStream {
  // Parse the input token stream as a string literal
  let input = parse_macro_input!(input as LitStr);
  let path = PathBuf::from_str(&input.value()).expect("Invalid path");
  let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
  let full_path = Path::new(&root).join(&path);

  // Read the image file
  let mut file = File::open(&full_path)
    .unwrap_or_else(|_| panic!("failed to open image file: {full_path:?}"));
  let mut buffer = Vec::new();
  file
    .read_to_end(&mut buffer)
    .expect("failed to read image file");

  // Decode the image
  let image = image::ImageReader::new(Cursor::new(buffer))
    .with_guessed_format()
    .expect("failed to guess image format")
    .decode()
    .expect("failed to decode image");

  // Convert to ASCII art
  let config = artem::config::ConfigBuilder::new()
    .target_size(NonZeroU32::new(60).unwrap())
    .characters("8Ybd'. ".to_string())
    .build();
  let ascii_art = artem::convert(image, &config);

  // Generate the ASCII art string as a static literal
  let output = quote! {
      eprintln!(#ascii_art);
  };

  TokenStream::from(output)
}
