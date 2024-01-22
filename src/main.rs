use anyhow::Result;
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use tokio_modbus::Request;
use tokio_util::codec::{Encoder, Decoder, FramedRead, FramedWrite};
use tracing::info;

struct ModbusDataCodec;
struct ModbusRequestCodec;

impl Decoder for ModbusRequestCodec {
    type Item = Request<'static>;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if src has enough data to read the length (u64 in this case)
        if src.len() < 8 {
            // Wait for more bytes
            return Ok(None);
        }

        // Read the length
        let mut length_bytes = [0u8; 8];
        length_bytes.copy_from_slice(&src[..8]);
        let length = usize::try_from(u64::from_be_bytes(length_bytes))?;
        // Check if src has enough data for a complete Request
        if src.len() - 8 < length {
            // Not enough data, wait for more
            return Ok(None);
        }

        // Split the buffer at the end of the complete Request
        let _ = src.split_to(8); // Remove length bytes
        let request_bytes = src.split_to(length);

        let request = bincode::deserialize(&request_bytes)?;
        Ok(Some(request))
    }
}

impl Encoder<Request<'static>> for ModbusRequestCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Request, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let serialized = bincode::serialize(&item)?;
        let length = serialized.len() as u64;

        // Serialize the length and append it to dst
        dst.extend_from_slice(&length.to_be_bytes()); // Big endian format
        dst.extend_from_slice(&serialized); // Append the serialized Request

        Ok(())
    }
}

impl Decoder for ModbusDataCodec {
    type Item = Vec<u16>;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if src has enough data to read the length (u64 in this case)
        if src.len() < 8 {
            // Wait for more bytes
            return Ok(None);
        }

        // Read the length
        let mut length_bytes = [0u8; 8];
        length_bytes.copy_from_slice(&src[..8]);
        let length = usize::try_from(u64::from_be_bytes(length_bytes))?;
        // Check if src has enough data for a complete Request
        if src.len() - 8 < length {
            // Not enough data, wait for more
            return Ok(None);
        }

        // Split the buffer at the end of the complete Request
        let _ = src.split_to(8); // Remove length bytes
        let request_bytes = src.split_to(length);

        let request = bincode::deserialize(&request_bytes)?;
        Ok(Some(request))
    }
}

impl Encoder<Vec<u16>> for ModbusDataCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: Vec<u16>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let serialized = bincode::serialize(&item)?;
        let length = serialized.len() as u64;

        // Serialize the length and append it to dst
        dst.extend_from_slice(&length.to_be_bytes()); // Big endian format
        dst.extend_from_slice(&serialized); // Append the serialized Request

        Ok(())
    }
}



#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let socket = tokio::net::TcpStream::connect("127.0.0.1:1234").await?;
    println!("Connected to server");
    let (reader, writer) = tokio::io::split(socket);
    let mut client_reader = FramedRead::new(reader, ModbusDataCodec);
    let mut client_writer = FramedWrite::new(writer, ModbusRequestCodec);

    client_writer.send(Request::ReadHoldingRegisters(0, 16)).await?;
    let data = client_reader.next().await.unwrap()?;
    println!("Holding Register Data is: {data:?}");
    client_writer.send(Request::ReadCoils(0, 3)).await?;
    let data = client_reader.next().await.unwrap()?;
    println!("Coils Data is: {data:?}");
    client_writer.send(Request::ReadDiscreteInputs(0, 16)).await?;
    let data = client_reader.next().await.unwrap()?;
    println!("Discretes Data is: {data:?}");
    client_writer.send(Request::ReadInputRegisters(0, 16)).await?;
    let data = client_reader.next().await.unwrap()?;
    println!("Input Register Data is: {data:?}");
    client_writer.send(Request::Disconnect).await?;

    Ok(())
}