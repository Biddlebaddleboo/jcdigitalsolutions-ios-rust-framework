use std::string::String;
use std::vec::Vec;

use framework_core::{Error, ErrorKind, PlatformErrorCode};
use framework_files::AppDirectory;
use framework_network::{ResponseHeader, StatusCode};
use framework_transfer::{TransferId, TransferStatus};

const MAGIC_V1: &[u8; 8] = b"JCDITF01";
const MAGIC_V2: &[u8; 8] = b"JCDITF02";
const MAX_RECORD_BYTES: usize = 1_048_576;

#[derive(Clone, Debug)]
pub(crate) struct StoredHeader {
    pub(crate) name: String,
    pub(crate) value: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct StoredRecord {
    pub(crate) id: TransferId,
    /// App-owned task incarnation mirrored in taskDescription to reject stale callbacks.
    pub(crate) task_incarnation: Option<u128>,
    pub(crate) url: String,
    pub(crate) headers: Vec<StoredHeader>,
    pub(crate) directory: AppDirectory,
    pub(crate) destination: String,
    pub(crate) status: TransferStatus,
    pub(crate) cancel_requested: bool,
    /// Durable intent written immediately before atomic destination adoption.
    pub(crate) commit_intent: Option<CommitIntent>,
}

#[derive(Clone, Debug)]
pub(crate) struct CommitIntent {
    pub(crate) status: StatusCode,
    pub(crate) headers: Vec<ResponseHeader>,
}

pub(crate) fn encode(record: &StoredRecord) -> Option<Vec<u8>> {
    if record.task_incarnation == Some(0)
        || (record.commit_intent.is_some() && record.status != TransferStatus::Active)
    {
        return None;
    }
    let mut output = Vec::new();
    output.extend_from_slice(MAGIC_V2);
    output.extend_from_slice(&record.id.get().to_be_bytes());
    match record.task_incarnation {
        None => output.push(0),
        Some(task_incarnation) => {
            output.push(1);
            output.extend_from_slice(&task_incarnation.to_be_bytes());
        }
    }
    output.push(u8::from(record.cancel_requested));
    output.push(directory_tag(record.directory));
    push_string_u32(&mut output, &record.url)?;
    push_string_u32(&mut output, &record.destination)?;
    push_u16(&mut output, record.headers.len())?;
    for header in &record.headers {
        push_string_u16(&mut output, &header.name)?;
        push_bytes_u32(&mut output, &header.value)?;
    }
    match &record.status {
        TransferStatus::Queued => output.push(0),
        TransferStatus::Active => output.push(1),
        TransferStatus::Succeeded { status, headers } => {
            output.push(2);
            output.extend_from_slice(&status.get().to_be_bytes());
            push_u16(&mut output, headers.len())?;
            for header in headers {
                push_string_u16(&mut output, header.name())?;
                push_bytes_u32(&mut output, header.value())?;
            }
        }
        TransferStatus::Failed(error) => {
            output.push(3);
            output.push(error.kind() as u8);
            output.extend_from_slice(
                &error
                    .platform_code()
                    .map_or(0, |code| code.get())
                    .to_be_bytes(),
            );
        }
        TransferStatus::Cancelled => output.push(4),
        _ => return None,
    }
    match &record.commit_intent {
        None => output.push(0),
        Some(intent) => {
            output.push(1);
            output.extend_from_slice(&intent.status.get().to_be_bytes());
            push_u16(&mut output, intent.headers.len())?;
            for header in &intent.headers {
                push_string_u16(&mut output, header.name())?;
                push_bytes_u32(&mut output, header.value())?;
            }
        }
    }
    (output.len() <= MAX_RECORD_BYTES).then_some(output)
}

pub(crate) fn decode(bytes: &[u8]) -> Option<StoredRecord> {
    if bytes.len() > MAX_RECORD_BYTES {
        return None;
    }
    let magic = bytes.get(..MAGIC_V1.len())?;
    let version = if magic == MAGIC_V1 {
        1
    } else if magic == MAGIC_V2 {
        2
    } else {
        return None;
    };
    let mut input = Reader {
        bytes,
        cursor: MAGIC_V1.len(),
    };
    let id = TransferId::new(u128::from_be_bytes(input.array()?))?;
    let task_incarnation = if version == 1 {
        None
    } else {
        match input.byte()? {
            0 => None,
            1 => {
                let incarnation = u128::from_be_bytes(input.array()?);
                if incarnation == 0 {
                    return None;
                }
                Some(incarnation)
            }
            _ => return None,
        }
    };
    let cancel_requested = match input.byte()? {
        0 => false,
        1 => true,
        _ => return None,
    };
    let directory = match input.byte()? {
        0 => AppDirectory::Documents,
        1 => AppDirectory::Caches,
        2 => AppDirectory::Temporary,
        3 => AppDirectory::ApplicationSupport,
        _ => return None,
    };
    let url = input.string_u32()?;
    let destination = input.string_u32()?;
    let header_count = usize::from(input.u16()?);
    let mut request_headers = Vec::with_capacity(header_count);
    for _ in 0..header_count {
        request_headers.push(StoredHeader {
            name: input.string_u16()?,
            value: input.bytes_u32()?.to_vec(),
        });
    }
    let status = match input.byte()? {
        0 => TransferStatus::Queued,
        1 => TransferStatus::Active,
        2 => {
            let status = StatusCode::new(input.u16()?)?;
            let count = usize::from(input.u16()?);
            let mut headers = Vec::with_capacity(count);
            for _ in 0..count {
                headers.push(
                    ResponseHeader::new(input.string_u16()?, input.bytes_u32()?.to_vec()).ok()?,
                );
            }
            TransferStatus::Succeeded { status, headers }
        }
        3 => {
            let kind = decode_error_kind(input.byte()?)?;
            let code = i32::from_be_bytes(input.array()?);
            let mut error = Error::new(kind);
            if let Some(code) = PlatformErrorCode::new(code) {
                error = error.with_platform_code(code);
            }
            TransferStatus::Failed(error)
        }
        4 => TransferStatus::Cancelled,
        _ => return None,
    };
    let commit_intent = match input.byte()? {
        0 => None,
        1 => {
            let status = StatusCode::new(input.u16()?)?;
            let count = usize::from(input.u16()?);
            let mut headers = Vec::with_capacity(count);
            for _ in 0..count {
                headers.push(
                    ResponseHeader::new(input.string_u16()?, input.bytes_u32()?.to_vec()).ok()?,
                );
            }
            Some(CommitIntent { status, headers })
        }
        _ => return None,
    };
    if input.cursor != bytes.len() {
        return None;
    }
    if commit_intent.is_some() && status != TransferStatus::Active {
        return None;
    }
    Some(StoredRecord {
        id,
        task_incarnation,
        url,
        headers: request_headers,
        directory,
        destination,
        status,
        cancel_requested,
        commit_intent,
    })
}

fn directory_tag(directory: AppDirectory) -> u8 {
    match directory {
        AppDirectory::Documents => 0,
        AppDirectory::Caches => 1,
        AppDirectory::Temporary => 2,
        AppDirectory::ApplicationSupport => 3,
        _ => u8::MAX,
    }
}

fn decode_error_kind(value: u8) -> Option<ErrorKind> {
    match value {
        0 => Some(ErrorKind::Unknown),
        1 => Some(ErrorKind::InvalidInput),
        2 => Some(ErrorKind::Unsupported),
        3 => Some(ErrorKind::Unavailable),
        4 => Some(ErrorKind::PermissionDenied),
        5 => Some(ErrorKind::Cancelled),
        6 => Some(ErrorKind::Timeout),
        7 => Some(ErrorKind::NotFound),
        8 => Some(ErrorKind::AlreadyExists),
        9 => Some(ErrorKind::ResourceExhausted),
        10 => Some(ErrorKind::Platform),
        11 => Some(ErrorKind::Internal),
        _ => None,
    }
}

fn push_u16(output: &mut Vec<u8>, value: usize) -> Option<()> {
    output.extend_from_slice(&u16::try_from(value).ok()?.to_be_bytes());
    Some(())
}

fn push_string_u16(output: &mut Vec<u8>, value: &str) -> Option<()> {
    push_bytes_u16(output, value.as_bytes())
}

fn push_bytes_u16(output: &mut Vec<u8>, value: &[u8]) -> Option<()> {
    push_u16(output, value.len())?;
    output.extend_from_slice(value);
    Some(())
}

fn push_string_u32(output: &mut Vec<u8>, value: &str) -> Option<()> {
    push_bytes_u32(output, value.as_bytes())
}

fn push_bytes_u32(output: &mut Vec<u8>, value: &[u8]) -> Option<()> {
    output.extend_from_slice(&u32::try_from(value.len()).ok()?.to_be_bytes());
    output.extend_from_slice(value);
    Some(())
}

struct Reader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl Reader<'_> {
    fn take(&mut self, length: usize) -> Option<&[u8]> {
        let end = self.cursor.checked_add(length)?;
        let value = self.bytes.get(self.cursor..end)?;
        self.cursor = end;
        Some(value)
    }

    fn byte(&mut self) -> Option<u8> {
        Some(*self.take(1)?.first()?)
    }

    fn u16(&mut self) -> Option<u16> {
        Some(u16::from_be_bytes(self.array()?))
    }

    fn array<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N)?.try_into().ok()
    }

    fn bytes_u32(&mut self) -> Option<&[u8]> {
        let length = usize::try_from(u32::from_be_bytes(self.array()?)).ok()?;
        self.take(length)
    }

    fn string_u32(&mut self) -> Option<String> {
        String::from_utf8(self.bytes_u32()?.to_vec()).ok()
    }

    fn string_u16(&mut self) -> Option<String> {
        let length = usize::from(self.u16()?);
        String::from_utf8(self.take(length)?.to_vec()).ok()
    }
}
