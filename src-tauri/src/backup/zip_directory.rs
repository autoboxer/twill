use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use super::{BackupError, BackupResult};

// Bound metadata before the ZIP reader allocates its central directory
pub(super) fn check_directory(input: &mut File) -> BackupResult<()> {
    let length = input.metadata()?.len();
    let tail_size = length.min(65_557);

    input.seek(SeekFrom::End(-(tail_size as i64)))?;

    let mut tail = vec![0; tail_size as usize];

    input.read_exact(&mut tail)?;

    let offset = (0..tail.len().saturating_sub(21))
        .rev()
        .find(|index| {
            tail[*index..].starts_with(b"PK\x05\x06")
                && *index + 22 + u16_at(&tail, *index + 20) as usize == tail.len()
        })
        .ok_or(BackupError::InvalidArchive("the ZIP directory is missing"))?;
    let end = &tail[offset..];

    if u16_at(end, 4) != 0 || u16_at(end, 6) != 0 || u16_at(end, 8) != u16_at(end, 10) {
        return Err(BackupError::InvalidArchive(
            "split archives are not supported",
        ));
    }

    let mut count = u16_at(end, 10) as u64;
    let mut size = u32_at(end, 12) as u64;
    let mut start = u32_at(end, 16) as u64;
    let end_position = length - tail_size + offset as u64;

    if count == u16::MAX as u64 || size == u32::MAX as u64 || start == u32::MAX as u64 {
        let locator_position = end_position
            .checked_sub(20)
            .ok_or(BackupError::InvalidArchive(
                "the ZIP64 directory is missing",
            ))?;
        let mut locator = [0; 20];

        input.seek(SeekFrom::Start(locator_position))?;
        input.read_exact(&mut locator)?;

        if !locator.starts_with(b"PK\x06\x07")
            || u32_at(&locator, 4) != 0
            || u32_at(&locator, 16) != 1
        {
            return Err(BackupError::InvalidArchive("the ZIP64 locator is invalid"));
        }

        let mut record = [0; 56];

        let record_position = u64_at(&locator, 8);

        input.seek(SeekFrom::Start(record_position))?;
        input.read_exact(&mut record)?;

        if !record.starts_with(b"PK\x06\x06")
            || !(44..=1024).contains(&u64_at(&record, 4))
            || record_position.checked_add(12 + u64_at(&record, 4)) != Some(locator_position)
            || u32_at(&record, 16) != 0
            || u32_at(&record, 20) != 0
            || u64_at(&record, 24) != u64_at(&record, 32)
        {
            return Err(BackupError::InvalidArchive(
                "the ZIP64 directory is invalid",
            ));
        }

        count = u64_at(&record, 32);
        size = u64_at(&record, 40);
        start = u64_at(&record, 48);
    }

    if !(4..=100_000).contains(&count)
        || size > 32 * 1024 * 1024
        || start
            .checked_add(size)
            .is_none_or(|position| position > end_position)
    {
        return Err(BackupError::InvalidArchive(
            "the ZIP directory exceeds its limits",
        ));
    }

    input.seek(SeekFrom::Start(start))?;

    let mut names = BTreeSet::new();

    for _ in 0..count {
        let mut header = [0; 46];

        input.read_exact(&mut header)?;

        if !header.starts_with(b"PK\x01\x02") || u16_at(&header, 8) & 1 != 0 {
            return Err(BackupError::InvalidArchive(
                "the ZIP entry is invalid or encrypted",
            ));
        }

        let name_size = u16_at(&header, 28) as usize;

        if name_size > 128 {
            return Err(BackupError::InvalidArchive("a ZIP filename is too long"));
        }

        let mut name = vec![0; name_size];

        input.read_exact(&mut name)?;

        let name = String::from_utf8(name)
            .map_err(|_| BackupError::InvalidArchive("a ZIP filename is invalid"))?;

        if super::validation::path_limit(&name).is_none() || !names.insert(name) {
            return Err(BackupError::InvalidArchive(
                "a ZIP path is unsafe, unsupported or duplicated",
            ));
        }

        input.seek(SeekFrom::Current(
            u16_at(&header, 30) as i64 + u16_at(&header, 32) as i64,
        ))?;

        if input.stream_position()? > start + size {
            return Err(BackupError::InvalidArchive(
                "the ZIP directory is truncated",
            ));
        }
    }

    if input.stream_position()? != start + size {
        return Err(BackupError::InvalidArchive(
            "the ZIP directory size does not match",
        ));
    }

    input.seek(SeekFrom::Start(0))?;

    Ok(())
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
