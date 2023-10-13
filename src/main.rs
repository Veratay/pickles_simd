#![feature(portable_simd)]
#![feature(const_trait_impl)]
// #![allow(arithmetic_overflow)] //not necessary bc its not UB for simd

use std::{simd::*, fmt::Display};

//even though heigths can be represented by nibbles, it is more useful for each one to be a byte for simd swizzling
//bottom row is all white
#[derive(Debug,Clone,Copy)]
struct CompressedBoard {
    filled:u8x16,
    colored:u8x16,
    heights:u64,
}

impl Display for CompressedBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filled:u128 = unsafe { std::mem::transmute(self.filled) };
        let colored:u128 = unsafe { std::mem::transmute(self.colored) };

        for y in (1..12).rev() {
            let s = if y%2==1 { 
                format!(" {} {} {} {} {} {} \n",
                    print_helper(colored, filled, y, 0),
                    print_helper(colored, filled, y, 1),
                    print_helper(colored, filled, y, 2),
                    print_helper(colored, filled, y, 3),
                    print_helper(colored, filled, y, 4),
                    print_helper(colored, filled, y, 5),
                )
            } else {
                    format!("{} {} {} {} {} {} {} \n",
                        print_helper(colored, filled, y, 0),
                        print_helper(colored, filled, y, 1),
                        print_helper(colored, filled, y, 2),
                        print_helper(colored, filled, y, 3),
                        print_helper(colored, filled, y, 4),
                        print_helper(colored, filled, y, 5),
                        print_helper(colored, filled, y, 6),
                    )
            };
            f.write_str(&s);
        }
        f.write_str("==============");
        f.write_str(&format!("{:16x}",self.heights))
    }
}

fn print_helper(colored:u128,filled:u128,y:u64,x:u64) -> &'static str {
    if colored & (1<<(y*8 + x)) != 0 {"⬢"} else if filled & (1<<(y*8 + x)) != 0 {"⬡"} else { "_"} 
}

fn new_board() -> CompressedBoard {
    return CompressedBoard {
        filled:u8x16::from_array([255,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]),
        colored:u8x16::splat(0),
        heights:u64::from_ne_bytes([1,1,1,1,1,1,1,1])
    };
}

// const guaranteed_mosaic_and:u32 =  0b00_00_00_00__00_00_00_00___00_00_00_00__01_01_01_01;
// const guaranteed_mosaic_not:u32 =  0b00_00_00_10__11_11_11_10___00_00_00_00__10_10_10_10;

#[inline]
fn placeable(board1:CompressedBoard,board2:CompressedBoard) -> u8x16 {
    let mut heights:u8x16 = unsafe { std::mem::transmute_copy(&[board1.heights,board2.heights]) };
    heights += u8x16::from_array([0,0,0,0,0,0,0,0,16,16,16,16,16,16,16,16]);
    let mut a:u8x16 = vtbl_2(board1.filled, board2.filled, heights);
    let shifts = (heights & Simd::splat(1)) * Simd::splat(2);
    unsafe { println!("{:0b}, {:0b}", std::mem::transmute::<u8x16,[u64; 2]>(a)[0], std::mem::transmute::<u8x16,[u64; 2]>(a)[1]) };
    a = (a & (u8x16::from_array([2,4,8,16,32,64,128,0,2,4,8,16,32,64,128,0]) >> shifts)) << shifts;
    a
}

//const so that compiler can inline the recusion
const DEPTH:usize = 6;

#[inline]
fn calc_score(board:CompressedBoard) {
    //  x x //6
    // x x x //7
    //x x x x //6
    // x x x //7

    //all of this is only 100 asm instructions without any branches/jumps on opt-level=3
    //weird mask to_array then transmute neccesary because it is not guaranteed that the mask will be booleans.
    // to_array() -> [bool; LANES], so this forces the compiler to make the simd_eq be 1 for each lane if true, and 0 otherwise.
    let (row_0, row_1, row_2, row_3) = unsafe {
        let mut row_0:u8x16 = u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(7)).simd_eq(Simd::splat(0)).to_array()));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(14)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(1));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(28)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(2));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(56)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(3));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(112)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(4));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(224)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(5));
        row_0 = row_0 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(192)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(6));

        let mut row_1:u8x16 = u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(7)).simd_eq(Simd::splat(3)).to_array()));
        row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(15)).simd_eq(Simd::splat(6)).to_array())) << Simd::splat(1));
        row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(30)).simd_eq(Simd::splat(12)).to_array())) << Simd::splat(2));
        row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(60)).simd_eq(Simd::splat(24)).to_array())) << Simd::splat(3));
        row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(120)).simd_eq(Simd::splat(48)).to_array())) << Simd::splat(4));
        row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(240)).simd_eq(Simd::splat(96)).to_array())) << Simd::splat(5));
        // row_1 = row_1 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(224)).simd_eq(Simd::splat(224)).to_array())) << Simd::splat(5));

        let mut row_2:u8x16 = u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(7)).simd_eq(Simd::splat(2)).to_array()));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(14)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(1));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(28)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(2));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(56)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(3));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(112)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(4));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(224)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(5));
        row_2 = row_2 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(192)).simd_eq(Simd::splat(4)).to_array())) << Simd::splat(6));

        let mut row_3:u8x16 = u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(3)).simd_eq(Simd::splat(0)).to_array()));
        row_3 = row_3 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(6)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(1));
        row_3 = row_3 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(12)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(2));
        row_3 = row_3 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(24)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(3));
        row_3 = row_3 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(48)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(4));
        row_3 = row_3 | (u8x16::from_array(std::mem::transmute((board.colored & Simd::splat(96)).simd_eq(Simd::splat(0)).to_array())) << Simd::splat(5));

        (row_0,row_1,row_2,row_3)
    };

    let mosaics = row_0 & 
    row_1.swizzle_dyn(u8x16::from_array([1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16])) & 
    row_2.swizzle_dyn(u8x16::from_array([2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17])) &
    row_3.swizzle_dyn(u8x16::from_array([3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18]));
}

//not sure which of these is faster, 2nd takes less asm instructions but it also has to do a simd table lookup
#[inline]
fn insert(board:&mut CompressedBoard,x:u64,colored:u128) {
    let idx = (board.heights >> (x*8)) & 0xFF;
    let shift = (x as u128+idx as u128*8);
    unsafe {
        board.filled += std::mem::transmute::<u128,Simd<u8,16>>(1u128 << shift); 
        board.colored += std::mem::transmute::<u128,Simd<u8,16>>(colored << shift );
    }

    board.heights += 1 << (x*8);

} 

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
        let mut swizzle_idxs_2:u8x16 = idxs + u8x16::from_array([128,128,128,128,128,128,128,128, 0,0,0,0,0,0,0,0]);
        swizzle_idxs_2 -= Simd::splat(16);
        v0.swizzle_dyn(swizzle_idxs) | v1.swizzle_dyn(swizzle_idxs_2)
    };
}
fn main() {

    let mut board = new_board();

    insert(&mut board, 3, 0);

    let p:[[u8; 8]; 2]= unsafe { std::mem::transmute(placeable(board,board)) };

    for (x,i) in p[0].into_iter().enumerate() {
        println!("{}",i);
        if i!=0 {
            let mut new = board;
            insert(&mut new, x as u64, 1);
            println!("{:#}",new);
        }
    }
    // println!("{:#}",board);
}
