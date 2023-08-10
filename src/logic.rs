// lock function

use crate::consts::*;
use crate::draw::*;
use rand::{ Rng, rngs::ThreadRng };
use termion::input::MouseTerminal;
use std::thread;



///Returns true if the new block is in a valid location 
pub(crate) fn check_transform( 
    board: [[u16; 10]; 24],
    new_block: Block
) -> bool {
    for dy in 0..4 {
        for dx in 0..4 {
            if BLOCKS[new_block.shape][new_block.rotation][dy as usize][dx as usize] != 0 {
                let col_index = new_block.x as i32 + dx as i32;
                let row_index = new_block.y as i32 + dy as i32;
                if col_index < 0 || col_index > 9 || row_index > 23 {
                    return false;
                }
                if row_index >= 0 && board[row_index as usize][col_index as usize] != 0 {
                    return false;
                }
            }
        }
    }
    return true;
}

//locks a block to the board (mutable borrow)

//returns the new board
pub(crate) fn lock_block(board: [[u16; 10]; 24], block: Block) -> [[u16; 10]; 24] {
    let mut board = board;
    for dy in 0..4 {
        for dx in 0..4 {
            if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize] != 0 {
                let col_index = block.x as i32 + dx as i32;
                let row_index = block.y as i32 + dy as i32;
                if row_index >= 0 {
                    //get the neighbours
                    /*[
                        16        1         32

                            12   12   12
                            84,  84,  84,
                    8                         2
                            12   12   12
                            84,  84,  84,

                        128       4        64

                    ]*/
                    let mut key = 0b00000000;
                    //top left neighbour
                    if dy > 0 && dx > 0 {
                        if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize - 1] as u16 != 0 {
                            key += 0b00010000;
                        }
                    }
                    //top neighbour
                    if dy > 0 {
                        if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize] as u16 != 0 {
                            key += 0b00000001;
                        }
                    }
                    //top right neighbour
                    if dy > 0 && dx < 3 {
                        if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize + 1] as u16 != 0 {
                            key += 0b00100000;
                        }
                    }
                    //right neighbour
                    if dx < 3 {
                        if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize + 1] as u16 != 0 {
                            key += 0b00000010;
                        }
                    }
                    //bottom right neighbour
                    if dy < 3 && dx < 3 {
                        if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize + 1] as u16 != 0 {
                            key += 0b01000000;
                        }
                    }
                    //bottom neighbour
                    if dy < 3 {
                        if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize] as u16 != 0 {
                            key += 0b00000100;
                        }
                    }
                    //bottom left neighbour
                    if dy < 3 && dx > 0 {
                        if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize - 1] as u16 != 0 {
                            key += 0b10000000;
                        }
                    }
                    //left neighbour
                    if dx > 0 {
                        if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize - 1] as u16 != 0 {
                            key += 0b00001000;
                        }
                    }
                    let block_type = block.shape as u8 + 1;
                    key += block_type as u16 * 256;
                    board[row_index as usize][col_index as usize] = key;
                }
            }
        }
    }
    board
} 


pub(crate) fn create_shuffled_bag(rng: &mut ThreadRng) -> [usize; 7] {
    let mut new_bag: [usize; 7] = [0; 7];
    for i in 0..7 {
        new_bag[i] = i;
    };
    for i in 0..7 {
        let j = rng.gen_range(0, 7);
        let temp = new_bag[i];
        new_bag[i] = new_bag[j];
        new_bag[j] = temp;
    }
    new_bag
}
/*[
                        16        1         32

                            12   12   12
                            84,  84,  84,
                    8                         2
                            12   12   12
                            84,  84,  84,

                        128       4        64

]*/
pub(crate) fn clear_lines(
    stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>,
    board: &mut [[u16; 10]; 24],
    block: Block,
) -> u32 {
    let mut old_board: [[u16; 10]; 24];
    let mut lines_cleared: u32 = 0;
    for i in 1..board.len() {
        let mut is_full: bool = true;
        for j in 0..board[i].len() {
            if board[i][j] == 0 {
                is_full = false;
            }
        }
        
        if is_full {
            old_board = board.clone();
            lines_cleared += 1;
            if i < board.len() - 1 {
                for j in 0..board[i+1].len() {
                    if board[i+1][j] != 0 {
                        board[i+1][j] &= 0b1111111111001110;
                    }
                }
            }
            for j in 0..board[i-1].len() {
                if board[i-1][j] != 0 {
                    board[i-1][j] &= 0b1111111100111011;
                }
            }

            for k in (1..i+1).rev() {
                for l in 0..board[k].len() {
                    board[k][l] = board[k-1][l];
                }
            }
            update_board_graphics(*board, old_board, block, block, stdout);
            thread::sleep(std::time::Duration::from_millis(64));
        }
    }

    //if lines_cleared == 0 {
    //    new_old_key_buffer_grid = update_graphics(*board, block, *old_key_buffer_grid, stdout);
    //};
    //*old_key_buffer_grid = new_old_key_buffer_grid;
    lines_cleared
}