#![feature(portable_simd)]
#![feature(const_trait_impl)]
// #![allow(arithmetic_overflow)] //not necessary bc its not UB for simd

use std::simd::*;

//even though heigths can be represented by nibbles, it is more useful for each one to be a byte for simd swizzling
//bottom row is all white
struct CompressedBoard {
    filled:u8x16,
    colored:u8x16,
    heights:u64,
}

// const guaranteed_mosaic_and:u32 =  0b00_00_00_00__00_00_00_00___00_00_00_00__01_01_01_01;
// const guaranteed_mosaic_not:u32 =  0b00_00_00_10__11_11_11_10___00_00_00_00__10_10_10_10;

// fn placeable(board1:CompressedBoard,board2:CompressedBoard) -> u8 {
//     let heights:u8x16 = unsafe { std::mem::transmute_copy(&[board1.heights,board2.heights]) };
//     let shifts = u8x16::from_array([1,2,3]) - (heights & u8x16::from_array([0,0,0,0, 1,1,1,1, 0,0,0,0, 1,1,1,1]));

// }


#[inline]
//finds if there is a colored pixel at the top of one of the columns
//takes 2 boards because it only needs 64bits to calculate for one.
fn is_colored_present(board1:CompressedBoard,board2:CompressedBoard) -> (bool,bool) {
    //has to be 128bits bc of swizzle, there might be a better way to do this
    let heights:u8x16 = unsafe { u8x16::from_array(std::mem::transmute([board1.heights,board2.heights])) };
    let a:u8x16 = vtbl_2(board1.colored, board2.colored, heights);
    //this is the same as 1 & a >> u8x16::from_array([0,1,2,3,4,5,6,7,0,1,2,3,4,5,6,7])
    let x:u8x16 = a & u8x16::from_array([1,2,4,8,16,32,64,128,1,2,4,8,16,32,64,128]);
    let mask:[u64;2] = unsafe { std::mem::transmute(x.simd_eq(Simd::splat(0))) };
    (mask[0] == 0, mask[1] == 0)
}

#[inline]
fn find_guaranteed_mosaic(board:CompressedBoard) -> () {
    //all sizes here are 128bit because of NEON.

    //we only care about the 1st 4 heights bc guaranteed mosaic pattern isint guaranteed when leftmost(which is where mask is based off of) is > 4
    let a = board.heights as u32;

    //all this coercion of a [u32; 4] -> [u8; 16] -> Simd<u8,16> gets optimized away :)
    let heights:u8x16 = unsafe { u8x16::from_array(std::mem::transmute([a,a,a,a])) };

    //1st part is to adjust the row. ex: if its the height of the 2nd column it should be shifted by 1 so that its finding the pattern on the 2nd pattern
    //2nd part is to shift it one more if height is odd bc of hexagonal grid.
    let shifts = u8x16::from_array([0,1,2,3, 0,1,2,3, 0,1,2,3, 0,1,2,3]) + (heights & u8x16::from_array([0,0,0,0, 1,1,1,1, 0,0,0,0, 1,1,1,1]));

    let mut working = vtbl_2(board.filled, board.colored, heights + u8x16::from_array([0,0,0,0,1,1,1,1,0,0,0,0,1,1,1,1]));
    
    working = working >> shifts;
}

#[inline]
fn vtbl_2(v0:u8x16, v1:u8x16, idxs:u8x16) -> u8x16 {
    //replace lower impl with with https://doc.rust-lang.org/core/arch/aarch64/fn.vqtbl2q_u8.html
    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "little"
    ))]
    return  {
        use std::arch::aarch64::{vqtbl2q_u8,uint8x16_t};
        #[repr(align(32))]
        struct u8x16x2(u8x16,u8x16);
        unsafe {
            //uh... yeah
            std::mem::transmute_copy::<uint8x16_t, u8x16>(&vqtbl2q_u8(
                std::mem::transmute_copy(&u8x16x2(v0,v1)),
                std::mem::transmute_copy(idxs))
            )
        }   
    };

    //alternate impl, both for testing on my machine and if above dosent work.
    //On NEON, swizzle_dyn results in a https://developer.arm.com/architectures/instruction-sets/intrinsics/vqtbl1q_u8 intrinsic. If the index is out of bounds, the result of the lookup is zero.
    //We cant normally lookup both the colored and filled vecs at the same time bc NEON only supports 128 bit and each colored and filled is 128 bits themselves,
    //but since we are only selecitng 4 rows from each, we can incombine them into one and use this one height mask by changing which swizzles are valid by changing the most significant bit.
    #[cfg(not(all(
        target_arch = "aarch64",
        target_feature = "neon",
        target_endian = "little"
    )))]
    return {
        let swizzle_idxs:u8x16 = idxs + u8x16::from_array([0,0,0,0,0,0,0,0,128,128,128,128,128,128,128,128]);
        let swizzle_idxs_2:u8x16 = idxs + u8x16::from_array([128,128,128,128,128,128,128,128, 0,0,0,0,0,0,0,0]);
        v0.swizzle_dyn(swizzle_idxs) | v1.swizzle_dyn(swizzle_idxs_2)
    };
}
fn main() {
    println!("Hello, world!");
}
