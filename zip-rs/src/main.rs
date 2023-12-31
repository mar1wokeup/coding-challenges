use std::collections::HashMap;
use std::env;
use std::fs;
use std::process;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::io::Write;
use std::io::BufRead;
use std::io::Read;
use std::fs::File;
use std::io::{ self, BufReader, BufWriter };
use std::ptr::read;

#[derive(Eq)]
struct TreeNode {
    character: Option<char>,
    frequency: u32,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
}

impl Ord for TreeNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.frequency.cmp(&self.frequency)
    }
}

impl PartialOrd for TreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.frequency == other.frequency
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        return Err(
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: huffman_tool [encode|decode] <input_file> <output_file>"
            )
        );
    }

    let mode = &args[1];
    let input_file_name = &args[2];
    let output_file_name = &args[3];

    match mode.as_str() {
        "encode" => {
            let input_file = File::open(input_file_name)?;
            let mut output_file = File::create(output_file_name)?;
            encode_file(input_file, &mut output_file)
        }
        "decode" => {
            let input_file = File::open(input_file_name)?;
            let mut output_file = File::create(output_file_name)?;
            decode_file(input_file, &mut output_file)
        }
        _ =>
            Err(
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid mode. use 'encode' or 'decode'."
                )
            ),
    }
}

fn encode_file(input_file: File, output_file: &mut File) -> io::Result<()> {
    let mut content = String::new();
    let mut reader = BufReader::new(&input_file);
    reader.read_to_string(&mut content)?;

    let freq_table = calculate_frequency(&content);
    let huffman_tree = build_huffman_tree(&freq_table).ok_or_else(||
        io::Error::new(io::ErrorKind::Other, "failed to build huffman tree")
    )?;
    let mut huffman_codes = HashMap::new();
    generate_codes(&Some(huffman_tree), String::new(), &mut huffman_codes);

    let mut writer = BufWriter::new(output_file);
    write_header(&mut writer, &freq_table)?;
    encode_and_write_data(&mut writer, &content, &huffman_codes)
}

fn decode_file(input_file: File, output_file: &mut File) -> io::Result<()> {
    let mut reader = BufReader::new(&input_file);
    let freq_table = read_header(&mut reader)?;
    let huffman_tree = build_huffman_tree(&freq_table).ok_or_else(||
        io::Error::new(io::ErrorKind::Other, "failed to rebuild huffman tree")
    )?;

    let mut writer = BufWriter::new(output_file);
    decode_and_write_output(&mut reader, &mut writer, &huffman_tree)
}
////////////////
fn calculate_frequency(content: &str) -> HashMap<char, u32> {
    let mut freq_table = HashMap::new();
    for char in content.chars() {
        *freq_table.entry(char).or_insert(0) += 1;
    }
    freq_table
}

fn build_huffman_tree(freq_table: &HashMap<char, u32>) -> Option<Box<TreeNode>> {
    let mut priority_queue = BinaryHeap::new();

    for (&character, &frequency) in freq_table {
        priority_queue.push(TreeNode {
            character: Some(character),
            frequency,
            left: None,
            right: None,
        });
    }

    while priority_queue.len() > 1 {
        let left = priority_queue.pop().unwrap();
        let right = priority_queue.pop().unwrap();
        let merged = TreeNode {
            character: None,
            frequency: left.frequency + right.frequency,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        };
        priority_queue.push(merged);
    }

    priority_queue.pop().map(Box::new)
}

fn generate_codes(
    node: &Option<Box<TreeNode>>,
    prefix: String,
    code_table: &mut HashMap<char, String>
) {
    if let Some(node) = node {
        if let Some(character) = node.character {
            code_table.insert(character, prefix);
        } else {
            generate_codes(&node.left, prefix.clone() + "0", code_table);
            generate_codes(&node.right, prefix.clone() + "1", code_table);
        }
    }
}
////////////////
fn write_header<W: Write>(writer: &mut W, freq_table: &HashMap<char, u32>) -> std::io::Result<()> {
    for (character, frequency) in freq_table {
        writeln!(writer, "{}:{}", character.escape_default(), frequency)?;
    }
    writeln!(writer, "---")
}

