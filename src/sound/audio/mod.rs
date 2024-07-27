use adpcm::AdpcmBlock;
use pcm::Pcm16Block;
use std::{fmt::Debug, io};

pub mod adpcm;
pub mod pcm;

#[derive(Debug)]
pub enum BlockError {
    IoError(io::Error),
}

impl From<io::Error> for BlockError {
    fn from(err: io::Error) -> BlockError {
        BlockError::IoError(err)
    }
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

pub(crate) trait BlockTrait: Clone + Debug {
    fn to_bytes(&self) -> Result<Vec<u8>, BlockError>;

    fn as_pcm16_block(&self) -> Pcm16Block;
}

#[derive(Clone, Debug)]
pub enum Block {
    AdpcmBlock(AdpcmBlock),
    Pcm16Block(Pcm16Block),
}

impl BlockTrait for Block {
    fn to_bytes(&self) -> Result<Vec<u8>, BlockError> {
        match self {
            Block::AdpcmBlock(block) => Ok(block.data.clone()),
            Block::Pcm16Block(block) => block.to_bytes(),
        }
    }

    fn as_pcm16_block(&self) -> Pcm16Block {
        match self {
            Block::AdpcmBlock(block) => block.as_pcm16_block(),
            Block::Pcm16Block(block) => block.clone(),
        }
    }
}
