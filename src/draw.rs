#![allow(arithmetic_overflow)]
extern crate termion;
use std::io::Write;
use termion::input::MouseTerminal;

use crate::{consts::*, logic::check_transform};

//alt is the numbers that correlate to the characters

fn get_alt(neighbours: u8) -> [u8; 6] {
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

fn apply_alt_to_graphics_buffer(alt: [u8; 6], colour: u8, x: u8, y: u8, draw_offset: [u8; 2], graphics_buffer: &mut [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]]) {
    if colour == 9 {
        return;
    }


    let mut res: [(char, u8, u8); 6] = [(' ', 0, 0); 6];

    for i in 0..6 {
        let col: (u8, u8) = match colour { //inside, edge
            // I cyan      
            0 => (87, 45),
            // O yellow
            1 => (226, 220),
            // T magenta
            2 => (129, 128),
            // Z red
            3 => (196, 160),
            // S green
            4 => (118, 46),
            // J cobalt
            5 => (20, 19),
            // L orange
            6 => (208, 202),
            //ghost block
            7 => (238, 237),
            // empty
            8 => (0, 0),
            // nothing,
            _ => (0, 0),
        };
        res[i] = (CHARREF[alt[i] as usize], col.0, col.1);
    }
    for i in 0..3 {
        graphics_buffer[(y * 2 + draw_offset[1]) as usize][(x * 3 + draw_offset[0]) as usize + i] = res[i];
    }
    for i in 0..3 {
        graphics_buffer[(y * 2 + draw_offset[1]) as usize + 1][(x * 3 + draw_offset[0]) as usize + i] = res[i + 3];
    }
    
}
    

fn get_block_key_grid(block: Block, ghost: bool) -> [[u16; 4]; 4] {
    let mut key_grid: [[u16; 4]; 4] = [[0; 4]; 4];
    for dy in 0..4 {
        for dx in 0..4 {
            if BLOCKS[block.shape][block.rotation][dy as usize][dx as usize] != 0 {
                //the second 8 bits are the type of block; either a block of a specific colour or air

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
                //key += 8 * 256;
                //key_grid[ghost_block.y as usize + dy as usize][ghost_block.x as usize + dx as usize] = key;
                //key -= 8 * 256;
                key += {
                    if ghost {
                        8
                    } else {
                        block.shape as u8 + 1
                    }
                } as u16 * 256;
                key_grid[dy as usize][dx as usize] = key;
            }
        }
    }
    return key_grid;
}


fn create_key_buffer_grid(board: [[u16; 10]; 24], block: Block) -> [[u16; 10]; 24] {

    
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

    let key_grid = get_block_key_grid(block, false);
    let ghost_key_grid = get_block_key_grid(ghost_block, true);

    for dy in 0..4 {
        for dx in 0..4 {
            if ghost_key_grid[dy as usize][dx as usize] != 0 {
                key_buffer_grid[ghost_block.y as usize + dy as usize][ghost_block.x as usize + dx as usize] = ghost_key_grid[dy as usize][dx as usize];
            }
            if key_grid[dy as usize][dx as usize] != 0 {
                key_buffer_grid[block.y as usize + dy as usize][block.x as usize + dx as usize] = key_grid[dy as usize][dx as usize];
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

pub(crate) fn update_graphics_from_buffer(
    graphics_buffer: [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]],
    old_graphics_buffer: [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]],
    stdout: &mut MouseTerminal<termion::raw::RawTerminal<std::io::Stdout>>
) {
    //apply buffer grid to print buffer
    let mut print_buffer: String = String::new();

    for row_index in 0..graphics_buffer.len() { 
        for col_index in 0..graphics_buffer[row_index].len() {
            let (character, colour, background) = graphics_buffer[row_index][col_index];
            let (old_character, old_colour, old_background) = old_graphics_buffer[row_index][col_index];
            if character != old_character || colour != old_colour || background != old_background {
                print_buffer += &format!(
                    "{}{}{}",
                    termion::cursor::Goto(1+col_index as u16, 1+row_index as u16),
                    format!("\x1b[38;5;{};48;5;{}m", colour, background),
                    character
                );
            }
        }
    }

    stdout.write_all(print_buffer.as_bytes()).unwrap();
    stdout.flush().unwrap();
}


pub(crate) fn update_board_graphics_buffer(
    board: [[u16; 10]; 24],
    old_board: [[u16; 10]; 24],
    block: Block,
    old_block: Block,
    graphics_buffer: &mut [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]]
)
 {

    let key_buffer_grid = create_key_buffer_grid(board, block);
    let old_key_buffer_grid = create_key_buffer_grid(old_board, old_block);

    for row_index in 0..key_buffer_grid.len() { 
        for col_index in 0..key_buffer_grid[row_index].len() {
            let key = key_buffer_grid[row_index][col_index];
            let old_key = old_key_buffer_grid[row_index][col_index];
            if key != old_key {
                let neighbours = key;
                let colour = key >> 8;

                let alt: [u8; 6] = {
                    if colour != 0 {
                        get_alt(neighbours as u8)
                    } else {
                        [0; 6]
                    }
                };
                apply_alt_to_graphics_buffer(alt, colour as u8 - 1, col_index as u8, row_index as u8, [DRAW_OFFSET[0],DRAW_OFFSET[1]], graphics_buffer)
            }
        }
    }
}


pub(crate) fn update_hold_block_graphics(
    hold_block: usize,
    graphics_buffer: &mut [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]]
) {
    let mut fine_draw_offset = [DRAW_OFFSET[0] - 11, DRAW_OFFSET[1] + 2];

    for x in 0..9 {
        for y in 0..8 {
            graphics_buffer[fine_draw_offset[1] as usize + y as usize][fine_draw_offset[0] as usize + x as usize] = (' ', 0, 0);
        }
    }

    let key_grid = get_block_key_grid({Block {
        x: 0,
        y: 0,
        shape: hold_block,
        rotation: match hold_block {
            0 => 1,
            _ => 0,
        },
    }}, false);

    if hold_block == 1 {
        fine_draw_offset[0] -= 1;
    } else if hold_block == 0 {
        fine_draw_offset[0] -= 3;
    }
    for dy in 0..4 {
        for dx in 0..4 {
            let key = key_grid[dy as usize][dx as usize];
            let neighbours = key;
            let mut colour = key >> 8;
            let alt: [u8; 6] = {
                if colour != 0 {
                    get_alt(neighbours as u8)
                } else {
                    colour = 10;
                    [0; 6]
                }
            };
            apply_alt_to_graphics_buffer(alt, colour as u8 - 1, dx as u8, dy as u8, fine_draw_offset, graphics_buffer)
        }
    }
}


