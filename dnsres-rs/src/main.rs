use byteorder::{ BigEndian, WriteBytesExt };
use std::net::UdpSocket;
use rand;

const DNS_ADDR: &str = "8.8.8.8";
const DNS_PORT: u16 = 53;
struct Header {
    id: u16,
    flags: u16,
    qdcount: u16, // nb questions
    ancount: u16, // nb answer resource records
    nscount: u16, // nb authority resource records
    arcount: u16, // nb additional resource records
}

struct Question {
    qname: String, // name being looked up
    qtype: u16, // record type
    qclass: u16, // class
}

impl Header {
    fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut bytes = vec![];
        bytes.write_u16::<BigEndian>(self.id)?;
        bytes.write_u16::<BigEndian>(self.flags)?;
        bytes.write_u16::<BigEndian>(self.qdcount)?;
        bytes.write_u16::<BigEndian>(self.ancount)?;
        bytes.write_u16::<BigEndian>(self.nscount)?;
        bytes.write_u16::<BigEndian>(self.arcount)?;
        Ok(bytes)
    }
}
impl Question {
    fn to_bytes(&self) -> Result<Vec<u8>, std::io::Error> {
        let mut bytes = vec![];
        for part in self.qname.split('.') {
            bytes.push(part.len() as u8);
            for b in part.bytes() {
                bytes.push(b);
            }
        }
        bytes.push(0); // End of name
        bytes.write_u16::<BigEndian>(self.qtype)?;
        bytes.write_u16::<BigEndian>(self.qclass)?;
        Ok(bytes)
    }
}

fn main() {
    // init
    let header = Header {
        id: rand::random(),
        flags: 0x0100, // standard query
        qdcount: 1, // questions
        ancount: 0, // answers
        nscount: 0, // authority records
        arcount: 0, // additional records
    };
    let question = Question {
        qname: "www.example.com".to_string(),
        qtype: 1, // A record
        qclass: 1, // internet class
    };

    // convert
    let message = match construct(header, question) {
        Ok(message) => message,
        Err(e) => {
            return;
        }
    };

    send(&message, DNS_ADDR, DNS_PORT);
}

fn construct(header: Header, question: Question) -> Result<Vec<u8>, std::io::Error> {
    let header_bytes = match header.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to convert header to bytes: {}", e);
            return Err(e);
        }
    };
    let question_bytes = match question.to_bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Failed to convert question to bytes: {}", e);
            return Err(e);
        }
    };
    let mut message = header_bytes;
    message.extend(question_bytes);
    Ok(message)
}

fn send(message: &[u8], address: &str, port: u16) {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(socket) => socket,
        Err(e) => {
            eprintln!("Failed to bind socket: {}", e);
            return;
        }
    };
    match socket.send_to(&message, &format!("{}:{}", address, port)) {
        Ok(_) => println!("Message sent successfully"),
        Err(e) => eprintln!("Failed to send message: {}", e),
    };
}
