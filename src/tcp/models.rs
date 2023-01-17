use async_trait::async_trait;
use chrono::{Datelike, Timelike};
use my_tcp_sockets::{
    socket_reader::{ReadBuffer, ReadingTcpContractFail, SocketReader},
    TcpSocketSerializer,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

const OUR_MARKER: u8 = 'O' as u8;
const SOURCE_MARKER: u8 = 'S' as u8;

#[derive(Debug, Clone)]
pub enum BidAskDateTime {
    Source(DateTimeAsMicroseconds),
    Our(DateTimeAsMicroseconds),
}

impl BidAskDateTime {
    #[cfg(test)]
    pub fn unwrap_as_our_date(&self) -> &DateTimeAsMicroseconds {
        match self {
            BidAskDateTime::Our(data) => data,
            _ => panic!("BidAsk::unwrap_as_our_date: not Our Date"),
        }
    }
    #[cfg(test)]
    pub fn unwrap_as_source_date(&self) -> &DateTimeAsMicroseconds {
        match self {
            BidAskDateTime::Source(data) => data,
            _ => panic!("BidAsk::unwrap_as_source_date: not Source Date"),
        }
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        match &self {
            BidAskDateTime::Source(date_time) => {
                dest.push(SOURCE_MARKER as u8);
                date_time_to_string(dest, date_time);
            }
            BidAskDateTime::Our(date_time) => {
                dest.push(OUR_MARKER as u8);
                date_time_to_string(dest, date_time);
            }
        };
    }

    pub fn parse(src: &str) -> Self {
        let source_time = src.as_bytes()[0] != OUR_MARKER;
        let date_time = parse_date_time(&src[1..]);

        if source_time {
            BidAskDateTime::Source(date_time)
        } else {
            BidAskDateTime::Our(date_time)
        }
    }
}

fn parse_date_time(line: &str) -> DateTimeAsMicroseconds {
    let year: i32 = line[0..4].parse().unwrap();
    let month: u32 = line[4..6].parse().unwrap();
    let day: u32 = line[6..8].parse().unwrap();
    let hour: u32 = line[8..10].parse().unwrap();
    let min: u32 = line[10..12].parse().unwrap();
    let sec: u32 = line[12..14].parse().unwrap();

    let micros_str = &line[15..];
    let mut micro: i64 = micros_str.parse().unwrap();

    match micros_str.len() {
        1 => {
            micro *= 100_000;
        }
        2 => {
            micro *= 10_000;
        }
        3 => {
            micro *= 1_000;
        }
        4 => {
            micro *= 100;
        }
        5 => {
            micro *= 10;
        }
        _ => {}
    }

    DateTimeAsMicroseconds::create(year, month, day, hour, min, sec, micro)
}

fn date_time_to_string(result: &mut Vec<u8>, dt: &DateTimeAsMicroseconds) {
    let dt = dt.to_chrono_utc();

    result.extend_from_slice(dt.year().to_string().as_bytes());

    push_with_leading_zero(result, dt.month() as u8);
    push_with_leading_zero(result, dt.day() as u8);
    push_with_leading_zero(result, dt.hour() as u8);
    push_with_leading_zero(result, dt.minute() as u8);
    push_with_leading_zero(result, dt.second() as u8);
    result.push('.' as u8);
    let mut ms_as_string = dt.nanosecond().to_string();

    let ms_as_slice = if ms_as_string.len() < 6 {
        while ms_as_string.len() < 3 {
            ms_as_string.push('0');
        }

        &ms_as_string
    } else {
        &ms_as_string[..6]
    };

    result.extend_from_slice(ms_as_slice.as_bytes());
}

fn push_with_leading_zero(result: &mut Vec<u8>, value: u8) {
    if value < 10 {
        result.push('0' as u8);
        let value = '0' as u8 + value;
        result.push(value);
    } else {
        result.extend_from_slice(value.to_string().as_bytes());
    }
}

#[derive(Debug, Clone)]
pub enum BidAskContract {
    Ping,
    Pong,
    BidAsk(BidAsk),
}

impl BidAskContract {
    pub fn is_ping(&self) -> bool {
        match self {
            BidAskContract::Ping => true,
            _ => false,
        }
    }

    pub fn parse(src: &str) -> Self {
        if src == "PING" {
            return Self::Ping;
        }
        if src == "PONG" {
            return Self::Pong;
        }

        Self::BidAsk(BidAsk::parse(src).unwrap())
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        match self {
            BidAskContract::Ping => dest.extend_from_slice(b"PING"),
            BidAskContract::Pong => dest.extend_from_slice(b"PONG"),
            BidAskContract::BidAsk(bid_ask) => bid_ask.serialize(dest),
        }
    }

    pub fn is_bid_ask(&self) -> bool {
        match self {
            BidAskContract::Ping => false,
            BidAskContract::Pong => false,
            BidAskContract::BidAsk(_) => true,
        }
    }
}

impl my_tcp_sockets::tcp_connection::TcpContract for BidAskContract {
    fn is_pong(&self) -> bool {
        match self {
            BidAskContract::Pong => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BidAsk {
    pub date_time: BidAskDateTime,
    pub id: String,
    pub bid: f64,
    pub ask: f64,
    pub source: String,
}

impl BidAsk {
    pub fn parse(src: &str) -> Option<Self> {
        let mut date_time = None;
        let mut id = None;
        let mut bid = None;
        let mut ask = None;
        let mut source = None;
        let mut no = 0;

        for line in src.split(' ') {
            match no {
                0 => {
                    date_time = BidAskDateTime::parse(line).into();
                }
                1 => id = Some(line.to_string()),
                2 => bid = Some(line.parse::<f64>().unwrap()),
                3 => ask = Some(line.parse::<f64>().unwrap()),
                4 => source = line.to_string().into(),
                _ => {}
            }
            no += 1;
        }

        let date_time = date_time?;
        let id = id?;
        let bid = bid?;
        let ask = ask?;
        let source = source?;

        Self {
            date_time,
            id,
            bid,
            ask,
            source,
        }
        .into()
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        self.date_time.serialize(dest);

        dest.push(' ' as u8);
        dest.extend_from_slice(self.id.as_bytes());
        dest.push(' ' as u8);

        dest.extend_from_slice(self.bid.to_string().as_bytes());
        dest.push(' ' as u8);
        dest.extend_from_slice(self.ask.to_string().as_bytes());
        dest.push(' ' as u8);
        dest.extend_from_slice(self.source.as_bytes());
    }
}

static CLCR: &[u8] = &[13u8, 10u8];
const MAX_PACKET_CAPACITY: usize = 255;

pub struct SourceFeedSerializer {
    read_buffer: ReadBuffer,
}

impl SourceFeedSerializer {
    pub fn new() -> Self {
        Self {
            read_buffer: ReadBuffer::new(1024 * 24),
        }
    }
}

#[async_trait]
impl TcpSocketSerializer<BidAskContract> for SourceFeedSerializer {
    const PING_PACKET_IS_SINGLETONE: bool = false;

    fn serialize(&self, contract: BidAskContract) -> Vec<u8> {
        let mut result = Vec::with_capacity(MAX_PACKET_CAPACITY);
        contract.serialize(&mut result);
        result.extend_from_slice(CLCR);
        result
    }

    // fn serialize_ref(&self, contract: &BidAskContract) -> Vec<u8> {
    //     let mut result = Vec::with_capacity(MAX_PACKET_CAPACITY);
    //     contract.serialize(&mut result);
    //     result.extend_from_slice(CLCR);
    //     result
    // }

    fn get_ping(&self) -> BidAskContract {
        return BidAskContract::Ping;
    }
    async fn deserialize<TSocketReader: Send + Sync + 'static + SocketReader>(
        &mut self,
        socket_reader: &mut TSocketReader,
    ) -> Result<BidAskContract, ReadingTcpContractFail> {
        let result = socket_reader
            .read_until_end_marker(&mut self.read_buffer, CLCR)
            .await?;

        let result = std::str::from_utf8(&result[..result.len() - CLCR.len()]).unwrap();

        Ok(BidAskContract::parse(result))
    }

    // fn apply_packet(&mut self, _contract: &BidAskContract) -> bool {
    //     false
    // }
}