fn encode_and_write_data<W: Write>(
    writer: &mut W,
    text: &str,
    huffman_codes: &HashMap<char, String>
) -> std::io::Result<()> {
    let mut bit_string = String::new();
    for character in text.chars() {
        if let Some(code) = huffman_codes.get(&character) {
            bit_string.push_str(code);
        } else {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid character in input text"
                )
            );
        }
    }

    while bit_string.len() >= 8 {
        let byte_str = &bit_string[..8];
        let byte = u8::from_str_radix(byte_str, 2).unwrap();
        writer.write_all(&[byte])?;
        bit_string = bit_string[8..].to_string();
    }

    if !bit_string.is_empty() {
        let byte = u8::from_str_radix(&format!("{:0<8}", bit_string), 2).unwrap();
        writer.write_all(&[byte])?;
    }

    Ok(())
}
////////////////
fn read_header<R: BufRead>(reader: &mut R) -> std::io::Result<HashMap<char, u32>> {
    let mut freq_table = HashMap::new();

    let reader = std::io::BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        if line == "---" {
            break;
        }
        let parts: Vec<&str> = line.split(':').collect();
        let character = parts[0].chars().next().unwrap();
        let frequency = parts[1].parse::<u32>().unwrap();
        freq_table.insert(character, frequency);
    }

    Ok(freq_table)
}

fn decode_and_write_output<R: BufRead>(
    input_reader: &mut R,
    output_writer: &mut BufWriter<&mut std::fs::File>,
    huffman_tree: &Box<TreeNode>
) -> std::io::Result<()> {
    let mut decoded_string = String::new();
    let mut node = huffman_tree.as_ref();

    let mut buffer = [0; 1];
    while let Ok(1) = input_reader.read(&mut buffer) {
        let mut bits = buffer[0];
        for _ in 0..8 {
            node = if (bits & 0b10000000) != 0 {
                &node.right.as_ref().unwrap()
            } else {
                &node.left.as_ref().unwrap()
            };
            bits <<= 1;
            if node.character.is_some() {
                decoded_string.push(node.character.unwrap());
                node = huffman_tree.as_ref();
            }
        }
    }

    output_writer.write_all(decoded_string.as_bytes())
}

//========================================//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_frequency() {
        let filename = "test.txt";
        let content = fs::read_to_string(filename).unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        });

        let test_str = "Example text with XXX and ttt";
        let freq = calculate_frequency(&test_str);
        assert_eq!(*freq.get(&'X').unwrap(), 3);
        assert_eq!(*freq.get(&'t').unwrap(), 5);
    }

    #[test]
    fn test_huffman_codes() {
        let filename = "test.txt";
        let content = fs::read_to_string(filename).unwrap_or_else(|err| {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        });

        let freq_table = calculate_frequency(&content);
        let huffman_tree = build_huffman_tree(&freq_table).unwrap();
        let mut huffman_codes = HashMap::new();
        generate_codes(&Some(huffman_tree), String::new(), &mut huffman_codes);

        // ensure codes are unique and valid (prefix property)
        let mut code_set = HashMap::new();
        for code in huffman_codes.values() {
            assert!(code_set.insert(code.clone(), ()).is_none());
        }
    }

    #[test]
    fn test_header_format() {
        let mut output = Vec::new();
        let freq_table = HashMap::from([
            ('a', 1),
            ('b', 2),
        ]);
        write_header(&mut output, &freq_table).unwrap();
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("a:1"));
        assert!(output_str.contains("b:2"));
        assert!(output_str.ends_with("---\n"));
    }

    #[test]
    fn test_encoding_and_bit_sequences() {
        let mut output = Vec::new();
        let huffman_codes = HashMap::from([
            ('a', "0".to_string()),
            ('b', "1".to_string()),
        ]);
        encode_and_write_data(&mut output, "abba", &huffman_codes).unwrap();
        assert_eq!(output, vec![0b01011000]);
    }
}
