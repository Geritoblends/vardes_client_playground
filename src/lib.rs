use wasmtime::{Module, Instance, Memory, Table};

pub struct Message<const N: usize> {
    payload: [u8; N],
    len: usize
}
