use std::io::Cursor;
use wo_ooxml::{OoxmlParser, OoxmlFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a minimal XLSX file for testing
    let xlsx_data = include_bytes!("test.xlsx");
    
    let parser = OoxmlParser::new();
    let doc = parser.parse(xlsx_data)?;
    
    println!("Format: {:?}", doc.format);
    println!("Shared strings: {:?}", doc.shared_strings);
    
    if let Some(workbook) = doc.xlsx_workbook {
        println!("Workbook parsed successfully!");
        println!("Number of sheets: {}", workbook.sheets.len());
        
        for sheet in workbook.sheets {
            println!("Sheet: {} ({} rows)", sheet.name, sheet.rows.len());
        }
    } else {
        println!("No workbook found");
    }
    
    Ok(())
}