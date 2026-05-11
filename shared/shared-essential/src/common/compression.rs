use std::io::{Error, Write};

#[tracing::instrument(skip_all)]
pub fn compress_bzip(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut compressor =
        bzip2::write::BzEncoder::new(vec![], bzip2::Compression::best());
    compressor.write_all(data)?;
    compressor.flush()?;

    tracing::debug!(
        "IN: {}, OUT: {}",
        compressor.total_in(),
        compressor.total_out()
    );
    compressor.finish()
}

pub fn decompress_bzip(data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decompressor = bzip2::write::BzDecoder::new(vec![]);
    decompressor.write_all(data)?;
    decompressor.flush()?;
    decompressor.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_compression() {
        let source_string = "Hello, world!".to_string();
        let source_buf = source_string.as_bytes();
        match compress_bzip(source_buf) {
            Ok(compressed_buffer) => {
                match decompress_bzip(compressed_buffer.as_slice()) {
                    Ok(data) => {
                        assert_eq!(String::from_utf8(data).unwrap(), source_string)
                    }
                    Err(_) => panic!("Should not happens!"),
                }
            }
            Err(_) => panic!("Should not happens!"),
        }
    }
}
