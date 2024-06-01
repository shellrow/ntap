use std::error::Error;

use comfy_table::presets::NOTHING;
use comfy_table::*;

pub fn show_interfaces() -> Result<(), Box<dyn Error>> {
    let interfaces = netdev::get_interfaces();

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["#", "Key", "Value"]);
        
    for iface in interfaces {
        table.add_row(vec![
            Cell::new(&iface.index),
            Cell::new("Name"),
            Cell::new(&iface.name),
        ]);
        table.add_row(vec![
            Cell::new(&iface.index),
            Cell::new("Name"),
            Cell::new(&iface.name),
        ]);
    }
    // Set the default alignment for the third column to right
    //let column = table.column_mut(2).expect("Our table has three columns");
    //column.set_cell_alignment(CellAlignment::Right);
    println!("{table}");
    Ok(())
}
