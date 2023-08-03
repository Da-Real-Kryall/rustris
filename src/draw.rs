
extern crate termion;
use std::io::Write;
use termion::{
    cursor::Goto,
    input::MouseTerminal
};

use crate::{consts::*, logic::check_transform};

//alt is the numbers that correlate to the characters

pub(crate) fn get_alt(neighbours: u8) -> [u8; 6] {
    /*
     [0, 1, 2,
      3, 4, 5]

    16        1         32

        12   12   12
        84,  84,  84,
8                         2
        12   12   12
        84,  84,  84,

    128       4        64

    */

    let mut block_alt: [u8; 6] = [
        0b0100, 0b1100, 0b1000,
        0b0010, 0b0011, 0b0001
    ]; //1 is there's a block there, 0 is there isn't

    if neighbours & 1 == 1 {
        block_alt[0] |= 0b0010;
        block_alt[1] |= 0b0011;
        block_alt[2] |= 0b0001;
    }
    if neighbours & 2 == 2 {
        block_alt[2] |= 0b0100;
        block_alt[5] |= 0b0010;
    }
    if neighbours & 4 == 4 {
        block_alt[3] |= 0b0100;
        block_alt[4] |= 0b1100;
        block_alt[5] |= 0b1000;
    }
    if neighbours & 8 == 8 {
        block_alt[0] |= 0b1000;
        block_alt[3] |= 0b0001;
    }
    if neighbours & 0b11001 == 0b11001 {
        block_alt[0] |= 0b0001;
    }
    if neighbours & 0b100011 == 0b100011 {
        block_alt[2] |= 0b10;
    }
    if neighbours & 0b10001100 == 0b10001100 {
        block_alt[3] |= 0b1000;
    }
    if neighbours & 0b1000110 == 0b1000110 {
        block_alt[5] |= 0b100;
    }

    return block_alt;
}


fn format_alt_to_string(alt: [u8; 6], colour: u8, x: u8, y: u8) -> String {
    let mut res_string: String = String::new();
    
    res_string.push_str(&format!(
        "{}",
        Goto(x as u16 * 3 + 1 + DRAW_OFFSET[0], y as u16 * 2 + 1 + DRAW_OFFSET[1])
    ));
    //push three of the same character to make it look square
    res_string.push_str(&format!(
        "{}",
        //the foreground is a slightly darker version of the background
        match colour {                           //inside, edge
            // I cyan      
            0 => {format!("\x1b[38;5;{};48;5;{}m", 87, 45)},
            // O yellow
            1 => {format!("\x1b[38;5;{};48;5;{}m", 226, 220)},
            // T magenta
            2 => {format!("\x1b[38;5;{};48;5;{}m", 129, 128)},
            // Z red
            3 => {format!("\x1b[38;5;{};48;5;{}m", 196, 160)},
            // S green
            4 => {format!("\x1b[38;5;{};48;5;{}m", 118, 46)},
            // J cobalt
            5 => {format!("\x1b[38;5;{};48;5;{}m", 20, 19)},
            // L orange
            6 => {format!("\x1b[38;5;{};48;5;{}m", 208, 202)},
            //ghost block
            7 => {format!("\x1b[38;5;{};48;5;{}m", 238, 237)},
            // empty
            _ => {format!("\x1b[38;5;{};48;5;{}m", 255, 0)},
        },
    ));


    //let alt: [u8; 6] = {
    //    if colour == 7 {
    //        [18; 6]
    //    } else {
    //        alt
    //    }
    //};


    res_string.push_str(&format!(
        "{}{}{}",//lol i'll compress this later
        CHARREF[alt[0] as usize],
        CHARREF[alt[1] as usize],
        CHARREF[alt[2] as usize]
    ));

    res_string.push_str(&format!(
        "{}",
        Goto(x as u16 * 3 + 1 + DRAW_OFFSET[0], y as u16 * 2 + 2 + DRAW_OFFSET[1])
    ));
    res_string.push_str(&format!(
        "{}{}{}",
        CHARREF[alt[3] as usize],
        CHARREF[alt[4] as usize],
        CHARREF[alt[5] as usize]
    ));
    //res_string.push_str(format!("\x1b[0m").as_str());
    return res_string;
}

