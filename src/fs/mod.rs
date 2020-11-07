use fuse::Filesystem;

#[derive(Default, Clone)]
struct BitField<T>(T);
bitfield_bitrange!{struct BitField([u32])}

#[repr(C)]
#[derive(Default, Clone)]
struct SuperBlock {
    inode_count: u32,
    block_count: u32,
    used_inodes: BitField<Vec<u32>>,
    used_blocks: BitField<Vec<u32>>,
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
struct Inode {
    size: u32,
    owner: u16,
    group: u16,
    data_p: u32,
    double_data_p: u32,
    mode: u8,
}

#[derive(Default)]
pub struct SFS{
    sb: SuperBlock,
    inodes: Vec<Inode>,
}

impl Filesystem for SFS {}