pub(crate) fn update_next_blocks_graphics(
    current_block_bag: [usize; 7],
    next_block_bag: [usize; 7],
    block_bag_index: usize,
    graphics_buffer: &mut [[(char, u8, u8); DRAW_GRID_SIZE[0]]; DRAW_GRID_SIZE[1]]
) {
    
    //clear next blocks area
    let mut fine_draw_offset: [u8; 2] = [30+DRAW_OFFSET[0],2+DRAW_OFFSET[1]];
    for y in 0..11 {
        for x in 1..=2 {
            graphics_buffer[(2*y+fine_draw_offset[1]) as usize][x+fine_draw_offset[0] as usize] = (' ', 0, 0);
            graphics_buffer[(2*y+fine_draw_offset[1]) as usize+1][x+fine_draw_offset[0] as usize] = (' ', 0, 0);
        }
        for x in 1..4 {
            apply_alt_to_graphics_buffer([0,0,0,0,0,0], 8, x, y, fine_draw_offset, graphics_buffer);
        }
    }
    fine_draw_offset = [32+DRAW_OFFSET[0],DRAW_OFFSET[1]];

    for i in 1..4 {
        let mut block: Block = Block {
            x: 0,
            y: 0,
            rotation: 0,
            shape: {
                if i+block_bag_index < 7 {
                    current_block_bag[i+block_bag_index]
                } else {
                    next_block_bag[i+block_bag_index-7]
                }
            }
        };
        if block.shape == 0 {
            fine_draw_offset[1] += 2;
            fine_draw_offset[0] -= 2;
            block.rotation = 1;
        } else if block.shape == 1 {
            fine_draw_offset[0] -= 1;
        }

        let key_grid = get_block_key_grid(block, false);
        
        for dy in 0..4 {
            for dx in 0..4 {
                let key = key_grid[dy as usize][dx as usize];
                let neighbours = key;
                let mut colour = key >> 8;
                let alt: [u8; 6] = {
                    if colour != 0 {
                        get_alt(neighbours as u8)
                    } else {
                        colour = 10;
                        [0; 6]
                    }
                };
                apply_alt_to_graphics_buffer(alt, colour as u8 - 1, dx as u8, dy as u8, fine_draw_offset, graphics_buffer)
            }
        }
        fine_draw_offset[1] += 5;
        if block.shape == 0 {
            fine_draw_offset[1] += 2;
            fine_draw_offset[0] += 2;
        } else if block.shape == 1 {
            fine_draw_offset[0] += 1;
        }
    }
}