pub(crate) fn create_key_buffer_grid(board: [[u16; 10]; 24], block: Block) -> [[u16; 10]; 24] {

    
    //make draw buffer grid (a 2d array of the characters being printed to the larger board))
    //the key is the colour and the neighbours, neighbours occupy the first 8 bits and the colours+everything else the second 8
    let mut key_buffer_grid: [[u16; 10]; 24] = [[0; 10]; 24];

    //draw unlocked block and ghost block

    let mut ghost_block = block;
    while check_transform(board, ghost_block) {
        ghost_block.y += 1;
    }
    ghost_block.y -= 1;
    /* neighbours
        012
        7 3
        654
     */
    for dy in 0..4 {
        for dx in 0..4 {
            if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize] != 0 {
                //the second 8 bits are the type of block; either a block of a specific colour or air

                let col_index = block.x as u16 + dx;
                let row_index = block.y as u16 + dy;
                let mut key: u16 = 0;
                //top left neighbour
                if dy > 0 && dx > 0 {
                    if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize - 1] as u16 != 0 {
                        key += 16;
                    }
                }
                //top neighbour
                if dy > 0 {
                    if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize] as u16 != 0 {
                        key += 1;
                    }
                }
                //top right neighbour
                if dy > 0 && dx < 3 {
                    if BLOCKS[block.shape][block.rotation][dy as usize - 1][dx as usize + 1] as u16 != 0 {
                        key += 32;
                    }
                }
                //right neighbour
                if dx < 3 {
                    if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize + 1] as u16 != 0 {
                        key += 2;
                    }
                }
                //bottom right neighbour
                if dy < 3 && dx < 3 {
                    if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize + 1] as u16 != 0 {
                        key += 64;
                    }
                }
                //bottom neighbour
                if dy < 3 {
                    if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize] as u16 != 0 {
                        key += 4;
                    }
                }
                //bottom left neighbour
                if dy < 3 && dx > 0 {
                    if BLOCKS[block.shape][block.rotation][dy as usize + 1][dx as usize - 1] as u16 != 0 {
                        key += 128;
                    }
                }
                //left neighbour
                if dx > 0 {
                    if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize - 1] as u16 != 0 {
                        key += 8;
                    }
                }
                let block_type = block.shape as u8 + 1;
                key += 8 * 256;
                key_buffer_grid[ghost_block.y as usize + dy as usize][ghost_block.x as usize + dx as usize] = key;
                key -= 8 * 256;
                key += block_type as u16 * 256;
                key_buffer_grid[row_index as usize][col_index as usize] = key;
            }
        }
    }

    //draw board
    for y in 0..24 {
        for x in 0..10 {
            if board[y][x] != 0 {
                key_buffer_grid[y][x] = board[y][x];
            }
        }
    }

    return key_buffer_grid;
}


/* returns the key_buffer_grid */
pub(crate) fn update_graphics(
    board: [[u16; 10]; 24],
    old_board: [[u16; 10]; 24],
    block: Block,
    old_block: Block,
    stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>
)
 {

    let key_buffer_grid = create_key_buffer_grid(board, block);
    let old_key_buffer_grid = create_key_buffer_grid(old_board, old_block);


    //apply buffer grid to print buffer
    let mut print_buffer: String = String::new();

    for row_index in 0..key_buffer_grid.len() { 
        for col_index in 0..key_buffer_grid[row_index].len() {
            let key = key_buffer_grid[row_index][col_index];
            let old_key = old_key_buffer_grid[row_index][col_index];
            if key != old_key {
                //continue;//key = 0b0000001011111111;
                let neighbours = key; //& 0b11111111;
                let colour = key >> 8;
                let alt: [u8; 6] = {
                    if colour != 0 {
                        get_alt(neighbours as u8)
                    } else {
                        [0; 6]
                    }
                };
                print_buffer.push_str(format_alt_to_string(alt, colour as u8 - 1, col_index as u8, row_index as u8).as_str());
            }

        }
    }
    stdout.write_all(print_buffer.as_bytes()).unwrap();
    stdout.flush().unwrap();
}