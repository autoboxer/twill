use std::fmt::Write as _;
use std::io::{Read, Write};

use sha2::{Digest, Sha256};

use super::{BackupError, BackupResult};

const COPY_BUFFER_SIZE: usize = 64 * 1024;

pub(super) fn copy_and_digest(
    input: &mut impl Read,
    output: &mut impl Write,
    limit: u64,
) -> BackupResult<(u64, String)> {
    let mut buffer = [0; COPY_BUFFER_SIZE];
    let mut digest = Sha256::new();
    let mut byte_size = 0_u64;

    loop {
        let read = input.read(&mut buffer)?;

        if read == 0 {
            break;
        }

        byte_size = byte_size
            .checked_add(read as u64)
            .ok_or(BackupError::Integrity("a file is too large"))?;

        if byte_size > limit {
            return Err(BackupError::Integrity("a file grew while exporting"));
        }

        output.write_all(&buffer[..read])?;
        digest.update(&buffer[..read]);
    }

    Ok((byte_size, finish_digest(digest)))
}

pub(super) fn finish_digest(digest: Sha256) -> String {
    let mut encoded = String::with_capacity(64);

    for byte in digest.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }

    encoded
}
