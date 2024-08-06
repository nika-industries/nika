use crate::{NarCreateArgs, NarExtractArgs};

pub(crate) fn create_nar_archive(
  NarCreateArgs { target, output }: NarCreateArgs,
) -> miette::Result<()> {
  let start = std::time::Instant::now();

  let output = output.unwrap_or("output.nar".into());
  tracing::info!("creating NAR archive from {target:?}, writing to {output:?}",);

  if !target.exists() {
    tracing::error!("{target:?} does not exist");
    miette::bail!("target does not exist");
  }

  if !target.is_dir() {
    tracing::error!("{target:?} is not a directory");
    tracing::info!(
      "any file system object can be archived, but the parent entry must be a \
       directory to be able to extract correctly."
    );
    miette::bail!("target is not a directory");
  }

  let mut encoder = match nasty::nar::Encoder::new(&target) {
    Ok(e) => e,
    Err(e) => {
      tracing::error!("failed to create NAR encoder: {}", e);
      miette::bail!("failed to create NAR encoder");
    }
  };

  let target_file = match std::fs::File::create_new(&output) {
    Ok(f) => f,
    Err(e) => {
      if e.kind() == std::io::ErrorKind::AlreadyExists {
        tracing::error!("{output:?} already exists");
      } else {
        tracing::error!("failed to create {:?}: {}", output, e);
      }
      miette::bail!("failed to create output file");
    }
  };
  let mut target_file = std::io::BufWriter::new(target_file);

  match std::io::copy(&mut encoder, &mut target_file) {
    Ok(_) => (),
    Err(e) => {
      tracing::error!("failed to write to {:?}: {}", output, e);
      miette::bail!("failed to write to output file");
    }
  };

  let archive_size = std::fs::metadata(&output)
    .expect("failed to get metadata of archive")
    .len();

  tracing::info!(
    "successfully created NAR archive ({}) in {}",
    humansize::format_size(archive_size, humansize::DECIMAL),
    humantime::format_duration(start.elapsed())
      .to_string()
      .split(" ")
      .next()
      .unwrap(),
  );

  Ok(())
}

pub(crate) fn extract_nar_archive(
  NarExtractArgs { target, output }: NarExtractArgs,
) -> miette::Result<()> {
  let start = std::time::Instant::now();

  let output = output.unwrap_or(
    target.file_stem().map(|n| n.into()).ok_or_else(|| {
      tracing::error!("failed to determine output file name");
      miette::miette!("failed to determine output file name")
    })?,
  );
  tracing::info!(
    "extracting NAR archive from {target:?}, writing to {output:?}",
  );

  if !target.exists() {
    tracing::error!("{target:?} does not exist");
    miette::bail!("target does not exist");
  }

  let target_file = match std::fs::File::open(&target) {
    Ok(f) => f,
    Err(e) => {
      tracing::error!("failed to open {:?}: {}", target, e);
      miette::bail!("failed to open target file");
    }
  };
  let mut target_file = std::io::BufReader::new(target_file);

  let decoder = match nasty::nar::Decoder::new(&mut target_file) {
    Ok(d) => d,
    Err(e) => {
      tracing::error!("failed to create NAR decoder: {}", e);
      miette::bail!("failed to create NAR decoder");
    }
  };

  match decoder.unpack(&output) {
    Ok(_) => (),
    Err(e) => {
      tracing::error!("failed to extract NAR archive: {}", e);
      miette::bail!("failed to extract NAR archive");
    }
  };

  tracing::info!(
    "successfully extracted NAR archive in {}",
    humantime::format_duration(start.elapsed())
      .to_string()
      .split(" ")
      .next()
      .unwrap(),
  );

  Ok(())
}
