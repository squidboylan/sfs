use fuse::Filesystem;
use std::fs::File;
use std::io::Write;
use std::io::Read;
use std::io::Result;

#[derive(Default, Debug, Clone, PartialEq)]
struct BitField<T>(T);
bitfield_bitrange!{struct BitField([u32])}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
struct SuperBlock {
    inode_count: u32,
    block_count: u32,
    used_inodes: BitField<Vec<u32>>,
    used_blocks: BitField<Vec<u32>>,
}

impl SuperBlock {
    pub fn new(inode_count: u32, block_count: u32) -> Self {
        assert!(inode_count % 32 == 0);
        assert!(block_count % 32 == 0);
        SuperBlock {
            inode_count,
            block_count,
            used_inodes: BitField(vec![0; inode_count as usize/32]),
            used_blocks: BitField(vec![0; block_count as usize/32]),
        }
    }

    pub fn load_from_file(image: &mut File) -> Result<SuperBlock> {
        let mut inode_count_array: [u8; 4] = [0; 4];
        let mut block_count_array: [u8; 4] = [0; 4];
        image.read_exact(&mut inode_count_array)?;
        image.read_exact(&mut block_count_array)?;
        let inode_count: u32 = u32::from_le_bytes(inode_count_array);
        let block_count: u32 = u32::from_le_bytes(block_count_array);

        let mut used_inodes: Vec<u32> = vec![0; inode_count as usize/32];
        let mut used_blocks: Vec<u32> = vec![0; block_count as usize/32];
        used_inodes.shrink_to_fit();
        used_blocks.shrink_to_fit();
        unsafe {
            image.read_exact(std::slice::from_raw_parts_mut(used_inodes.as_mut_ptr() as *mut u8, used_inodes.len() * 4))?;
            image.read_exact(std::slice::from_raw_parts_mut(used_blocks.as_mut_ptr() as *mut u8, used_blocks.len() * 4))?;
        }
        Ok(SuperBlock {
            inode_count,
            block_count,
            used_inodes: BitField(used_inodes),
            used_blocks: BitField(used_blocks),
        })
    }

    pub fn dump(&self, image: &mut File) -> Result<()> {
        image.write(&self.inode_count.to_le_bytes())?;
        image.write(&self.block_count.to_le_bytes())?;
        unsafe {
            image.write(std::slice::from_raw_parts(self.used_inodes.0.as_ptr() as *const u8, self.used_inodes.0.len() * 4))?;
            image.write(std::slice::from_raw_parts(self.used_blocks.0.as_ptr() as *const u8, self.used_blocks.0.len() * 4))?;
        }
        Ok(())
    }
}

/// This will get aligned to 4 bytes and its size must be a multiple of 4 bytes, because accessing
/// padding is UB we add padding to prevent UB when accessing the struct to write it to disk.
/// (Maybe the compiler does this automatically?)
#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
struct Inode {
    size: u32,
    owner: u16,
    group: u16,
    data_p: u32,
    double_data_p: u32,
    mode: u8,
    pad: [u8; 3],
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug, Clone, PartialEq)]
struct DataBlock {
    bytes: [u8; 4096],
}

impl std::default::Default for DataBlock {
    fn default() -> Self {
        Self {
            bytes: [0;4096],
        }
    }
}

#[repr(C)]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct SFS{
    sb: SuperBlock,
    inodes: Vec<Inode>,
}

impl SFS {
    pub fn new(inode_count: u32, block_count: u32) -> Self {
        SFS {
            sb: SuperBlock::new(inode_count, block_count),
            inodes: vec![Inode::default(); inode_count as usize],
        }
    }

    pub fn load_from_file(image: &mut File) -> Result<SFS> {
        let sb = SuperBlock::load_from_file(image)?;
        let mut inodes = vec![Inode::default(); sb.inode_count as usize];
        inodes.shrink_to_fit();
        unsafe {
            image.read_exact(std::slice::from_raw_parts_mut(inodes.as_mut_ptr() as *mut u8, inodes.len() * std::mem::size_of::<Inode>()))?;
        }
        Ok(SFS {
            sb,
            inodes,
        })
    }

    pub fn dump(&self, image: &mut File) -> Result<()> {
        self.sb.dump(image)?;
        let data_blocks = vec![DataBlock::default(); self.sb.block_count as usize];
        unsafe {
            image.write(std::slice::from_raw_parts(self.inodes.as_ptr() as *const u8, self.inodes.len() * std::mem::size_of::<Inode>()))?;
            image.write(std::slice::from_raw_parts(data_blocks.as_ptr() as *const u8, data_blocks.len() * std::mem::size_of::<DataBlock>()))?;
        }
        Ok(())
    }
}

impl Filesystem for SFS {}

#[test]
fn test_dump_and_load() {
    use std::io::Seek;
    use std::io::SeekFrom;
    use tempfile::tempfile;

    let kb_size = 16 * 1024;
    let inode_count = kb_size;
    let block_count = kb_size/4;
    let sfs = SFS::new(inode_count, block_count);
    let mut file = tempfile().expect("Failed to open temp file");
    sfs.dump(&mut file).expect("Failed to dump FS to disk");
    file.seek(SeekFrom::Start(0)).expect("Failed to seek to beginning of file");
    let loaded_sfs = SFS::load_from_file(&mut file).expect("Failed to load FS from disk");

    assert_eq!(sfs, loaded_sfs);
}

/*
#[test]
fn test_super_block_size_and_alignment() {
    assert_eq!(std::mem::size_of::<SuperBlock>(), 4096);
    assert_eq!(std::mem::align_of::<SuperBlock>(), 4096);
}
*/
