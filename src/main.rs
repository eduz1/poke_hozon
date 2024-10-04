use std::fs::{self};

const SAVESIZE: usize = 131072;
const SECTIONS: usize = 14;
const DATASIZE: usize = 3968;
const SIGNATURE: u32 = 0x08012025;

struct FileStructure {
    gamesave_a: Vec<GameSaveBlock>,
    gamesave_b: Vec<GameSaveBlock>,
}

impl FileStructure {
    fn new() -> Self {
        FileStructure {
            gamesave_a: vec![GameSaveBlock::new(); SECTIONS],
            gamesave_b: vec![GameSaveBlock::new(); SECTIONS],
        }
    }
}

#[derive(Clone)]
struct GameSaveBlock {
    data: Vec<u8>,
    sectionid: u16,
    checksum: u16,
    signature: u32,
    saveindex: u32,
}

impl GameSaveBlock {
    fn new() -> Self {
        GameSaveBlock {
            data: vec![0; DATASIZE],
            sectionid: 0,
            checksum: 0,
            signature: 0,
            saveindex: 0,
        }
    }
}

fn calculate_checksum(gamesaveblock: &GameSaveBlock) -> bool {
    let mut checksum: u32 = 0;

    for chunk in gamesaveblock.data.chunks(4) {
        checksum =
            checksum.wrapping_add(u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
    }

    let result = ((checksum >> 16) as u16).wrapping_add((checksum & 0xFFFF) as u16);

    if result != gamesaveblock.checksum {
        println!(
            "Checksum mismatch. Section block {} invalid.\nExpected: 0x{:x} - Result: 0x{:x}",
            gamesaveblock.sectionid, gamesaveblock.checksum, result
        );

        return false;
    }

    return true;
}

fn get_save_sections_data(
    data: &Vec<u8>,
    gamesaves: &mut Vec<GameSaveBlock>,
    mut offset: usize,
) -> bool {
    // Parse data for all 14 sections
    for i in 0..SECTIONS {
        gamesaves[i].sectionid = u16::from_le_bytes([data[offset + 0x0FF4], data[offset + 0x0FF5]]);

        // Read signature early, to avoid further reading if invalid.
        gamesaves[i].signature = u32::from_le_bytes([
            data[offset + 0x0FF8],
            data[offset + 0x0FF9],
            data[offset + 0x0FFA],
            data[offset + 0x0FFB],
        ]);

        if gamesaves[i].signature != SIGNATURE {
            println!(
                "Signature mismatch. Section block {} invalid.\nExpected: 0x{:x} - Result: 0x{:x}",
                gamesaves[i].sectionid, SIGNATURE, gamesaves[i].signature
            );
            return false;
        }

        gamesaves[i].data = data[offset..offset + 0x0F80].to_vec();

        gamesaves[i].saveindex = u32::from_le_bytes([
            data[offset + 0x0FFC],
            data[offset + 0x0FFD],
            data[offset + 0x0FFE],
            data[offset + 0x0FFF],
        ]);

        // Now get the checksum and calculate from data to check for
        // invalid section blocks.
        gamesaves[i].checksum = u16::from_le_bytes([data[offset + 0x0FF6], data[offset + 0x0FF7]]);

        if !calculate_checksum(&gamesaves[i]) {
            return false;
        }

        println!(
            "{} - 0x{:x} - 0x{:x} - {}",
            gamesaves[i].sectionid,
            gamesaves[i].checksum,
            gamesaves[i].signature,
            gamesaves[i].saveindex
        );

        offset += 0x1000;
    }

    return true;
}

fn main() {
    let data = fs::read("Pokemon_FireRed.sav").expect("Unable to read file");

    if data.len() != SAVESIZE {
        println!("Unexpected size: {}. Expected: {}.", data.len(), SAVESIZE);
        return;
    }

    let mut gamesave = FileStructure::new();

    get_save_sections_data(&data, &mut gamesave.gamesave_a, 0);
    get_save_sections_data(&data, &mut gamesave.gamesave_b, 0xE000);
}
