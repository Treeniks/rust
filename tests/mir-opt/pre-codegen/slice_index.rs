// skip-filecheck
//@ compile-flags: -O -C debuginfo=0 -Zmir-opt-level=2
// EMIT_MIR_FOR_EACH_PANIC_STRATEGY

#![crate_type = "lib"]
#![feature(slice_ptr_get)]

use std::ops::Range;

// EMIT_MIR slice_index.slice_index_usize.PreCodegen.after.mir
pub fn slice_index_usize(slice: &[u32], index: usize) -> u32 {
    slice[index]
}

// EMIT_MIR slice_index.slice_get_mut_usize.PreCodegen.after.mir
pub fn slice_get_mut_usize(slice: &mut [u32], index: usize) -> Option<&mut u32> {
    slice.get_mut(index)
}

// EMIT_MIR slice_index.slice_index_range.PreCodegen.after.mir
pub fn slice_index_range(slice: &[u32], index: Range<usize>) -> &[u32] {
    &slice[index]
}

// EMIT_MIR slice_index.slice_get_unchecked_mut_range.PreCodegen.after.mir
pub unsafe fn slice_get_unchecked_mut_range(slice: &mut [u32], index: Range<usize>) -> &mut [u32] {
    slice.get_unchecked_mut(index)
}

// EMIT_MIR slice_index.slice_ptr_get_unchecked_range.PreCodegen.after.mir
pub unsafe fn slice_ptr_get_unchecked_range(
    slice: *const [u32],
    index: Range<usize>,
) -> *const [u32] {
    slice.get_unchecked(index)
}
