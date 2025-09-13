use std::{
    borrow::Cow,
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    path::Path,
};

use bendy::{
    decoding::{Decoder, FromBencode},
    value::Value,
};
use serde::Serialize;
use sha1::{Digest, Sha1};
use tokio::{fs, task::spawn_blocking};

use crate::{
    bencode::error::{BencodeError, BencodeErrorKind},
    error::CommonError,
    task::get_torrent_path,
};
pub mod error;

type BytesList<'a> = Cow<'a, [u8]>;

/// Get the torrent name from the torrent file
/// # Prerequisite
/// the torrent file must have been exported to TORRENT_DIR
/// # Error
/// Returns a [SingleFile](BencodeErrorKind::SingleFile) BencodeError if the torrent is not a multi-file torrent
pub async fn get_torrent_name(hash: &str) -> Result<String, BencodeError> {
    let torrent_path = get_torrent_path(hash);
    let value = get_value(&torrent_path).await?;
    let info = get_info(&value)?;
    check(info)?;
    get_root_dir(info)
}

/// check if the torrent is multi-file, else return SingleFile error
fn check(info: &BTreeMap<BytesList, Value>) -> Result<(), BencodeError> {
    if info.contains_key("length".as_bytes()) {
        return Err(BencodeError {
            kind: BencodeErrorKind::SingleFile,
        });
    }
    Ok(())
}
pub async fn get_value(torrent_path: &Path) -> Result<Value<'_>, BencodeError> {
    let file = fs::read(torrent_path).await.map_err(CommonError::from)?;
    Ok(Value::from_bencode(&file)?.to_owned())
}

fn get_info<'a, 'b: 'a>(
    value: &'a Value<'b>,
) -> Result<&'a BTreeMap<BytesList<'a>, Value<'a>>, BencodeError> {
    if let Value::Dict(dict) = value
        && let Some(Value::Dict(info)) = dict.get("info".as_bytes())
    {
        return Ok(info);
    }
    Err(BencodeError {
        kind: BencodeErrorKind::Decode,
    })
}

fn get_root_dir(info: &BTreeMap<BytesList, Value>) -> Result<String, BencodeError> {
    if let Some(Value::Bytes(name)) = info.get("name".as_bytes()) {
        let name = String::from_utf8_lossy(name).to_string();
        return Ok(name);
    }
    Err(BencodeError {
        kind: BencodeErrorKind::Decode,
    })
}

fn get_files<'a>(info: &'a BTreeMap<BytesList, Value>) -> Result<&'a Vec<Value<'a>>, BencodeError> {
    if let Some(Value::List(files)) = info.get("files".as_bytes()) {
        return Ok(files);
    }
    Err(BencodeError {
        kind: BencodeErrorKind::SingleFile,
    })
}

fn get_file_length_list<'a>(files: &'a Vec<Value>) -> Result<Vec<&'a i64>, BencodeError> {
    let mut lengths = Vec::new();
    for file in files {
        if let Value::Dict(f) = file
            && let Some(Value::Integer(length)) = f.get("length".as_bytes())
        {
            lengths.push(length);
            continue;
        }
        return Err(BencodeError {
            kind: BencodeErrorKind::Decode,
        });
    }
    Ok(lengths)
}

fn get_file_name_list(files: &Vec<Value>) -> Result<Vec<Vec<String>>, BencodeError> {
    let mut paths = Vec::new();
    for file in files {
        if let Value::Dict(f) = file
            && let Some(Value::List(path)) = f.get("path".as_bytes())
        {
            let mut path_vec = Vec::new();
            for node in path {
                if let Value::Bytes(n) = node {
                    let n = String::from_utf8_lossy(n).to_string();
                    path_vec.push(n);
                } else {
                    return Err(BencodeError {
                        kind: BencodeErrorKind::Decode,
                    });
                }
            }
            paths.push(path_vec);
            continue;
        }
        return Err(BencodeError {
            kind: BencodeErrorKind::Decode,
        });
    }
    Ok(paths)
}

pub fn parse_torrent<'a>(value: &'a Value) -> Result<(String, Vec<&'a i64>), BencodeError> {
    let info = get_info(value)?;
    let root_dir = get_root_dir(info)?;
    let files = get_files(info)?;
    let lengths = get_file_length_list(files)?;
    Ok((root_dir, lengths))
}

pub fn get_hash(file: &[u8]) -> Result<String, BencodeError> {
    let mut decoder = Decoder::new(file);
    let obj = decoder.next_object()?.ok_or(BencodeError {
        kind: BencodeErrorKind::Decode,
    })?;
    let mut dict = obj.try_into_dictionary()?;
    while let Some(pair) = dict.next_pair()? {
        if let b"info" = pair.0 {
            let info_bytes = pair.1.try_into_dictionary()?.into_raw()?;
            let hash = Sha1::digest(info_bytes)
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<String>();
            return Ok(hash);
        }
    }
    Err(BencodeError {
        kind: BencodeErrorKind::Decode,
    })
}

#[derive(Debug, Serialize)]
pub struct FileNode {
    pub id: i32,
    pub label: String,
    pub children: Vec<FileNode>,
}

#[derive(Clone, Debug)]
struct FileNodeBuilder {
    pub id: i32,
    pub label: String,
    children_map: HashMap<String, FileNodeBuilder>,
}

impl FileNodeBuilder {
    fn new(id: i32, label: String) -> Self {
        Self {
            id,
            label,
            children_map: HashMap::new(),
        }
    }

    fn build(file_name_list: Vec<Vec<String>>, root_dir: String) -> Self {
        let mut root = Self::new(-1, root_dir);
        let mut _folder = -1;
        for (i, path) in file_name_list.into_iter().enumerate() {
            let mut current_node = &mut root;
            let path_len = path.len();
            for (j, label) in path.into_iter().enumerate() {
                let id = if j < path_len - 1 {
                    _folder -= 1;
                    _folder
                } else {
                    i as i32
                };

                current_node = current_node
                    .children_map
                    .entry(label.clone())
                    .or_insert_with(|| FileNodeBuilder::new(id, label));
            }
        }
        root
    }

    fn into_node(self) -> FileNode {
        let mut children: Vec<_> = self
            .children_map
            .into_values()
            .map(|builder| builder.into_node())
            .collect();

        children.sort_by(FileNode::cmp);

        FileNode {
            id: self.id,
            label: self.label,
            children,
        }
    }
}

impl FileNode {
    pub async fn get_tree(torrent_path: &Path) -> Result<Self, BencodeError> {
        let (file_name_list, root_dir) = {
            let torrent_value = get_value(torrent_path).await?;
            let info = get_info(&torrent_value)?;
            let files = get_files(info)?;

            (get_file_name_list(files)?, get_root_dir(info)?)
        };
        let tree = spawn_blocking(move || {
            let builder = FileNodeBuilder::build(file_name_list, root_dir);
            builder.into_node()
        })
        .await
        .map_err(|_| BencodeError {
            kind: BencodeErrorKind::Decode,
        })?;
        Ok(tree)
    }

    /// default sort: folder first, then file, both by name
    fn cmp(a: &Self, b: &Self) -> Ordering {
        let a_is_file = a.children.is_empty();
        let b_is_file = b.children.is_empty();

        match (a_is_file, b_is_file) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a.label.cmp(&b.label),
        }
    }
}
